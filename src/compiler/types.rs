use super::ChordDegrees;
use crate::ast::parsed::{Accidental, NumberedPitch, PartKind};
use crate::error::{Diagnostic, Span};
use crate::pitch_description::GuitarVoicing;

#[derive(Debug, Clone, PartialEq)]
pub enum ArcKind {
    Slur,
    Tie,
}

/// How a bar line is drawn. `Final` is the thin-then-thick double bar that
/// closes the whole score — only ever the last measure's closing bar line
/// on the last page (see `grid_layout::layout::mark_final_bar_line`).
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum BarLineKind {
    Single,
    Final,
}

/// Column width (in grid-layout columns) reserved for a collapsed
/// `MultiMeasureRest` block, regardless of how many source measures it
/// represents — wide enough to read visually as "more than one measure",
/// with the printed count communicating the actual number. Shared between
/// the compiler (which positions the block's `BarLine`) and the grid-layout
/// expansion step (which spans the `MultiMeasureRest` glyph to match).
pub const MULTI_MEASURE_REST_WIDTH: u32 = 8;

#[derive(Debug, Clone)]
pub struct MeasureBlock {
    pub rows: Vec<MeasureRow>,
    pub decorations: Vec<Decoration>,
    /// Diagnostics collected during grouping for this measure.
    pub diagnostics: Vec<Diagnostic>,
    /// Number of original source measures this block stands in for. `1` for
    /// every normal block; > 1 when a run of all-rest measures has been
    /// folded into a single `MultiMeasureRest` block.
    pub represents_measures: usize,
    /// Resolved `merge_duplicate_measures_across_parts=` setting in effect on the
    /// measure this block was compiled from (see `consolidator::consolidate_block`).
    pub merge_duplicate_measures_across_parts: bool,
    /// `break` directive in effect on the measure this block was compiled from:
    /// forces a new system to start at this block (see
    /// `grid_layout::layout_systems::pack_into_systems`). A no-op if the block
    /// would already start a system.
    pub system_break: bool,
    /// Source span of the measure(s) this block was compiled from (see
    /// `MultiPartMeasure::source_span`) — carried forward so a layout-stage
    /// diagnostic (e.g. `WarningKind::MeasureOverflow`) can point at real
    /// source text even though layout runs after grouping/compiling.
    pub source_span: Span,
}

