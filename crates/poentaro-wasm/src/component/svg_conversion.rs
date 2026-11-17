use super::*;

pub(super) fn text_anchor_to_wit(anchor: &crate::svg_types::TextAnchorOut) -> TextAnchor {
    match anchor {
        crate::svg_types::TextAnchorOut::Start => TextAnchor::Start,
        crate::svg_types::TextAnchorOut::Middle => TextAnchor::Middle,
        crate::svg_types::TextAnchorOut::End => TextAnchor::End,
    }
}

pub(super) fn dominant_baseline_to_wit(
    baseline: &crate::svg_types::DominantBaselineOut,
) -> DominantBaseline {
    match baseline {
        crate::svg_types::DominantBaselineOut::Middle => DominantBaseline::Middle,
        crate::svg_types::DominantBaselineOut::Hanging => DominantBaseline::Hanging,
        crate::svg_types::DominantBaselineOut::Ideographic => DominantBaseline::Ideographic,
    }
}

pub(super) fn font_family_to_wit(font: &crate::svg_types::FontFamilyOut) -> FontFamily {
    match font {
        crate::svg_types::FontFamilyOut::Monospace => FontFamily::Monospace,
        crate::svg_types::FontFamilyOut::SansSerif => FontFamily::SansSerif,
        crate::svg_types::FontFamilyOut::Serif => FontFamily::Serif,
    }
}

pub(super) fn font_weight_to_wit(weight: &crate::svg_types::FontWeightOut) -> FontWeight {
    match weight {
        crate::svg_types::FontWeightOut::Normal => FontWeight::Normal,
        crate::svg_types::FontWeightOut::Bold => FontWeight::Bold,
    }
}

pub(super) fn svg_variant_to_wit(variant: crate::svg_types::SvgVariantOut) -> SvgVariant {
    use crate::svg_types::SvgVariantOut;
    match variant {
        SvgVariantOut::Text => SvgVariant::Text,
        SvgVariantOut::NoteHead => SvgVariant::NoteHead,
        SvgVariantOut::UnplayableNoteHead => SvgVariant::UnplayableNoteHead,
        SvgVariantOut::HarmonicaSlide => SvgVariant::HarmonicaSlide,
        SvgVariantOut::Rest => SvgVariant::Rest,
        SvgVariantOut::OmittedPartRest => SvgVariant::OmittedPartRest,
        SvgVariantOut::MultiMeasureRest => SvgVariant::MultiMeasureRest,
        SvgVariantOut::ChordSymbol => SvgVariant::ChordSymbol,
        SvgVariantOut::PercussionHit => SvgVariant::PercussionHit,
        SvgVariantOut::HorizontalLine => SvgVariant::HorizontalLine,
        SvgVariantOut::Underline => SvgVariant::Underline,
        SvgVariantOut::TieOrSlur => SvgVariant::TieOrSlur,
        SvgVariantOut::TupletBracket => SvgVariant::TupletBracket,
        SvgVariantOut::BarLine => SvgVariant::BarLine,
        SvgVariantOut::Lyric => SvgVariant::Lyric,
        SvgVariantOut::DirectiveLine => SvgVariant::DirectiveLine,
        SvgVariantOut::GuitarDiagram => SvgVariant::GuitarDiagram,
    }
}

pub(super) fn transparent_rect_role_to_wit(
    role: &crate::svg_types::TransparentRectRoleOut,
) -> TransparentRectRole {
    match role {
        crate::svg_types::TransparentRectRoleOut::MeasureClickTarget => {
            TransparentRectRole::MeasureClickTarget
        }
        crate::svg_types::TransparentRectRoleOut::BarNumberClickTarget => {
            TransparentRectRole::BarNumberClickTarget
        }
        crate::svg_types::TransparentRectRoleOut::SectionLabelBackground => {
            TransparentRectRole::SectionLabelBackground
        }
        crate::svg_types::TransparentRectRoleOut::SectionLabelClickTarget => {
            TransparentRectRole::SectionLabelClickTarget
        }
        crate::svg_types::TransparentRectRoleOut::NoteClickTarget => {
            TransparentRectRole::NoteClickTarget
        }
        crate::svg_types::TransparentRectRoleOut::PartLabelClickTarget => {
            TransparentRectRole::PartLabelClickTarget
        }
        crate::svg_types::TransparentRectRoleOut::LyricClickTarget => {
            TransparentRectRole::LyricClickTarget
        }
        crate::svg_types::TransparentRectRoleOut::LyricLabelClickTarget => {
            TransparentRectRole::LyricLabelClickTarget
        }
        crate::svg_types::TransparentRectRoleOut::BarLineClickTarget => {
            TransparentRectRole::BarLineClickTarget
        }
    }
}

pub(super) fn tspan_to_wit(tspan: &crate::svg_types::TspanOut) -> Tspan {
    Tspan {
        content: tspan.content.clone(),
        bold: tspan.bold,
        italic: tspan.italic,
        underline: tspan.underline,
        font_size: tspan.font_size,
    }
}

