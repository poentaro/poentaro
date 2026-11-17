//! The source edits of a document written in anyx expressions. Each finds the
//! node it changes in the tree, rewrites that node's text, and formats only the
//! section it sits in, so every other section keeps its layout and comments.

use itertools::Itertools;

use crate::ast::parsed::{ParsedMetadata, PresetKey, DEFAULT_PRESET_NAME};
use crate::error::Span;
use crate::parser::anyx::decoded::{decode, span, AbbreviationText, FieldName};
use crate::parser::anyx::quoted;
use crate::parser::anyx::{field_text, notation_of as read_notation, read_metadata_field};
use poentaro_grammar::{
    DecodeError, Document, DocumentPreset, Keyed, MetadataField, MetadataRecord, PresetField,
};

use super::metadata_edit::{canonical_fields, MetadataEdit, MetadataFields};
use super::preset_edit::{NewPreset, PresetEdit, PresetSummary};
use crate::ast::parsed::Notation;

pub(super) fn text_of(source: &str, span: Span) -> &str {
    source.get(span.start..span.end).unwrap_or_default()
}

pub(super) fn replace_span(source: &str, span: Span, replacement: &str) -> String {
    [
        source.get(..span.start).unwrap_or_default(),
        replacement,
        source.get(span.end..).unwrap_or_default(),
    ]
    .concat()
}

/// A top-level expression in canonical layout, without a separator after it.
pub(super) fn formatted_section(text: &str) -> String {
    anyx::format(text).map_or_else(
        |_| text.to_owned(),
        |formatted| {
            formatted
                .trim_end()
                .trim_end_matches(',')
                .trim_end()
                .to_owned()
        },
    )
}

fn find_preset<'d>(document: &'d Document, name: &str) -> Option<&'d DocumentPreset> {
    document
        .preset
        .iter()
        .find(|preset| preset.name.value.value == name)
}

/// A preset's fields that decoded, in order.
fn preset_fields(preset: &DocumentPreset) -> impl DoubleEndedIterator<Item = &PresetField> {
    preset.fields.preset_field.iter().flatten()
}

/// A section's metadata fields as decoded: a preset's metadata overrides, or
/// the metadata section's fields.
type Metadata<'d> = Vec<Result<&'d MetadataField, &'d DecodeError>>;

fn section_metadata(record: &MetadataRecord) -> Metadata<'_> {
    record.metadata_field.iter().map(Result::as_ref).collect()
}

fn preset_metadata(preset: &DocumentPreset) -> Metadata<'_> {
    preset
        .fields
        .preset_field
        .iter()
        .filter_map(|field| match field {
            Ok(PresetField::Metadata(field)) => Some(Ok(field)),
            Ok(_) => None,
            Err(error) => Some(Err(error)),
        })
        .collect()
}

// ── metadata ─────────────────────────────────────────────────────────────

/// The section Edit Metadata edits, and its record.
struct EditedSection<'d> {
    whole: anyx::Span,
    /// The `{ … }` the fields are in.
    braces: anyx::Span,
    metadata: Metadata<'d>,
    /// A preset's own fields, which the form leaves as written.
    preset_only: Vec<anyx::Span>,
    is_preset: bool,
}

fn edited_section<'d>(
    document: &'d Document,
    preset_name: Option<&str>,
) -> Option<EditedSection<'d>> {
    let preset = preset_name
        .and_then(|name| find_preset(document, name))
        .map(|preset| EditedSection {
            whole: preset.span,
            braces: preset.fields.span,
            metadata: preset_metadata(preset),
            preset_only: preset_fields(preset)
                .filter(|field| !matches!(field, PresetField::Metadata(_)))
                .map(Keyed::span)
                .collect(),
            is_preset: true,
        });
    preset.or_else(|| {
        document.metadata.as_ref().map(|section| EditedSection {
            whole: section.span,
            braces: section.fields.span,
            metadata: section_metadata(&section.fields),
            preset_only: Vec::new(),
            is_preset: false,
        })
    })
}

fn metadata_of(fields: &[Result<&MetadataField, &DecodeError>]) -> ParsedMetadata {
    let mut metadata = ParsedMetadata::default();
    fields
        .iter()
        .for_each(|field| read_metadata_field(&mut metadata, *field, &mut Vec::new()));
    metadata
}

