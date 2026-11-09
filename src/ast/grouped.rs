use crate::ast::parsed::{KeyChange, Offset, Syllable};
use crate::error::{Diagnostic, RecoverableError, Span, Warning};

// ── Public final types ────────────────────────────────────────────────────────

#[path = "grouped_text_style.rs"]
mod grouped_text_style;
pub(crate) use grouped_text_style::{default_font_size, resolve_text_style, TextStyleDefaults};
pub use grouped_text_style::{
    FontSizeDefault, TextStyle, DEFAULT_CHORDS_HORIZONTAL_PADDING_PT, DEFAULT_DIRECTIVE_ROW_OFFSET,
    DEFAULT_HEADER_VERTICAL_PADDING_PT, DEFAULT_HIDE_RESTING_PARTS, DEFAULT_HIDE_SYSTEM_DIVIDERS,
    DEFAULT_LYRICS_HORIZONTAL_PADDING_PT, DEFAULT_LYRIC_CLICK_TARGET_PADDING_PT,
    DEFAULT_MAX_MEASURES_PER_SYSTEM, DEFAULT_MERGE_DUPLICATE_MEASURES_ACROSS_PARTS,
    DEFAULT_NOTES_HORIZONTAL_PADDING_PT, DEFAULT_NOTE_DASH_HORIZONTAL_PADDING_PT,
    DEFAULT_NOTE_NUMBER_WIDTH, DEFAULT_PAGE_MARGIN_PT, DEFAULT_PARTS_LIST_COLUMNS,
    DEFAULT_PART_LABEL_WIDTH_PT, DEFAULT_ROW_HEIGHT,
};

#[derive(Debug, Clone)]
pub struct Metadata {
    pub title: Option<String>,
    pub subtitle: Option<String>,
    pub author: Option<String>,
    /// Row height in points. Controls font sizes, dot radii, and all vertical spacing. Default: 24.
    pub row_height: u32,
    /// Maximum number of measures per system line before wrapping. Default: 4.
    pub max_measures_per_system: u32,
    /// Estimated rendered width of a single digit note number (0–9) in points. Default: 8.
    pub note_number_width: u32,
    /// Number of columns in the parts list header. Default: 4.
    pub parts_list_columns: u32,
    /// Fixed width in points of the part-label column at the start of each
    /// system row, shared by every system in the score regardless of that
    /// system's musical density (see `grid_layout::types::GridRow::column_geometry`).
    /// A flat scalar field, not part of `part_label`'s `TextStyle`, since
    /// it's a layout constant rather than a text style component. Default: 40.
    pub part_label_width_pt: u32,
    /// Blank margin in points on every side of the page: the space between
    /// the paper's edge and the header, music and page number. Default: 25.
    pub page_margin_pt: u32,
    /// Title text style. `font_size` default: 150% of `row_height`.
    pub title_style: TextStyle,
    /// Subtitle text style. `font_size` default: 80% of `row_height`.
    pub subtitle_style: TextStyle,
    /// Author text style. `font_size` default: 60% of `row_height`.
    pub author_style: TextStyle,
    /// `sequence` summary line text style. `font_size` default: 12.
    pub sequence: TextStyle,
    /// Part-name legend entry text style (e.g. `V — Vocal (S1,S2,A1,A2)`).
    /// `font_size` default: 60% of `row_height`.
    pub part_legend: TextStyle,
    /// Measure bar-number text style. `font_size` default: 10.
    pub measure_number: TextStyle,
    /// Inline section-label text style (`label="..."` on a measure's directive
    /// line). `font_size` default: 12.
    pub section_label: TextStyle,
    /// Part row-label text style (the instrument name shown at the start of
    /// each system row, e.g. "Soprano"). `font_size` default: 12. See
    /// `part_label_width_pt` for the column's reserved width.
    pub part_label: TextStyle,
    /// Page-number footer text style. `font_size` default: 60% of `row_height`.
    pub page_number: TextStyle,
    /// Running-title text style: the small title (no subtitle/author) shown on
    /// pages after the first. `font_size` default: 60% of `row_height`; italic,
    /// serif.
    pub running_title: TextStyle,
    /// Lyric syllable text style. `font_size` default: 60% of `row_height`.
    /// `horizontal_padding_pt` default: 4. `vertical_padding_pt` (formerly
    /// `lyric_click_target_padding_pt`) is extra padding added around a lyric
    /// syllable's click-target box on top of the font's own measured
    /// ascender+descender span (see
    /// `grid_layout::layout_heights::lyric_row_height`); default: 12.
    pub lyrics: TextStyle,
    /// Note head/rest/percussion-hit/tuplet-bracket text style. `font_size`
    /// default: `lyrics.font_size`. `horizontal_padding_pt` widens the note
    /// column's spacing rod (see `grid_layout::layout_spacing::column_rod`);
    /// also used for the multi-measure-rest bar's end insets and the
    /// tie/slur/underline/tuplet-bracket span anchors, all of which key off a
    /// note column. Default: 4.
    pub notes: TextStyle,
    /// Chord symbol text style. `font_size` default: `lyrics.font_size`.
    /// `horizontal_padding_pt` default: 4.
    pub chords: TextStyle,
    /// Note-dash (sustain-beat `-` extension) text style. `font_size` default:
    /// `notes.font_size`; scales the rendered dash's width. `horizontal_padding_pt`
    /// default: 4.
    pub note_dash: TextStyle,
    /// Score-wide default for `merge_duplicate_measures_across_parts=`: when `false`,
    /// identical measure rows from different parts are no longer merged into one row
    /// (see `consolidator::consolidate`). Default: `true`. A `merge_duplicate_measures_across_parts=`
    /// directive line can override this from a given measure onward (see
    /// `MultiPartMeasure::merge_duplicate_measures_across_parts`).
    pub merge_duplicate_measures_across_parts: bool,
    /// Score-wide default for `hide_resting_parts=`: when `false`, an all-rest part is
    /// no longer omitted from a measure that has other parts with real content (see
    /// `compiler::compile_measure`). Default: `true`. A `hide_resting_parts=` directive
    /// line can override this from a given measure onward (see
    /// `MultiPartMeasure::hide_resting_parts`).
    pub hide_resting_parts: bool,
    /// When `true`, the horizontal divider line drawn between systems is omitted (see
    /// `grid_layout::layout`). Default: `false`.
    pub hide_system_dividers: bool,
    /// Translation in points applied to every rendered directive row (bar number, section
    /// label, key, bpm, time signature, nav markers), after layout (see
    /// `renderer::new_renderer::render_directive_line`). Not applied to the `sequence`
    /// summary header. Default: `(0, 0)`.
    pub directive_row_offset: Offset,
}

