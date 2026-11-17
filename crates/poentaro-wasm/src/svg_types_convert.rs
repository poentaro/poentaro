//! Conversions from the internal [`poentaro`] renderer types to the
//! `*Out` types in [`crate::svg_types`] that get serialized across the wasm
//! boundary. Split out from `svg_types.rs` to keep that file under the
//! repo's max-file-lines limit.
use poentaro::{
    compositor::types::{DominantBaseline, FontFamily, FontWeight, TextAnchor},
    renderer::new_types::{
        SvgDocument, SvgElement, SvgKind, SvgVariant, Tag, TransparentRectRole, TspanData,
    },
};

use crate::svg_types::{
    DominantBaselineOut, FontFamilyOut, FontWeightOut, SvgDocumentOut, SvgElementOut, SvgKindOut,
    SvgVariantOut, TagOut, TextAnchorOut, TransparentRectRoleOut, TspanOut,
};

pub(crate) fn svg_document_to_out(doc: &SvgDocument) -> SvgDocumentOut {
    SvgDocumentOut {
        width_pt: doc.width_pt,
        height_pt: doc.height_pt,
        elements: doc.elements.iter().map(svg_element_to_out).collect(),
    }
}

fn tspan_to_out(span: &TspanData) -> TspanOut {
    TspanOut {
        content: span.content.clone(),
        bold: span.bold,
        italic: span.italic,
        underline: span.underline,
        font_size: span.font_size,
    }
}

fn svg_element_to_out(el: &SvgElement) -> SvgElementOut {
    SvgElementOut {
        x: el.x,
        y: el.y,
        variant: el.variant.map(svg_variant_to_out),
        kind: svg_kind_to_out(&el.kind),
    }
}

fn svg_variant_to_out(variant: SvgVariant) -> SvgVariantOut {
    match variant {
        SvgVariant::Text => SvgVariantOut::Text,
        SvgVariant::NoteHead => SvgVariantOut::NoteHead,
        SvgVariant::UnplayableNoteHead => SvgVariantOut::UnplayableNoteHead,
        SvgVariant::HarmonicaSlide => SvgVariantOut::HarmonicaSlide,
        SvgVariant::Rest => SvgVariantOut::Rest,
        SvgVariant::OmittedPartRest => SvgVariantOut::OmittedPartRest,
        SvgVariant::MultiMeasureRest => SvgVariantOut::MultiMeasureRest,
        SvgVariant::ChordSymbol => SvgVariantOut::ChordSymbol,
        SvgVariant::PercussionHit => SvgVariantOut::PercussionHit,
        SvgVariant::HorizontalLine => SvgVariantOut::HorizontalLine,
        SvgVariant::Underline => SvgVariantOut::Underline,
        SvgVariant::TieOrSlur => SvgVariantOut::TieOrSlur,
        SvgVariant::TupletBracket => SvgVariantOut::TupletBracket,
        SvgVariant::BarLine => SvgVariantOut::BarLine,
        SvgVariant::Lyric => SvgVariantOut::Lyric,
        SvgVariant::DirectiveLine => SvgVariantOut::DirectiveLine,
        SvgVariant::GuitarDiagram => SvgVariantOut::GuitarDiagram,
    }
}

fn text_anchor_to_out(anchor: &TextAnchor) -> TextAnchorOut {
    match anchor {
        TextAnchor::Start => TextAnchorOut::Start,
        TextAnchor::Middle => TextAnchorOut::Middle,
        TextAnchor::End => TextAnchorOut::End,
    }
}

fn dominant_baseline_to_out(baseline: &DominantBaseline) -> DominantBaselineOut {
    match baseline {
        DominantBaseline::Middle => DominantBaselineOut::Middle,
        DominantBaseline::Hanging => DominantBaselineOut::Hanging,
        DominantBaseline::Ideographic => DominantBaselineOut::Ideographic,
    }
}

fn font_family_to_out(font: &FontFamily) -> FontFamilyOut {
    match font {
        FontFamily::Monospace => FontFamilyOut::Monospace,
        FontFamily::SansSerif => FontFamilyOut::SansSerif,
        FontFamily::Serif => FontFamilyOut::Serif,
    }
}

fn font_weight_to_out(weight: &FontWeight) -> FontWeightOut {
    match weight {
        FontWeight::Normal => FontWeightOut::Normal,
        FontWeight::Bold => FontWeightOut::Bold,
    }
}

fn tag_to_out(tag: &Tag) -> TagOut {
    match tag {
        Tag::Measure { index, end } => TagOut::Measure {
            index: *index,
            end: *end,
        },
        Tag::BarNumber { index, end } => TagOut::BarNumber {
            index: *index,
            end: *end,
        },
        Tag::SectionLabel { label } => TagOut::SectionLabel {
            label: label.clone(),
        },
        Tag::Note {
            source_part_index,
            note_id,
        } => TagOut::Note {
            source_part_index: *source_part_index,
            note_id: *note_id,
        },
        Tag::PartLabel {
            source_part_index,
            measure_index_start,
            measure_index_end,
        } => TagOut::PartLabel {
            source_part_index: *source_part_index,
            measure_index_start: *measure_index_start,
            measure_index_end: *measure_index_end,
        },
        Tag::Lyric {
            source_part_index,
            note_id,
        } => TagOut::Lyric {
            source_part_index: *source_part_index,
            note_id: *note_id,
        },
        Tag::LyricLabel {
            source_part_index,
            measure_index_start,
            measure_index_end,
        } => TagOut::LyricLabel {
            source_part_index: *source_part_index,
            measure_index_start: *measure_index_start,
            measure_index_end: *measure_index_end,
        },
        Tag::BarLine {
            measure_index_next,
            measure_index_prev,
        } => TagOut::BarLine {
            measure_index_next: *measure_index_next,
            measure_index_prev: *measure_index_prev,
        },
        Tag::GuitarDiagram { frets, base_fret } => TagOut::GuitarDiagram {
            frets: frets.clone(),
            base_fret: *base_fret,
        },
    }
}