fn fields_of(section: &EditedSection, document: &Document) -> MetadataFields {
    // As the form's text input shows it: `0 -4`.
    let directive_row_offset = section
        .metadata
        .iter()
        .flatten()
        .filter_map(|field| match field {
            MetadataField::DirectiveRowOffset { offset, .. } => {
                Some(format!("{} {}", offset.x.value, offset.y.value))
            }
            _ => None,
        })
        .next_back();
    let inherited = match &document.metadata {
        Some(metadata) if section.is_preset => metadata_of(&section_metadata(&metadata.fields)),
        _ => ParsedMetadata::default(),
    };
    MetadataFields {
        metadata: metadata_of(&section.metadata),
        directive_row_offset,
        inherited,
    }
}

pub(super) fn parse_metadata_fields(source: &str, preset_name: Option<&str>) -> MetadataFields {
    let document = decode(source).document;
    edited_section(&document, preset_name)
        .map(|section| fields_of(&section, &document))
        .unwrap_or_default()
}

/// The elements of the `{ … }` at `braces`.
fn record_elements(source: &str, braces: anyx::Span) -> Vec<anyx::Span> {
    fn find(node: &anyx::Node, braces: anyx::Span) -> Option<Vec<anyx::Span>> {
        let children: &[anyx::Node] = match &node.kind {
            anyx::NodeKind::Braces(nodes) if node.span == braces => {
                return Some(nodes.iter().map(|node| node.span).collect());
            }
            anyx::NodeKind::Parens(nodes)
            | anyx::NodeKind::Brackets(nodes)
            | anyx::NodeKind::Braces(nodes) => nodes,
            anyx::NodeKind::Call(call) => &call.arguments,
            _ => &[],
        };
        children.iter().find_map(|child| find(child, braces))
    }
    anyx::parse(source)
        .syntax
        .nodes()
        .iter()
        .find_map(|node| find(node, braces))
        .unwrap_or_default()
}

/// What the form leaves as written: a preset's own fields, text the converter
/// kept because it couldn't read it, and fields that don't decode.
fn kept_fields<'s>(source: &'s str, section: &EditedSection) -> Vec<&'s str> {
    let unconverted = section
        .metadata
        .iter()
        .flatten()
        .filter(|field| matches!(field, MetadataField::CouldNotConvert { .. }))
        .map(|field| field.span());
    let failed = section.metadata.iter().filter_map(|field| field.err());
    let undecoded = record_elements(source, section.braces)
        .into_iter()
        .filter(|element| {
            failed
                .clone()
                .any(|error| element.start <= error.span.start && error.span.end <= element.end)
        })
        .collect_vec();
    section
        .preset_only
        .iter()
        .copied()
        .chain(unconverted)
        .chain(undecoded)
        .unique_by(|at| (at.start, at.end))
        .sorted_by_key(|at| at.start)
        .map(|at| text_of(source, span(at)))
        .collect()
}

pub(super) fn update_metadata_field(
    source: &str,
    preset_name: Option<&str>,
    edit: MetadataEdit,
) -> String {
    let document = decode(source).document;
    let Some(section) = edited_section(&document, preset_name) else {
        return source.to_owned();
    };
    let mut fields = fields_of(&section, &document);
    edit.apply(&mut fields);
    let elements = kept_fields(source, &section)
        .into_iter()
        .map(str::to_owned)
        .chain(
            canonical_fields(&fields)
                .iter()
                .map(|field| field_text(&field.key, &field.value)),
        )
        .join(",\n");
    let head = Span::new(section.whole.start, section.braces.start);
    let text = format!("{}{{\n{elements}\n}}", text_of(source, head));
    replace_span(source, span(section.whole), &formatted_section(&text))
}

// ── presets ──────────────────────────────────────────────────────────────

pub(super) fn list_presets(source: &str) -> Vec<PresetSummary> {
    decode(source)
        .document
        .preset
        .iter()
        .filter_map(|preset| {
            let name = preset.name.value.value.trim();
            let is_listed = !name.is_empty() && !name.eq_ignore_ascii_case(DEFAULT_PRESET_NAME);
            is_listed.then(|| PresetSummary {
                name: name.to_owned(),
                notation: notation_of(preset),
            })
        })
        .unique_by(|preset| preset.name.clone())
        .collect()
}

fn notation_of(preset: &DocumentPreset) -> Option<Notation> {
    preset_fields(preset)
        .filter_map(|field| match field {
            PresetField::Notation { notation, .. } => Some(read_notation(notation).value),
            _ => None,
        })
        .next_back()
}

