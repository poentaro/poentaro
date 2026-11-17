use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
pub struct SvgDocumentOut {
    pub width_pt: f32,
    pub height_pt: f32,
    pub elements: Vec<SvgElementOut>,
}

#[derive(Debug, Clone, Serialize)]
pub struct SvgElementOut {
    pub x: f32,
    pub y: f32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub variant: Option<SvgVariantOut>,
    pub kind: SvgKindOut,
}

#[derive(Debug, Clone, Copy, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum SvgVariantOut {
    Text,
    NoteHead,
    UnplayableNoteHead,
    HarmonicaSlide,
    Rest,
    OmittedPartRest,
    MultiMeasureRest,
    ChordSymbol,
    PercussionHit,
    HorizontalLine,
    Underline,
    TieOrSlur,
    TupletBracket,
    BarLine,
    Lyric,
    DirectiveLine,
    GuitarDiagram,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum TransparentRectRoleOut {
    MeasureClickTarget,
    BarNumberClickTarget,
    SectionLabelBackground,
    SectionLabelClickTarget,
    NoteClickTarget,
    PartLabelClickTarget,
    LyricClickTarget,
    LyricLabelClickTarget,
    BarLineClickTarget,
}

#[derive(Debug, Clone, Serialize)]
#[serde(tag = "type", rename_all = "camelCase")]
pub enum SvgKindOut {
    Text {
        content: String,
        font_size: f32,
        anchor: TextAnchorOut,
        baseline: DominantBaselineOut,
        font: FontFamilyOut,
        weight: FontWeightOut,
        italic: bool,
        underline: bool,
    },
    Line {
        x2: f32,
        y2: f32,
        stroke_width: f32,
    },
    Circle {
        r: f32,
    },
    RingCircle {
        r: f32,
        stroke_width: f32,
    },
    RoundedBar {
        width: f32,
        height: f32,
    },
    Path {
        control_x: f32,
        control_y: f32,
        end_x: f32,
        end_y: f32,
        stroke_width: f32,
    },
    Rect {
        width: f32,
        height: f32,
    },
    ErrorRect {
        width: f32,
        height: f32,
    },
    PlaybackCursorRect {
        width: f32,
        height: f32,
    },
    TransparentRect {
        width: f32,
        height: f32,
        role: TransparentRectRoleOut,
    },
    TextWithTspans {
        font_size: f32,
        anchor: TextAnchorOut,
        baseline: DominantBaselineOut,
        font: FontFamilyOut,
        spans: Vec<TspanOut>,
    },
    Group {
        children: Vec<SvgElementOut>,
        tag: Option<TagOut>,
    },
}

#[derive(Debug, Clone, Serialize)]
pub struct TspanOut {
    pub content: String,
    pub bold: bool,
    pub italic: bool,
    pub underline: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub font_size: Option<f32>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(tag = "type", rename_all = "camelCase")]
pub enum TagOut {
    Measure {
        index: usize,
        end: usize,
    },
    BarNumber {
        index: usize,
        end: usize,
    },
    SectionLabel {
        label: String,
    },
    Note {
        source_part_index: usize,
        note_id: usize,
    },
    PartLabel {
        source_part_index: usize,
        measure_index_start: usize,
        measure_index_end: usize,
    },
    Lyric {
        source_part_index: usize,
        note_id: usize,
    },
    LyricLabel {
        source_part_index: usize,
        measure_index_start: usize,
        measure_index_end: usize,
    },
    BarLine {
        measure_index_next: Option<usize>,
        measure_index_prev: Option<usize>,
    },
    GuitarDiagram {
        frets: String,
        base_fret: u8,
    },
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum TextAnchorOut {
    Start,
    Middle,
    End,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum DominantBaselineOut {
    Middle,
    Hanging,
    Ideographic,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum FontFamilyOut {
    Monospace,
    SansSerif,
    Serif,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum FontWeightOut {
    Normal,
    Bold,
}