#[derive(Clone)]
pub struct Notes {
    pub events: Vec<NoteEvent>,
}

/// What a lyric part carries for one measure instead of notes of its own.
///
/// The part's target is named rather than indexed so the link survives
/// filtering parts by name, and `target_events` is a copy of the target's
/// events for this measure so the lyric row lays out against the target's
/// columns even when the target itself is filtered out.
#[derive(Clone)]
pub struct LyricsSlice {
    /// Abbreviation of the part whose notes these lyrics sing along to.
    pub target_name: String,
    /// The target's events this measure, which fix where syllables sit.
    pub target_events: Vec<NoteEvent>,
    /// The target is a `chords` part: syllables are then placed by
    /// `chord_lyric_placement` instead of one per note slot.
    pub targets_chords: bool,
    /// Tie-aware syllables, one per lyric slot of `target_events` (see
    /// `lyric_slots::measure_lyric_slots`); empty for no lyrics (`_`). Under a
    /// chord target there are no such slots: these are the line's syllables
    /// whole, in order, bracketed ones (`Syllable::bracketed`) tallied 1:1 with
    /// the target's `lyric_slots::chord_attacks` by the grouper.
    pub syllables: Vec<Syllable>,
}

#[derive(Clone)]
pub struct PartSlice {
    pub name: Option<String>,
    pub kind: crate::ast::parsed::PartKind,
    pub soundfont: crate::ast::parsed::Soundfont,
    pub volume: u8,
    pub octave_offset: i8,
    /// The instrument the part's `parts` line names, if any (see
    /// `RangeInstrument`).
    pub range_instrument: Option<crate::ast::parsed::RangeInstrument>,
    pub notes: Notes,
    /// Set exactly for a lyric part (`kind` is `PartKind::Lyrics`), whose own
    /// `notes` are empty.
    pub lyrics: Option<LyricsSlice>,
    /// True when this slice's source measure had at least one `Diagnostic::Error`.
    /// The compiler uses this to drop incoming cross-measure tie/slur arcs.
    pub has_error: bool,
    /// Copied from the source `GroupedMeasure::resolution_multiplier`: the factor
    /// every duration in `notes` was multiplied by during tuplet rescaling. `1`
    /// when the measure has no tuplets (the common case, a no-op). The compiler
    /// (`compiler::part_slice`) scales its column/underline arithmetic by this
    /// value so a rescaled measure still lays out correctly — see **Tuplet** in
    /// `ARCHITECTURE.md`.
    pub resolution_multiplier: u32,
    /// Copied from the source `GroupedMeasure::beat_group_size`: the quarter-beat
    /// width of one beam group under this measure's time signature (`4` for simple
    /// meters, `6` for compound meters like 6/8, 9/8, 12/8). The compiler scales
    /// this by `resolution_multiplier` before comparing against it, same as the
    /// other quarter-beat-grid constants.
    pub beat_group_size: u32,
}

