use crate::error::{Diagnostic, RecoverableError, Span, Spanned, Warning};

mod events;
mod metadata_access;
mod metadata_keys;
mod pitch;
mod preset;
mod range_instrument;
pub use events::*;
pub use metadata_keys::*;
pub use pitch::*;
pub use preset::*;
pub use range_instrument::*;

#[derive(Debug)]
pub enum ParsedMeasureSlot {
    EmptyNote { span: Span },
    Real { events: Vec<Spanned<ScoreEvent>> },
}

#[derive(Debug)]
pub struct ParsedLyrics {
    /// Measure -> syllables, in score order, for one lyric part. An empty inner
    /// vec = `_` (no lyrics) for that measure.
    pub measure_syllables: Vec<Vec<Syllable>>,
    /// Byte offset of the start of the lyric line for each measure, in order.
    pub measure_starts: Vec<usize>,
    /// Byte offset of the end of the lyric line for each measure, in order.
    /// Used to extend the measure's source span to cover the lyric line.
    pub measure_ends: Vec<usize>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Soundfont(pub u8);

impl Soundfont {
    pub const ACOUSTIC_GRAND_PIANO: Self = Self(0);
    pub const CHOIR_AAHS: Self = Self(52);
    /// A GM percussion key (not a program), for `percussion` parts.
    pub const ACOUSTIC_SNARE: Self = Self(38);
}

#[derive(Debug, Clone, PartialEq)]
pub struct PartDecl {
    pub abbreviation: String,
    /// Byte span of the abbreviation token on its `parts` declaration line,
    /// used by rename-symbol to locate the declaration site.
    pub abbreviation_span: Span,
    pub display_name: String,
    pub kind: PartKind,
    pub follow_target: Option<String>,
    pub soundfont: Soundfont,
    pub volume: u8,
    /// MIDI-only octave shift applied to every note in this part (−4..=+4).
    pub octave_offset: i8,
    /// The instrument whose range this part's notes are checked against;
    /// `None` checks nothing. A follow part inherits its target's.
    pub range_instrument: Option<RangeInstrument>,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum PartKind {
    Chords,
    Notes,
    Percussion,
    /// A `lyrics[X]` part: one verse of lyrics sung along to the notes of the
    /// part at `target_part_index` (an index into the declaration list, always
    /// before this part). It has no notes of its own.
    Lyrics {
        target_part_index: usize,
    },
}

impl PartKind {
    /// Whether a part of this kind makes sound. Lyric parts only align to their
    /// target's notes.
    pub fn sounds(self) -> bool {
        !matches!(self, Self::Lyrics { .. })
    }

