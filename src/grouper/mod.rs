use crate::ast::grouped::{
    default_font_size, resolve_text_style, GroupedScore, GroupedTrack, Metadata, Score, TextStyle,
    TextStyleDefaults, DEFAULT_CHORDS_HORIZONTAL_PADDING_PT, DEFAULT_DIRECTIVE_ROW_OFFSET,
    DEFAULT_HIDE_RESTING_PARTS, DEFAULT_HIDE_SYSTEM_DIVIDERS, DEFAULT_LYRICS_HORIZONTAL_PADDING_PT,
    DEFAULT_LYRIC_CLICK_TARGET_PADDING_PT, DEFAULT_MAX_MEASURES_PER_SYSTEM,
    DEFAULT_MERGE_DUPLICATE_MEASURES_ACROSS_PARTS, DEFAULT_NOTES_HORIZONTAL_PADDING_PT,
    DEFAULT_NOTE_DASH_HORIZONTAL_PADDING_PT, DEFAULT_NOTE_NUMBER_WIDTH, DEFAULT_PAGE_MARGIN_PT,
    DEFAULT_PARTS_LIST_COLUMNS, DEFAULT_PART_LABEL_WIDTH_PT, DEFAULT_ROW_HEIGHT,
};
use crate::ast::parsed::{ParsedDocument, ParsedMeasureSlot, ParsedMetadata, ParsedTrack};
use crate::ast::parsed::{PartKind, TextStyleKind};
use crate::combiner;
use crate::error::{Diagnostic, IrrecoverableError};

#[path = "empty_note_measures.rs"]
mod empty_note_measures;

mod directive_grouper;
mod lyrics_pairing;
mod metadata_defaults;
mod part_grouper;
mod range_validation;
mod sequence_resolution;
mod simple_text_styles;
mod text_style_helpers;
mod tie_validation;

use directive_grouper::DirectiveGrouper;
pub use metadata_defaults::metadata_defaults;
use part_grouper::{group_lyrics_track, group_timed_track};
use range_validation::validate_ranges;
use sequence_resolution::resolve_sequence;
use simple_text_styles::resolve_simple_text_styles;
use text_style_helpers::resolve_running_title;
use tie_validation::validate_ties;

pub fn group(doc: ParsedDocument) -> Result<Score, IrrecoverableError> {
    let metadata = doc.metadata;
    let sequence = doc.sequence;
    let sequence_parse_errors = doc.sequence_parse_errors;
    let declarations = doc.declarations;
    let document_diagnostics: Vec<Diagnostic> = doc
        .section_structure_errors
        .into_iter()
        .chain(doc.metadata_parse_errors)
        .chain(doc.parts_parse_errors)
        .map(Diagnostic::Error)
        .chain(doc.preset_diagnostics)
        .collect();
    let global_resolution_multipliers = compute_global_resolution_multipliers(&doc.tracks);
    let mut grouped_tracks: Vec<GroupedTrack> = Vec::new();
    for track in doc.tracks {
        let ParsedTrack::Timed(part) = track;
        let grouped = match part.kind {
            // A lyric part's target is declared (and so already grouped) before it.
            PartKind::Lyrics { target_part_index } => {
                let target = grouped_tracks
                    .get(target_part_index)
                    .map(|track| match track {
                        GroupedTrack::Timed(target) => target,
                    });
                target.map(|target| group_lyrics_track(part, target))
            }
            _ => Some(group_timed_track(part, &global_resolution_multipliers)?),
        };
        grouped_tracks.extend(grouped.map(GroupedTrack::Timed));
    }

    let measure_directives = DirectiveGrouper::new(
        metadata
            .merge_duplicate_measures_across_parts
            .unwrap_or(DEFAULT_MERGE_DUPLICATE_MEASURES_ACROSS_PARTS),
        metadata
            .hide_resting_parts
            .unwrap_or(DEFAULT_HIDE_RESTING_PARTS),
    )
    .process_all(&doc.directive_events_per_measure);

    let grouped_score = GroupedScore {
        measure_directives,
        parts: grouped_tracks,
        per_measure_parse_errors: doc.per_measure_parse_errors,
    };

    let (measures, combiner_diagnostics) = combiner::combine(&grouped_score);

    let mut score = Score {
        metadata: resolve_metadata(metadata),
        measures,
        document_diagnostics: document_diagnostics
            .into_iter()
            .chain(combiner_diagnostics)
            .collect(),
        sequence: None,
    };
    validate_ties(&mut score);
    validate_ranges(&mut score, &declarations);
    resolve_sequence(&mut score, sequence, sequence_parse_errors, &declarations);
    Ok(score)
}