pub(super) fn tag_to_wit(tag: &crate::svg_types::TagOut) -> Tag {
    match tag {
        crate::svg_types::TagOut::Measure { index, end } => Tag::Measure(MeasureTag {
            index: *index as u32,
            end: *end as u32,
        }),
        crate::svg_types::TagOut::BarNumber { index, end } => Tag::BarNumber(BarNumberTag {
            index: *index as u32,
            end: *end as u32,
        }),
        crate::svg_types::TagOut::SectionLabel { label } => Tag::SectionLabel(SectionLabelTag {
            label: label.clone(),
        }),
        crate::svg_types::TagOut::Note {
            source_part_index,
            note_id,
        } => Tag::Note(NoteTag {
            source_part_index: *source_part_index as u32,
            note_id: *note_id as u32,
        }),
        crate::svg_types::TagOut::PartLabel {
            source_part_index,
            measure_index_start,
            measure_index_end,
        } => Tag::PartLabel(PartLabelTag {
            source_part_index: *source_part_index as u32,
            measure_index_start: *measure_index_start as u32,
            measure_index_end: *measure_index_end as u32,
        }),
        crate::svg_types::TagOut::Lyric {
            source_part_index,
            note_id,
        } => Tag::Lyric(LyricTag {
            source_part_index: *source_part_index as u32,
            note_id: *note_id as u32,
        }),
        crate::svg_types::TagOut::LyricLabel {
            source_part_index,
            measure_index_start,
            measure_index_end,
        } => Tag::LyricLabel(LyricLabelTag {
            source_part_index: *source_part_index as u32,
            measure_index_start: *measure_index_start as u32,
            measure_index_end: *measure_index_end as u32,
        }),
        crate::svg_types::TagOut::BarLine {
            measure_index_next,
            measure_index_prev,
        } => Tag::BarLine(BarLineTag {
            measure_index_next: measure_index_next.map(|v| v as u32),
            measure_index_prev: measure_index_prev.map(|v| v as u32),
        }),
        crate::svg_types::TagOut::GuitarDiagram { frets, base_fret } => {
            Tag::GuitarDiagram(GuitarDiagramTag {
                frets: frets.clone(),
                base_fret: *base_fret,
            })
        }
    }
}

pub(super) fn svg_text_kind_to_wit(
    content: &str,
    font_size: f32,
    anchor: &crate::svg_types::TextAnchorOut,
    baseline: &crate::svg_types::DominantBaselineOut,
    font: &crate::svg_types::FontFamilyOut,
    weight: &crate::svg_types::FontWeightOut,
    italic: bool,
    underline: bool,
) -> SvgTextKind {
    SvgTextKind {
        content: content.to_string(),
        font_size,
        anchor: text_anchor_to_wit(anchor),
        baseline: dominant_baseline_to_wit(baseline),
        font: font_family_to_wit(font),
        weight: font_weight_to_wit(weight),
        italic,
        underline,
    }
}

pub(super) fn svg_text_with_tspans_kind_to_wit(
    font_size: f32,
    anchor: &crate::svg_types::TextAnchorOut,
    baseline: &crate::svg_types::DominantBaselineOut,
    font: &crate::svg_types::FontFamilyOut,
    spans: &[crate::svg_types::TspanOut],
) -> SvgTextWithTspansKind {
    SvgTextWithTspansKind {
        font_size,
        anchor: text_anchor_to_wit(anchor),
        baseline: dominant_baseline_to_wit(baseline),
        font: font_family_to_wit(font),
        spans: spans.iter().map(tspan_to_wit).collect(),
    }
}

pub(super) fn svg_path_kind_to_wit(
    control_x: f32,
    control_y: f32,
    end_x: f32,
    end_y: f32,
    stroke_width: f32,
) -> SvgPathKind {
    SvgPathKind {
        control_x,
        control_y,
        end_x,
        end_y,
        stroke_width,
    }
}

pub(super) fn svg_transparent_rect_kind_to_wit(
    width: f32,
    height: f32,
    role: &crate::svg_types::TransparentRectRoleOut,
) -> SvgTransparentRectKind {
    SvgTransparentRectKind {
        width,
        height,
        role: transparent_rect_role_to_wit(role),
    }
}

pub(super) fn svg_line_kind_to_wit(x2: f32, y2: f32, stroke_width: f32) -> SvgLineKind {
    SvgLineKind {
        x2,
        y2,
        stroke_width,
    }
}

pub(super) fn svg_circle_kind_to_wit(r: f32) -> SvgCircleKind {
    SvgCircleKind { r }
}

pub(super) fn svg_ring_circle_kind_to_wit(r: f32, stroke_width: f32) -> SvgRingCircleKind {
    SvgRingCircleKind { r, stroke_width }
}

pub(super) fn svg_rounded_bar_kind_to_wit(width: f32, height: f32) -> SvgRoundedBarKind {
    SvgRoundedBarKind { width, height }
}

pub(super) fn svg_rect_kind_to_wit(width: f32, height: f32) -> SvgRectKind {
    SvgRectKind { width, height }
}

pub(super) fn svg_error_rect_kind_to_wit(width: f32, height: f32) -> SvgErrorRectKind {
    SvgErrorRectKind { width, height }
}

pub(super) fn svg_playback_cursor_rect_kind_to_wit(
    width: f32,
    height: f32,
) -> SvgPlaybackCursorRectKind {
    SvgPlaybackCursorRectKind { width, height }
}