#[derive(Clone)]
pub struct MultiPartMeasure {
    pub time_signature: Option<TimeSignature>,
    pub bpm: Option<u32>,
    pub key: Option<KeyChange>,
    pub label: Option<String>,
    /// Resolved value of `merge_duplicate_measures_across_parts=` in effect on this
    /// measure — either an override starting here or carried forward from an earlier
    /// measure or the `metadata` default (see `consolidator::consolidate`).
    pub merge_duplicate_measures_across_parts: bool,
    /// Resolved value of `hide_resting_parts=` in effect on this measure — either an
    /// override starting here or carried forward from an earlier measure or the
    /// `metadata` default (see `compiler::compile_measure`).
    pub hide_resting_parts: bool,
    /// `break` directive on this measure's directive line: forces a new system
    /// to start at this measure (see `grid_layout::layout_systems::pack_into_systems`).
    /// Applies only to this measure; does not persist to later ones.
    pub system_break: bool,
    pub parts: Vec<PartRow>,
    /// Byte range of this measure's note events in the original source.
    /// Used to map editor cursor position to a measure index.
    pub source_span: Span,
    /// Diagnostics collected during grouping for this measure.
    /// Non-empty triggers a colored overlay in the SVG renderer.
    pub diagnostics: Vec<Diagnostic>,
}

#[derive(Clone)]
pub enum PartRow {
    Timed(PartSlice),
}

impl PartSlice {
    /// The events that fix this slice's timing: its own notes, or for a lyric
    /// part (which has none) its target's.
    pub fn timing_events(&self) -> &[NoteEvent] {
        self.lyrics
            .as_ref()
            .map_or(self.notes.events.as_slice(), |lyrics| {
                lyrics.target_events.as_slice()
            })
    }
}

impl PartRow {
    pub fn name(&self) -> Option<&String> {
        self.slice().name.as_ref()
    }

    pub fn slice(&self) -> &PartSlice {
        match self {
            PartRow::Timed(s) => s,
        }
    }

    pub fn slice_mut(&mut self) -> &mut PartSlice {
        match self {
            PartRow::Timed(s) => s,
        }
    }
}

pub(crate) enum GroupedTrack {
    Timed(GroupedPart),
}

impl GroupedTrack {
    pub(crate) fn measure_count(&self) -> usize {
        match self {
            GroupedTrack::Timed(part) => part.measures.len(),
        }
    }

    pub(crate) fn track_name(&self) -> &Option<String> {
        match self {
            GroupedTrack::Timed(part) => &part.name,
        }
    }
}

#[derive(Clone)]
pub struct Score {
    pub metadata: Metadata,
    pub measures: Vec<MultiPartMeasure>,
    /// Document-level diagnostics (e.g. metadata parse errors), not tied to any measure.
    pub document_diagnostics: Vec<Diagnostic>,
    /// Resolved playback order from a `sequence` section, if present and
    /// valid: each span is a labeled section's inclusive measure-index range
    /// in `measures`, in the order they should play.
    pub sequence: Option<Vec<SequenceSpan>>,
}