fn transparent_rect_role_to_out(role: &TransparentRectRole) -> TransparentRectRoleOut {
    match role {
        TransparentRectRole::MeasureClickTarget => TransparentRectRoleOut::MeasureClickTarget,
        TransparentRectRole::BarNumberClickTarget => TransparentRectRoleOut::BarNumberClickTarget,
        TransparentRectRole::SectionLabelBackground => {
            TransparentRectRoleOut::SectionLabelBackground
        }
        TransparentRectRole::SectionLabelClickTarget => {
            TransparentRectRoleOut::SectionLabelClickTarget
        }
        TransparentRectRole::NoteClickTarget => TransparentRectRoleOut::NoteClickTarget,
        TransparentRectRole::PartLabelClickTarget => TransparentRectRoleOut::PartLabelClickTarget,
        TransparentRectRole::LyricClickTarget => TransparentRectRoleOut::LyricClickTarget,
        TransparentRectRole::LyricLabelClickTarget => TransparentRectRoleOut::LyricLabelClickTarget,
        TransparentRectRole::BarLineClickTarget => TransparentRectRoleOut::BarLineClickTarget,
    }
}

fn svg_kind_to_out(kind: &SvgKind) -> SvgKindOut {
    match kind {
        SvgKind::Text {
            content,
            font_size,
            anchor,
            baseline,
            font,
            weight,
            italic,
            underline,
        } => SvgKindOut::Text {
            content: content.clone(),
            font_size: *font_size,
            anchor: text_anchor_to_out(anchor),
            baseline: dominant_baseline_to_out(baseline),
            font: font_family_to_out(font),
            weight: font_weight_to_out(weight),
            italic: *italic,
            underline: *underline,
        },
        SvgKind::Line {
            x2,
            y2,
            stroke_width,
        } => SvgKindOut::Line {
            x2: *x2,
            y2: *y2,
            stroke_width: *stroke_width,
        },
        SvgKind::Circle { r } => SvgKindOut::Circle { r: *r },
        SvgKind::RingCircle { r, stroke_width } => ring_circle_to_out(*r, *stroke_width),
        SvgKind::RoundedBar { width, height } => rounded_bar_to_out(*width, *height),
        SvgKind::Path {
            control_x,
            control_y,
            end_x,
            end_y,
            stroke_width,
        } => SvgKindOut::Path {
            control_x: *control_x,
            control_y: *control_y,
            end_x: *end_x,
            end_y: *end_y,
            stroke_width: *stroke_width,
        },
        SvgKind::Rect { width, height } => SvgKindOut::Rect {
            width: *width,
            height: *height,
        },
        SvgKind::ErrorRect { width, height } => SvgKindOut::ErrorRect {
            width: *width,
            height: *height,
        },
        SvgKind::PlaybackCursorRect { width, height } => SvgKindOut::PlaybackCursorRect {
            width: *width,
            height: *height,
        },
        SvgKind::TransparentRect {
            width,
            height,
            role,
        } => SvgKindOut::TransparentRect {
            width: *width,
            height: *height,
            role: transparent_rect_role_to_out(role),
        },
        SvgKind::TextWithTspans {
            font_size,
            anchor,
            baseline,
            font,
            spans,
        } => text_with_tspans_to_out(*font_size, anchor, baseline, font, spans),
        SvgKind::Group { children, tag } => SvgKindOut::Group {
            children: children.iter().map(svg_element_to_out).collect(),
            tag: tag.as_ref().map(tag_to_out),
        },
    }
}

/// The guitar-diagram shape arms of [`svg_kind_to_out`]'s dispatch, split out
/// to keep that function under clippy's line-count limit.
fn ring_circle_to_out(r: f32, stroke_width: f32) -> SvgKindOut {
    SvgKindOut::RingCircle { r, stroke_width }
}

fn rounded_bar_to_out(width: f32, height: f32) -> SvgKindOut {
    SvgKindOut::RoundedBar { width, height }
}

/// The `SvgKind::TextWithTspans` arm of [`svg_kind_to_out`]'s dispatch, split
/// out to keep that function under clippy's line-count limit.
fn text_with_tspans_to_out(
    font_size: f32,
    anchor: &TextAnchor,
    baseline: &DominantBaseline,
    font: &FontFamily,
    spans: &[TspanData],
) -> SvgKindOut {
    SvgKindOut::TextWithTspans {
        font_size,
        anchor: text_anchor_to_out(anchor),
        baseline: dominant_baseline_to_out(baseline),
        font: font_family_to_out(font),
        spans: spans.iter().map(tspan_to_out).collect(),
    }
}