/// The preset's part list under `key` as written, `None` when it sets none or
/// there is no such preset.
pub(super) fn part_list_of(source: &str, name: &str, key: PresetKey) -> Option<Vec<String>> {
    let document = decode(source).document;
    let preset = find_preset(&document, name)?;
    let parts = preset_fields(preset)
        .filter_map(|field| match (field, key) {
            (PresetField::VisibleParts { parts, .. }, PresetKey::VisibleParts)
            | (PresetField::SoloedParts { parts, .. }, PresetKey::SoloedParts) => Some(parts),
            _ => None,
        })
        .next_back()
        .map(|parts| {
            parts
                .abbreviation
                .iter()
                .filter_map(|element| element.as_ref().ok()?.text().ok())
                .collect()
        });
    parts
}

pub(super) fn has_preset(source: &str, name: &str) -> bool {
    find_preset(&decode(source).document, name).is_some()
}

/// Where a list element that ends at `end` ends with the comma after it.
fn past_comma(source: &str, end: usize) -> usize {
    let rest = source.get(end..).unwrap_or_default();
    let trimmed = rest.trim_start();
    if trimmed.starts_with(',') {
        end + (rest.len() - trimmed.len()) + 1
    } else {
        end
    }
}

pub(super) fn update_preset_field(source: &str, name: &str, edit: &PresetEdit) -> String {
    let document = decode(source).document;
    let Some(preset) = find_preset(&document, name) else {
        return source.to_owned();
    };
    let keyword = edit.key().keyword();
    let matching = preset_fields(preset)
        .filter(|field| field.key() == keyword)
        .map(|field| span(field.span()))
        .collect_vec();
    let whole = span(preset.span);
    let text = text_of(source, whole);
    let relative = |span: Span| Span::new(span.start - whole.start, span.end - whole.start);
    let edited = if edit.removes_line() {
        matching.iter().rev().fold(text.to_owned(), |text, &span| {
            let span = relative(span);
            replace_span(
                &text,
                Span::new(span.start, past_comma(&text, span.end)),
                "",
            )
        })
    } else {
        let field = field_text(keyword, &edit.value());
        match matching.last() {
            Some(&span) => replace_span(text, relative(span), &field),
            None => {
                let inside = relative(span(preset.fields.span)).start + 1;
                replace_span(text, Span::new(inside, inside), &format!("{field},"))
            }
        }
    };
    replace_span(source, whole, &formatted_section(&edited))
}

/// Where a section that goes before `start` starts, so a comment above that
/// section stays with it.
fn above_comments(source: &str, start: usize) -> usize {
    let mut position = start;
    while let Some(line_start) = source
        .get(..position)
        .and_then(|before| before.strip_suffix('\n'))
        .map(|before| before.rfind('\n').map_or(0, |newline| newline + 1))
    {
        let line = source.get(line_start..position).unwrap_or_default();
        if !line.trim_start().starts_with('#') {
            break;
        }
        position = line_start;
    }
    position
}

pub(super) fn add_preset_section(source: &str, preset: NewPreset) -> String {
    let soloed_parts = PresetEdit::SoloedParts(preset.soloed_parts);
    let fields = [
        Some(PresetEdit::VisibleParts(preset.visible_parts)),
        (!soloed_parts.removes_line()).then_some(soloed_parts),
        Some(PresetEdit::Notation(preset.notation)),
    ]
    .into_iter()
    .flatten()
    .map(|edit| field_text(edit.key().keyword(), &edit.value()))
    .join(", ");
    let section = formatted_section(&format!("preset {} {{ {fields} }}", quoted(&preset.name)));

    let document = decode(source).document;
    let before_music = [
        document.sequence.as_ref().map(|section| section.span),
        document.score.as_ref().map(|section| section.span),
    ]
    .into_iter()
    .flatten()
    .map(|whole| whole.start)
    .min();
    let last_preset = document.preset.last().map(|preset| preset.span.end);
    // An empty document's span is empty at 0; any node ends after that.
    let last_node = (document.span.end > 0).then_some(document.span.end);
    match (last_preset, before_music) {
        (None, Some(start)) => {
            let start = above_comments(source, start);
            replace_span(source, Span::new(start, start), &format!("{section},\n\n"))
        }
        (last_preset, _) => match last_preset.or(last_node) {
            Some(end) => {
                let after_comma = past_comma(source, end);
                let comma = if after_comma == end { "," } else { "" };
                replace_span(
                    source,
                    Span::new(after_comma, after_comma),
                    &format!("{comma}\n\n{section},"),
                )
            }
            None => format!("{source}{section},\n"),
        },
    }
}
