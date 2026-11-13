use crate::ast::parsed::{Accidental, NumberedPitch};
use crate::compiler::types::BarLineKind;
use crate::compositor::types::FontFamily;
use crate::pitch_description::GuitarVoicing;

use crate::grid_layout::types::SequenceEntryInfo;

/// `GridContent` after arc variants have been resolved.
/// Used in the coordinate-resolver layer; arc variants are handled before this point.
#[derive(Debug, Clone)]
pub enum PostArcGridContent {
    NoteHead {
        pitch: NumberedPitch,
        accidental: Accidental,
        octave: i8,
        dotted: bool,
        double_dotted: bool,
        head_label: Option<crate::compiler::NoteHeadLabel>,
    },
    Rest {
        dotted: bool,
        double_dotted: bool,
        implicit_fill: bool,
    },
    /// A single wide rest bar standing in for `count` consecutive
    /// all-rest source measures.
    MultiMeasureRest {
        count: u32,
    },
    NoteDash {
        dotted: bool,
        double_dotted: bool,
    },
    LowOctaveDots {
        count: u8,
    },
    ChordSymbol {
        text: String,
        dotted: bool,
        double_dotted: bool,
    },
    GuitarDiagram {
        voicing: GuitarVoicing,
    },
    PercussionHit,
    Underline {
        level: u32,
    },
    BarLine {
        height_pt: f32,
        kind: BarLineKind,
    },
    HorizontalLine,
    RowLabel(String),
    LyricSyllable {
        text: String,
        ruby: Option<String>,
        ruby_band_pt: f32,
        source_part_index: usize,
        note_id: usize,
    },
    LyricLine(String),
    DirectiveLine {
        label: Option<String>,
        bar_number: Option<u32>,
        key: Option<String>,
        bpm: Option<u32>,
        time_signature: Option<(u32, u32)>,
    },
    Text {
        content: String,
        font_size: f32,
        bold: bool,
        italic: bool,
        underline: bool,
        font_family: FontFamily,
    },
    SequenceLine {
        entries: Vec<SequenceEntryInfo>,
        font_size: f32,
    },
}