/// For each measure index, the tuplet-rescale multiplier every part must use at that
/// measure: `lcm` of every part's own `resolution_multiplier_of` at that index. Parts
/// share a measure's column space (see `MeasureBlock` in `grid_layout`), so if each part
/// rescaled tuplets against only its own content, a triplet in one part would desync
/// that measure's grid from a sibling part with no tuplet (or a different tuplet ratio),
/// leaving their notes misaligned at matching beats. Computing one multiplier per
/// measure index up front, before any part is grouped, keeps every part on the same
/// rescaled grid for that measure.
fn compute_global_resolution_multipliers(tracks: &[ParsedTrack]) -> Vec<u32> {
    let measure_count = tracks
        .iter()
        .map(|track| match track {
            ParsedTrack::Timed(part) => part.measure_slots.len(),
        })
        .max()
        .unwrap_or(0);
    (0..measure_count)
        .map(|slot_index| {
            tracks
                .iter()
                .filter_map(|track| match track {
                    ParsedTrack::Timed(part) => part.measure_slots.get(slot_index),
                })
                .filter_map(|slot| match slot {
                    ParsedMeasureSlot::Real { events } => {
                        Some(crate::tuplet::resolution_multiplier_of(events))
                    }
                    ParsedMeasureSlot::EmptyNote { .. } => None,
                })
                .chain(chord_lyrics_multiplier(tracks, slot_index))
                .fold(1, crate::tuplet::lcm)
        })
        .collect()
}

/// The multiplier measure `slot_index` needs for lyric lines under a chord part:
/// room for their unbracketed syllables between chord attacks (see
/// `chord_lyric_placement`). `None` when no such line has any.
fn chord_lyrics_multiplier(tracks: &[ParsedTrack], slot_index: usize) -> Option<u32> {
    tracks
        .iter()
        .filter_map(|track| match track {
            ParsedTrack::Timed(part) => match part.kind {
                PartKind::Lyrics { target_part_index }
                    if matches!(
                        tracks.get(target_part_index),
                        Some(ParsedTrack::Timed(target)) if target.kind == PartKind::Chords
                    ) =>
                {
                    part.lyrics.as_ref()?.measure_syllables.get(slot_index)
                }
                _ => None,
            },
        })
        .map(|syllables| crate::chord_lyric_placement::unbracketed_resolution_multiplier(syllables))
        .reduce(crate::tuplet::lcm)
}

/// The fourteen `TextStyle` fields of `Metadata`, resolved separately from
/// `resolve_metadata`'s other (non-text-style) fields to keep each function
/// under clippy's line-count limit.
struct ResolvedTextStyles {
    title_style: TextStyle,
    subtitle_style: TextStyle,
    author_style: TextStyle,
    sequence: TextStyle,
    part_legend: TextStyle,
    measure_number: TextStyle,
    section_label: TextStyle,
    part_label: TextStyle,
    page_number: TextStyle,
    running_title: TextStyle,
    lyrics: TextStyle,
    notes: TextStyle,
    chords: TextStyle,
    note_dash: TextStyle,
}