/// A labeled section's resolved measure range, in written-score order: from
/// the labeled measure up to (but not including) the next labeled measure,
/// or the end of the score.
#[derive(Clone)]
pub struct SequenceSpan {
    pub label: String,
    /// Inclusive index into `Score.measures`.
    pub start: usize,
    /// Inclusive index into `Score.measures`.
    pub end: usize,
    /// Individual part abbreviations to omit from this occurrence's MIDI/WAV
    /// playback (from a `sequence` entry's `(-abbrev ...)` omit suffix, or
    /// the complement of an `(abbrev ...)` only suffix against `parts`).
    pub omit_parts: Vec<String>,
    /// The entry's `(-abbrev ...)` / `(abbrev ...)` suffix's abbreviations as
    /// written, plus whether it's an omit or only suffix, for display on the
    /// SVG/PDF "Sequence: ..." summary line. `None` when the entry has no
    /// suffix.
    pub part_filter_display: Option<PartFilterDisplay>,
}

/// A `sequence` entry's `(...)` suffix, as written, for display purposes.
#[derive(Debug, Clone, PartialEq)]
pub struct PartFilterDisplay {
    pub kind: crate::parser::sequence_parser::PartFilterKind,
    pub parts: Vec<String>,
}

// ── Intermediate grouper types (not part of the public API) ─────────────────

pub(crate) struct MeasureDirectives {
    pub(crate) time_signature: Option<TimeSignature>,
    pub(crate) bpm: Option<u32>,
    pub(crate) key: Option<KeyChange>,
    pub(crate) label: Option<String>,
    pub(crate) merge_duplicate_measures_across_parts: bool,
    pub(crate) hide_resting_parts: bool,
    pub(crate) system_break: bool,
}

pub(crate) struct GroupedScore {
    pub(crate) measure_directives: Vec<MeasureDirectives>,
    pub(crate) parts: Vec<GroupedTrack>,
    pub(crate) per_measure_parse_errors: Vec<Option<Diagnostic>>,
}

pub(crate) struct GroupedMeasure {
    pub(crate) notes: Notes,
    pub(crate) source_span: Span,
    /// Tie-aware syllables paired to the target's lyric slots in this measure.
    /// Set only for a lyric part, during grouping.
    pub(crate) paired_lyrics: Vec<Syllable>,
    /// Recoverable lyrics underflow/overflow for this measure, if any.
    pub(crate) lyrics_error: Vec<Warning>,
    /// Recoverable beat overflow for this measure (notes trimmed), if any.
    pub(crate) beat_overflow_error: Option<Warning>,
    /// Grouping diagnostics: dotted-eighth RecoverableErrors and half-bar-boundary Warnings.
    pub(crate) dotted_eighth_errors: Vec<Diagnostic>,
    /// Chord parse diagnostics: promoted kinds are Error, others are Warning.
    pub(crate) chord_errors: Vec<Diagnostic>,
    /// Recoverable lex error from an unexpected character on the notes line, if any.
    pub(crate) lex_error: Option<RecoverableError>,
    /// Recoverable error from a malformed lyrics line (e.g. empty lyrics line), if any.
    pub(crate) lyrics_parse_error: Option<RecoverableError>,
    /// Recoverable error from `-` at the start of a measure with no preceding event, if any.
    pub(crate) extension_no_preceding_event_error: Option<RecoverableError>,
    /// Factor by which every duration in this measure's events was multiplied by
    /// the tuplet-rescale pass so that tuplet ratios (e.g. 3-in-2) resolve to
    /// whole numbers. `1` when the measure has no tuplets (the common case).
    pub(crate) resolution_multiplier: u32,
    /// Quarter-beat width of one beam group under the time signature in effect
    /// for this measure — `4` (one quarter note) for simple meters, `6` (one
    /// dotted quarter) for compound meters like 6/8, 9/8, 12/8. Unscaled by
    /// `resolution_multiplier`; the compiler applies that scaling itself, the
    /// same way it does for the `4`/`6` constants used elsewhere.
    pub(crate) beat_group_size: u32,
}

pub(crate) struct GroupedPart {
    pub(crate) name: Option<String>,
    pub(crate) kind: crate::ast::parsed::PartKind,
    pub(crate) soundfont: crate::ast::parsed::Soundfont,
    pub(crate) volume: u8,
    pub(crate) octave_offset: i8,
    pub(crate) range_instrument: Option<crate::ast::parsed::RangeInstrument>,
    pub(crate) measures: Vec<GroupedMeasure>,
}

// ── Shared note types ─────────────────────────────────────────────────────────

#[path = "grouped_notes.rs"]
mod grouped_notes;
pub use grouped_notes::{
    GroupedChordNote, GroupedNote, GroupedPercussionHit, GroupedRest, NoteEvent, TimeSignature,
};