impl PartialEq for MeasureBlock {
    fn eq(&self, other: &Self) -> bool {
        self.rows == other.rows && self.decorations == other.decorations
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MeasureRow {
    pub id: RowId,
    pub label: String,
    /// The kind of the part this row was compiled from; what a row draws
    /// (e.g. lyric syllables vs. notes) follows from it.
    pub kind: PartKind,
    pub elements: Vec<ColumnElement>,
    /// The original part index this row was compiled from, before consolidation.
    /// Used to look up slur arcs keyed by original part index.
    pub source_part_index: usize,
    /// Other parts' rows that `consolidator::consolidate_rows` merged into
    /// this one because their content was identical (see
    /// `merge_duplicate_measures_across_parts`), kept in full (not just their
    /// `RowId`) so a later pass can re-render one on its own. Empty for a row
    /// that wasn't merged with anything. Lets a later pass
    /// (`grid_layout::layout_systems`'s union-of-parts system packing) tell
    /// "this part's row is missing from this block because it's genuinely
    /// absent" (pad with a rest) apart from "this part's notes are actually
    /// right here, merged into another row" (re-render this row's own
    /// original content instead — a rest would misreport it as resting, and
    /// a blank row would misreport it as silent).
    pub absorbed_rows: Vec<MeasureRow>,
}

impl MeasureRow {
    /// The `note_id` of this row's first sounding element, if any — used to
    /// pick a single representative identity for a collapsed
    /// `MultiMeasureRest` row (see `merge_rest_run`) and by
    /// `midi::timing::note_timings_seconds`, which reads it back off the
    /// compiled block to stay in agreement with whatever the row carries.
    pub fn first_note_id(&self) -> Option<usize> {
        self.elements.iter().find_map(|el| el.note_id)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct RowId(pub String);

#[derive(Debug, Clone, PartialEq)]
pub struct ColumnElement {
    pub column: u32,
    pub content: ElementContent,
    /// Identity of the sounding note/rest this element belongs to, unique per
    /// `(source_part_index, note_id)` across the whole score (not reset per
    /// measure). Shared by a note's dash/tie-continuation columns so they
    /// highlight together during playback. `None` for elements that never
    /// sound on their own (lyrics, bar lines, underlines). A `MultiMeasureRest`
    /// collapsed-rest-run glyph reuses the `note_id` of the first underlying
    /// measure's rest for the part it stands in for (see `merge_rest_run`),
    /// so the whole run still highlights as one note during playback.
    pub note_id: Option<usize>,
}

/// What a note head reads as besides, or in place of, its numbered digit: its
/// letter name (`F♯`) or its harmonica tab (`(4)`), as set by
/// `label_note_heads`.
#[derive(Debug, Clone, PartialEq)]
pub struct NoteHeadLabel {
    pub text: String,
    /// True when the pitch can't be played on the instrument the label is
    /// for (a harmonica tab with no hole for it), drawn as a warning.
    pub unplayable: bool,
    /// True when the note is played with a chromatic harmonica's slide
    /// button pressed in, drawn as an arrow (`←`) above the label.
    pub slide_button: bool,
    pub placement: HeadLabelPlacement,
}

/// Where a `NoteHeadLabel` is drawn relative to the note head's degree.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HeadLabelPlacement {
    /// Drawn in place of the degree: the letter-name (`F♯`) and
    /// harmonica-tab (`(4)`) views.
    InPlaceOfDegree,
    /// Drawn after the degree, in parentheses: the guitar chord view
    /// (`3 (F♯)`).
    AfterDegree,
}

impl NoteHeadLabel {
    pub fn playable(text: String) -> Self {
        Self {
            text,
            unplayable: false,
            slide_button: false,
            placement: HeadLabelPlacement::InPlaceOfDegree,
        }
    }

    /// A label for a pitch the instrument can't play, drawn as a warning.
    pub fn unplayable(text: String) -> Self {
        Self {
            unplayable: true,
            ..Self::playable(text)
        }
    }

    /// Whether the label is drawn in place of the degree rather than after
    /// it.
    pub fn replaces_degree(&self) -> bool {
        self.placement == HeadLabelPlacement::InPlaceOfDegree
    }

    /// What the note head's text run reads after its leading sharp/flat (if
    /// any): the label alone, or `digit` followed by it in parentheses.
    pub fn run_text(&self, digit: char) -> String {
        match self.placement {
            HeadLabelPlacement::InPlaceOfDegree => self.text.clone(),
            HeadLabelPlacement::AfterDegree => format!("{digit} ({})", self.text),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum ElementContent {
    NoteHead {
        pitch: NumberedPitch,
        accidental: Accidental,
        octave: i8,
        dotted: bool,
        double_dotted: bool,
        head_label: Option<NoteHeadLabel>,
    },
    Rest {
        dotted: bool,
        double_dotted: bool,
        /// True when this rest fills a part not mentioned in this measure
        /// rather than being written by the composer — rendered with a
        /// distinct glyph. See `ParsedRest::implicit_fill`.
        implicit_fill: bool,
    },
    /// A single wide rest bar standing in for `count` consecutive
    /// all-rest source measures (cross-measure collapsing).
    MultiMeasureRest {
        count: usize,
    },
    ChordSymbol {
        text: String,
        /// The chord `text` was formatted from, kept so the letter-name pass
        /// (`label_note_heads`) can respell it.
        degrees: ChordDegrees,
        dotted: bool,
        double_dotted: bool,
        /// The chord diagram the guitar chord view draws above `text`
        /// (`label_note_heads` with `NoteLabels::GuitarChords`); `None` in
        /// every other view, and for a chord chords-db has no voicing for.
        guitar_voicing: Option<GuitarVoicing>,
    },
    PercussionHit,
    Underline {
        from_column: u32,
        to_column: u32,
        last_head_column: u32,
        level: u32,
    },
    BarLine,
    /// Visual dash rendered after a note head for each extra beat of duration (e.g. `1-`).
    /// `dotted`/`double_dotted` are true when the dash came from a `-.`/`-..`
    /// (dotted-beat/double-dotted-beat) extension rather than a plain `-`, and are
    /// rendered with trailing dot(s) to match.
    NoteDash {
        dotted: bool,
        double_dotted: bool,
    },
    /// A syllable of a lyric part.
    Lyric {
        text: String,
        /// Identity of the note this syllable is sung on — the same id its
        /// note's own `ColumnElement::note_id` carries (not stored on this
        /// element's own `ColumnElement::note_id`, which stays `None`: see
        /// that field's doc comment). Lets the SVG preview select a syllable
        /// independently of its note while still being able to map back to
        /// the right source note if ever needed; primarily used together
        /// with `source_part_index` to give each syllable a click target
        /// distinct from its note's (see `renderer::new_types::Tag::Lyric`).
        note_id: usize,
    },
    /// A lyric part's full text line, adurational and not tied to any note,
    /// rendered as a single left-aligned block spanning the whole measure
    /// rather than one syllable per column. Currently unreachable: nothing
    /// constructs this variant since the standalone `lyrics` part kind was
    /// removed; kept for the downstream rendering machinery that still
    /// consumes it, in case a future adurational-text feature needs it.
    LyricLine {
        text: String,
    },
}

/// The full logical extent of one slur or tie arc across measures.
/// Resolved into grid arc elements by the layout stage.
#[derive(Debug, Clone, PartialEq)]
pub struct SlurSpan {
    pub kind: ArcKind,
    pub part_index: usize,
    pub from_measure: usize, // 0-indexed position in the final `CompileResult.blocks` list, after rest-run merging
    pub from_column: u32,    // measure-relative column of the opening note
    pub to_measure: usize,
    pub to_column: u32, // measure-relative column of the closing note
}

/// The full logical extent of one tuplet bracket over a contiguous run of
/// tuplet-tagged notes/rests sharing the same ratio. Unlike `SlurSpan`,
/// never crosses a measure or system boundary — tuplets can't span lines
/// (see the `crate::tuplet` docs) — so one `measure_index` suffices.
#[derive(Debug, Clone, PartialEq)]
pub struct TupletSpan {
    pub part_index: usize,
    pub measure_index: usize, // 0-indexed position in the final `CompileResult.blocks` list, after rest-run merging
    pub from_column: u32,     // measure-relative column of the first tuplet-tagged note/rest
    pub to_column: u32,       // measure-relative column of the last tuplet-tagged note/rest
    pub label: String,        // the tuplet's printed digit, e.g. "3" for a triplet
}

/// Return value of `compiler::compile`.
#[derive(Debug, Clone, PartialEq)]
pub struct CompileResult {
    pub blocks: Vec<MeasureBlock>,
    pub slur_spans: Vec<SlurSpan>,
    pub tuplet_spans: Vec<TupletSpan>,
    /// Which lyric part sings along to which part, by row id. Built from the
    /// part-level `LyricsSlice::target_name`; rows themselves carry no target.
    /// Grid layout reads it to group a note row with the lyric rows under it.
    pub lyric_links: Vec<LyricLink>,
}

/// A lyric part's link to the part it sings along to, as row ids. The target
/// may be absent from a block's rows when it is hidden or filtered out.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct LyricLink {
    pub lyric_row_id: RowId,
    pub target_row_id: RowId,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Decoration {
    DirectiveLine {
        label: Option<String>,
        bar_number: Option<u32>,
        key: Option<String>,
        bpm: Option<u32>,
        time_signature: Option<(u32, u32)>,
    },
}