/// Resolves every text kind's `TextStyle`, applying each documented default
/// (see `Metadata`'s per-kind field docs) to whichever components the
/// `metadata` section left unset.
fn resolve_text_styles(metadata: &ParsedMetadata, row_height: u32) -> ResolvedTextStyles {
    let (
        title_style,
        subtitle_style,
        author_style,
        sequence,
        part_legend,
        measure_number,
        section_label,
        page_number,
        part_label,
    ) = resolve_simple_text_styles(metadata, row_height);
    let font_size = |kind| default_font_size(metadata, kind, row_height);
    ResolvedTextStyles {
        title_style,
        subtitle_style,
        author_style,
        sequence,
        part_legend,
        measure_number,
        section_label,
        part_label,
        page_number,
        running_title: resolve_running_title(metadata, row_height),
        lyrics: resolve_text_style(
            metadata.lyrics_style,
            font_size(TextStyleKind::Lyrics),
            DEFAULT_LYRICS_HORIZONTAL_PADDING_PT,
            DEFAULT_LYRIC_CLICK_TARGET_PADDING_PT,
            TextStyleDefaults {
                bold: false,
                italic: false,
                font_family: crate::compositor::types::FontFamily::Serif,
            },
        ),
        notes: resolve_text_style(
            metadata.notes_style,
            font_size(TextStyleKind::Notes),
            DEFAULT_NOTES_HORIZONTAL_PADDING_PT,
            0,
            TextStyleDefaults {
                bold: false,
                italic: false,
                font_family: crate::compositor::types::FontFamily::Monospace,
            },
        ),
        chords: resolve_text_style(
            metadata.chords_style,
            font_size(TextStyleKind::Chords),
            DEFAULT_CHORDS_HORIZONTAL_PADDING_PT,
            0,
            TextStyleDefaults {
                bold: false,
                italic: false,
                font_family: crate::compositor::types::FontFamily::Monospace,
            },
        ),
        note_dash: resolve_text_style(
            metadata.note_dash_style,
            font_size(TextStyleKind::NoteDash),
            DEFAULT_NOTE_DASH_HORIZONTAL_PADDING_PT,
            0,
            TextStyleDefaults {
                bold: false,
                italic: false,
                font_family: crate::compositor::types::FontFamily::Monospace,
            },
        ),
    }
}

/// Fills in each unset `metadata` field with its documented default.
fn resolve_metadata(metadata: ParsedMetadata) -> Metadata {
    let row_height = metadata.row_height.unwrap_or(DEFAULT_ROW_HEIGHT);
    let styles = resolve_text_styles(&metadata, row_height);
    Metadata {
        title: metadata.title,
        subtitle: metadata.subtitle,
        author: metadata.author,
        row_height,
        max_measures_per_system: metadata
            .max_measures_per_system
            .unwrap_or(DEFAULT_MAX_MEASURES_PER_SYSTEM),
        note_number_width: metadata
            .note_number_width
            .unwrap_or(DEFAULT_NOTE_NUMBER_WIDTH),
        parts_list_columns: metadata
            .parts_list_columns
            .unwrap_or(DEFAULT_PARTS_LIST_COLUMNS),
        part_label_width_pt: metadata
            .part_label_width_pt
            .unwrap_or(DEFAULT_PART_LABEL_WIDTH_PT),
        page_margin_pt: metadata.page_margin_pt.unwrap_or(DEFAULT_PAGE_MARGIN_PT),
        title_style: styles.title_style,
        subtitle_style: styles.subtitle_style,
        author_style: styles.author_style,
        sequence: styles.sequence,
        part_legend: styles.part_legend,
        measure_number: styles.measure_number,
        section_label: styles.section_label,
        part_label: styles.part_label,
        page_number: styles.page_number,
        running_title: styles.running_title,
        lyrics: styles.lyrics,
        notes: styles.notes,
        chords: styles.chords,
        note_dash: styles.note_dash,
        merge_duplicate_measures_across_parts: metadata
            .merge_duplicate_measures_across_parts
            .unwrap_or(DEFAULT_MERGE_DUPLICATE_MEASURES_ACROSS_PARTS),
        hide_resting_parts: metadata
            .hide_resting_parts
            .unwrap_or(DEFAULT_HIDE_RESTING_PARTS),
        hide_system_dividers: metadata
            .hide_system_dividers
            .unwrap_or(DEFAULT_HIDE_SYSTEM_DIVIDERS),
        directive_row_offset: metadata
            .directive_row_offset
            .unwrap_or(DEFAULT_DIRECTIVE_ROW_OFFSET),
    }
}

#[cfg(test)]
#[path = "tests_metadata_defaults.rs"]
mod tests_metadata_defaults;

#[cfg(test)]
mod percussion_tests;

#[cfg(test)]
mod tests;

#[cfg(test)]
#[path = "tests_lyrics.rs"]
mod tests_lyrics;

#[cfg(test)]
#[path = "tests_tie.rs"]
mod tests_tie;