    /// The soundfont a concrete part of this kind plays with when its
    /// declaration names none.
    pub fn default_soundfont(self) -> Soundfont {
        match self {
            Self::Chords | Self::Notes => Soundfont::ACOUSTIC_GRAND_PIANO,
            Self::Percussion => Soundfont::ACOUSTIC_SNARE,
            Self::Lyrics { .. } => Soundfont::CHOIR_AAHS,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ScoreLineRole {
    Chord,
    Notes,
    Lyrics,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ScoreLineSlot {
    pub track_index: usize,
    pub role: ScoreLineRole,
}

impl PartDecl {
    /// The one score line a part owns in each measure group.
    pub fn score_line_role(&self) -> ScoreLineRole {
        match self.kind {
            PartKind::Chords => ScoreLineRole::Chord,
            PartKind::Notes | PartKind::Percussion => ScoreLineRole::Notes,
            PartKind::Lyrics { .. } => ScoreLineRole::Lyrics,
        }
    }
}

#[derive(Debug)]
pub enum ParsedTrack {
    Timed(ParsedTimedTrack),
}

#[derive(Debug)]
pub struct ParsedTimedTrack {
    pub abbreviation: String,
    pub display_name: String,
    pub kind: PartKind,
    pub soundfont: Soundfont,
    pub volume: u8,
    pub octave_offset: i8,
    pub range_instrument: Option<RangeInstrument>,
    pub measure_slots: Vec<ParsedMeasureSlot>,
    pub lyrics: Option<ParsedLyrics>,
    /// Per-measure beat-overflow error (None = no overflow for that measure).
    pub per_measure_beat_errors: Vec<Option<Warning>>,
    /// Per-measure grouping diagnostics: dotted-eighth errors (RecoverableError) and
    /// half-bar-boundary warnings (Warning), mixed as Diagnostic.
    pub per_measure_dotted_eighth_errors: Vec<Vec<Diagnostic>>,
    /// Per-measure recoverable chord parse diagnostics (empty = no violations for that measure).
    pub per_measure_chord_errors: Vec<Vec<Diagnostic>>,
    /// Per-measure recoverable lex error from an unexpected character on the notes line.
    pub per_measure_lex_errors: Vec<Option<RecoverableError>>,
    /// Per-measure recoverable error on the lyrics line (e.g. empty lyrics line).
    pub per_measure_lyrics_errors: Vec<Option<RecoverableError>>,
    /// Every closed `(…)` tie/slur group on this track, in closing order.
    /// Unclosed groups are omitted.
    pub slur_groups: Vec<ParsedSlurGroup>,
}

/// Source spans of one closed `(…)` tie/slur group's parentheses — lets
/// source edits (e.g. the editor's "Slur/Unslur" toolbar action) locate the
/// group's delimiters without re-scanning the raw text.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ParsedSlurGroup {
    pub open_paren_span: Span,
    pub close_paren_span: Span,
}

/// One `[Abbrev]` key-prefix reference to a part or group abbreviation in the
/// `score` section, with the abbreviation text's own byte span (excluding
/// the surrounding brackets and whitespace). Used by rename-symbol to locate
/// this reference site; kept separate from the wider bracket span used by
/// `RecoverableError::part_key_unknown`, which must keep covering `[Abbrev]`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AbbreviationReference {
    pub abbreviation: String,
    pub span: Span,
}

#[derive(Debug)]
pub struct ParsedDocument {
    pub metadata: ParsedMetadata,
    pub declarations: Vec<PartDecl>,
    pub tracks: Vec<ParsedTrack>,
    pub directive_events_per_measure: Vec<Vec<Spanned<ScoreEvent>>>,
    /// Every `[Abbrev]` key-prefix reference found in the `score` section,
    /// in file order. Used by rename-symbol to find all reference sites.
    pub abbreviation_references: Vec<AbbreviationReference>,
    /// Per-measure recoverable errors from desugaring (e.g. missing lyrics line).
    pub per_measure_parse_errors: Vec<Option<Diagnostic>>,
    /// Recoverable errors from parsing the [metadata] section.
    pub metadata_parse_errors: Vec<RecoverableError>,
    /// Recoverable errors from parsing the [parts] section.
    pub parts_parse_errors: Vec<RecoverableError>,
    /// Recoverable errors from section structure validation (unknown/duplicate/missing sections).
    pub section_structure_errors: Vec<RecoverableError>,
    /// The parsed `sequence` section, if present: an ordered list of
    /// section-label references defining explicit playback order.
    pub sequence: Option<crate::parser::sequence_parser::SequenceSection>,
    /// Recoverable errors from parsing the `sequence` section (e.g. empty entries).
    pub sequence_parse_errors: Vec<RecoverableError>,
    /// Every valid `preset` section, in file order.
    pub presets: Vec<ParsedPreset>,
    /// Errors and warnings from the `preset` sections.
    pub preset_diagnostics: Vec<Diagnostic>,
}

keyword_enum! {
    /// A `font_family` text-style value: which of the three globally-embedded
    /// font roles (see `crate::compositor::types::FontFamily`) a text kind's
    /// glyphs render in. Kept as its own parse-level type rather than reusing
    /// `compositor::types::FontFamily` directly, to preserve the parser/compositor
    /// layering — `resolve_text_style` maps it 1:1 onto the compositor enum.
    pub enum FontFamilyChoice {
        Serif => "serif",
        SansSerif => "sans_serif",
        Monospace => "monospace",
    }
}

/// Parsed (all-optional) form of the three-component style object a single
/// `<kind> = { font_size: N, horizontal_padding_pt: N, vertical_padding_pt: N }`
/// metadata line resolves to. See `crate::ast::grouped::TextStyle` for the
/// fully-resolved (defaulted) counterpart and `syntax.md` for the
/// object-literal grammar.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct TextStyle {
    pub font_size: Option<u32>,
    pub horizontal_padding_pt: Option<u32>,
    pub vertical_padding_pt: Option<u32>,
    pub bold: Option<bool>,
    pub italic: Option<bool>,
    pub underline: Option<bool>,
    /// Which embedded font role (`serif`/`sans_serif`/`monospace`) this kind's
    /// glyphs render in, including `notes`/`chords`/`note_dash` — those three
    /// kinds' glyph widths are re-measured against whichever font this
    /// resolves to (`font_metrics::advance_width_for_family` and friends), so
    /// overriding it here can never desync layout from what actually
    /// renders.
    pub font_family: Option<FontFamilyChoice>,
}

#[derive(Debug, Clone, Default)]
pub struct ParsedMetadata {
    pub title: Option<String>,
    pub subtitle: Option<String>,
    pub author: Option<String>,
    pub row_height: Option<u32>,
    pub max_measures_per_system: Option<u32>,
    pub note_number_width: Option<u32>,
    pub parts_list_columns: Option<u32>,
    /// Fixed width (points) of the part-label column at the start of each
    /// system, shared by every system in the score (see
    /// `Metadata::part_label_width_pt`). A flat scalar field, not part of
    /// `part_label_style`, since it's a layout constant rather than a text
    /// style component.
    pub part_label_width_pt: Option<u32>,
    /// Blank margin (points) on every side of the page (see
    /// `Metadata::page_margin_pt`).
    pub page_margin_pt: Option<u32>,
    /// Title text style.
    pub title_style: TextStyle,
    /// Subtitle text style.
    pub subtitle_style: TextStyle,
    /// Author text style.
    pub author_style: TextStyle,
    /// `sequence` summary line text style.
    pub sequence_style: TextStyle,
    /// Part-name legend entry text style.
    pub part_legend_style: TextStyle,
    /// Measure bar-number text style.
    pub measure_number_style: TextStyle,
    /// Inline section-label text style.
    pub section_label_style: TextStyle,
    /// Page-number footer text style.
    pub page_number_style: TextStyle,
    /// Running-title text style (the small title on pages after the first).
    pub running_title_style: TextStyle,
    /// Part row-label text style (see `Metadata::part_label`).
    pub part_label_style: TextStyle,
    /// Lyric syllable text style, including the click-target's extra vertical
    /// padding (formerly `lyric_click_target_padding_pt`).
    pub lyrics_style: TextStyle,
    /// Note head/rest/percussion-hit text style.
    pub notes_style: TextStyle,
    /// Chord symbol text style.
    pub chords_style: TextStyle,
    /// Note-dash (sustain-beat `-` extension) text style.
    pub note_dash_style: TextStyle,
    /// When `false`, disables merging of identical measure rows that come from different
    /// parts (see `consolidator::consolidate`). Default: `true`.
    pub merge_duplicate_measures_across_parts: Option<bool>,
    /// When `false`, an all-rest part is no longer omitted from a measure that has other
    /// parts with real content (see `compiler::compile_measure`). Default: `true`.
    pub hide_resting_parts: Option<bool>,
    /// When `true`, the horizontal divider line drawn between systems is omitted (see
    /// `grid_layout::layout`). Default: `false`.
    pub hide_system_dividers: Option<bool>,
    /// Translation in points applied to every rendered directive row (bar number, section
    /// label, key, bpm, time signature, nav markers), after layout (see
    /// `renderer::new_renderer::render_directive_line`). Not applied to the `sequence`
    /// summary header. Default: `(0, 0)`.
    pub directive_row_offset: Option<Offset>,
}
