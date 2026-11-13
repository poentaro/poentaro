use itertools::Itertools;

use super::DocumentSection;
use crate::parser::parts_parser::{PartSettingRange, PART_SETTING_LIMITS};

/// Identifies the specific kind of recoverable error for programmatic matching.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RecoverableErrorKind {
    /// `measure_directives` is shorter than the measure count (internal invariant).
    MeasureDirectivesMissing,
    /// `source_span` is absent for the given measure index (internal invariant).
    SourceSpanMissing { index: usize },
    /// A timed-part measure is missing at the given index (internal invariant).
    TimedPartMeasureMissing,
    /// Generic recoverable error with a free-form message.
    General { message: String },
    /// Text that doesn't read as an anyx document: a syntax error, or an
    /// expression of the wrong shape. That expression is left out.
    AnyxInvalid { message: String },
    /// A note has both octave-up (`'`) and octave-down (`,`) markers — octave shift is zeroed.
    DurationMixedOctaveMarkers,
    /// An unexpected character was encountered while lexing a notes line — the line is dropped.
    LexUnexpectedChar { ch: char },
    /// Measure group has no data lines at all.
    MeasureNoDataLines,
    /// A dotted eighth note or rest is not followed by a sixteenth — rhythmic structure is broken.
    DottedEighthNeedsSixteenth,
    /// A chord symbol did not start with a degree digit (0–7) — chord is dropped.
    ChordExpectedDegreeDigit { ch: char },
    /// A chord token is entirely invalid — chord is dropped.
    ChordInvalidToken { message: String },
    /// An unexpected character in a note duration suffix — note is emitted with default duration.
    DurationUnexpectedChar { ch: char },
    /// A parts abbreviation is used by more than one declaration — the duplicate is skipped.
    PartsDuplicateAbbreviation { abbrev: String },
    /// The `parts` section contains no valid declarations — document renders empty.
    PartsEmptySection,
    /// A display name before `(` is empty — the declaration is skipped.
    PartsEmptyDisplayName,
    /// The abbreviation inside `()` is empty — the declaration is skipped.
    PartsEmptyAbbreviation,
    /// No `()` and the track name is empty — the declaration is skipped.
    PartsEmptyTrackName,
    /// The RHS of a parts declaration is not a recognized column spec — the declaration is skipped.
    PartsInvalidColumns { rhs: String },
    /// A required section is absent — an empty default is used.
    SectionMissing { section: DocumentSection },
    /// Two `preset` sections share a name — the later one is skipped.
    PresetDuplicateName { name: String },
    /// A `preset` is named `"Default"`, which the preset menu uses for "no preset" — the section is skipped.
    PresetReservedName { name: String },
    /// A preset's `visible_parts` or `soloed_parts` names an abbreviation not declared in `parts` — the entry is ignored.
    PresetUnknownPart { abbreviation: String },
    /// A lyrics line is empty — treated as `_` (no lyrics for this measure).
    LyricsLineEmpty,
    /// A lyrics slot has no paired notes track — lyrics are skipped.
    LyricsNoNotesTrack { abbrev: String },
    /// A bracketed lyric item holds zero or several syllables instead of exactly one.
    LyricsBracketNotOneSyllable { found: usize },
    /// `字(…)` whose parentheses don't hold one pinyin syllable.
    LyricsPinyinInvalid { text: String },
    /// A lyric bracket `(` / `（` has no closing bracket.
    LyricsBracketUnclosed,
    /// A closing lyric bracket `)` / `）` has no opening bracket.
    LyricsBracketUnmatched,
    /// A chord part's lyric line pairs more items with chord attacks than the measure has:
    /// its bracketed items, plus its unbracketed first syllable when
    /// `first_syllable_takes_beat_one` (the first chord attack is on beat 1).
    LyricsBracketCountMismatch {
        brackets: usize,
        attacks: usize,
        first_syllable_takes_beat_one: bool,
    },
    /// A part one lyric part is sung on doesn't take its syllables at the
    /// times the part they're paired with (`paired`) does.
    LyricsTargetsDiffer { paired: String, other: String },
    /// A part has a different number of measures than the first part.
    PartMeasureCountMismatch {
        part: String,
        got: usize,
        expected: usize,
    },
    /// `-` used to extend a note/chord but there is no preceding event — the `-` is ignored.
    ExtensionNoPrecedingEvent { chord_track: bool },
    /// A notes token did not start with a pitch digit (0-7) — the token is skipped.
    NoteExpectedPitchDigit { ch: char },
    /// A percussion token did not start with `x` (hit) or `0` (rest) — the token is skipped.
    PercussionExpectedHitOrRest { ch: char },
    /// A dot was applied to a quarter-beat (`=`) note — dot is ignored, duration stays 1.
    DurationCannotDotQuarterBeat,
    /// A second dot (`..`) was applied but the base duration doesn't divide evenly into
    /// quarter-beats — the second dot is dropped, the first dot (if any) is kept.
    DurationCannotDoubleDot,
    /// A `)` appeared with no matching `(` — the `)` is ignored.
    GroupUnexpectedCloseParen,
    /// A `(` was not closed before the end of the score — group treated as open.
    UnclosedGroupAtEnd { part: String },
    /// A `[Key]` prefix does not match any declared part abbreviation — the line is dropped.
    PartKeyUnknown { key: String },
    /// A score data line has no `[Abbrev]` prefix — the line is dropped.
    ScoreLineMissingKeyPrefix,
    /// `~` (tie-to-next) appeared on a rest — `~` is ignored, rest is emitted normally.
    TieOnRest,
    /// `~` appears on the last note of a part — there is no following note to tie to.
    DanglingTie,
    /// `~`-marked note is followed by a note with a different pitch or octave — tie is dropped.
    TiePitchMismatch { expected: String, got: String },
    /// A `follow[Target]` clause names an abbreviation not in the parts declaration — the clause is ignored.
    PartsFollowUnknownTarget { target: String },
    /// The first declared part uses `follow[...]`, which is not allowed.
    PartsFirstPartCannotFollow,
    /// A `lyrics[Target]` clause does not name an earlier-declared notes or chords part — the lyrics part is dropped.
    PartsLyricsInvalidTarget { target: String },
    /// Soundfont string does not match any known instrument — declaration is kept with the parsed
    /// MIDI program number; audio playback may differ from what the user intended.
    PartsUnknownSoundfont {
        soundfont: String,
        suggestions: Vec<String>,
    },
    /// Per-part volume outside `PART_SETTING_LIMITS.volume` — clamped to the valid range.
    PartsVolumeOutOfRange { volume: u16 },
    /// Per-part octave offset outside `PART_SETTING_LIMITS.octave_offset` — clamped to the valid range.
    PartsOctaveOffsetOutOfRange { offset: i16 },
    /// A bare word on a `parts` line that isn't a known instrument — ignored.
    PartsUnknownInstrument { name: String },
    /// An instrument on a part that isn't `notes` or `follow[X]` — ignored.
    PartsInstrumentOnNonNotesPart,
    /// A voice on a lyric part under a `chords` part, whose syllables have no
    /// A part that sings (its sound is a Chimera voice) with no lyric part
    /// sung on it.
    PartsSingingPartWithoutLyrics { part: String },
    /// A part that sings with more than one lyric part sung on it.
    PartsSingingPartWithSeveralLyrics {
        part: String,
        lyric_parts: Vec<String>,
    },
    /// A `range` option that isn't two pitches joined by `to` — ignored.
    PartsMalformedRange,
    /// A `range` bound that isn't a pitch like `C3` — the range is ignored.
    PartsMalformedRangePitch { pitch: String },
    /// A `range` whose low bound is above its high bound — ignored.
    PartsRangeLowAboveHigh { lowest: String, highest: String },
    /// A `range` on a part that isn't `notes` or `follow[X]` — ignored.
    PartsRangeOnNonNotesPart,
    /// `r`/bare `_`/`=` used with no prior pitched note/chord to repeat — token ignored.
    RepeatNoPriorNote,
    /// A `{N:...}` tuplet has no standard implied ratio and no explicit `{N:M:...}` override.
    TupletAmbiguousRatio { num: u32 },
    /// A tuplet's note count at `}` doesn't match its declared `N`.
    TupletNoteCountMismatch { expected: u32, got: usize },
}

impl RecoverableErrorKind {
    pub fn message(&self) -> String {
        match self {
            Self::MeasureDirectivesMissing => "internal invariant: measure_directives shorter than measure count".to_string(),
            Self::SourceSpanMissing { index } => format!("internal invariant: source_span missing for measure {index}"),
            Self::TimedPartMeasureMissing => "internal invariant: timed part measure missing".to_string(),
            Self::General { message } | Self::AnyxInvalid { message } => message.clone(),
            Self::LexUnexpectedChar { ch } => format!("unexpected character: {ch}"),
            Self::MeasureNoDataLines => "measure has no data lines; treating all parts as empty".to_string(),
            Self::DottedEighthNeedsSixteenth => "dotted eighth must be followed by a sixteenth note or rest".to_string(),
            Self::ChordExpectedDegreeDigit { ch } => format!("expected chord degree digit (0-7), got: {ch}"),
            Self::ChordInvalidToken { message } => message.clone(),
            Self::DurationUnexpectedChar { ch } => format!("unexpected character in note duration: {ch}"),
            Self::PartsDuplicateAbbreviation { abbrev } => format!("duplicate abbreviation: {abbrev}"),
            Self::PartsEmptySection => "expected at least one part in parts [...]".to_string(),
            Self::PartsEmptyDisplayName => "display name cannot be empty".to_string(),
            Self::PartsEmptyAbbreviation => "abbreviation cannot be empty".to_string(),
            Self::PartsEmptyTrackName => "track name cannot be empty".to_string(),
            Self::PartsInvalidColumns { rhs } => format!(
                "invalid track columns '{rhs}': expected 'chords', 'notes', or 'follow[X]'"
            ),
            Self::SectionMissing { section } => format!("missing {}", section.header()),
            Self::PresetDuplicateName { name } => format!("duplicate preset \"{name}\""),
            Self::PresetReservedName { name } => format!("\"{name}\" is reserved for the song without a preset; choose another name"),
            Self::PresetUnknownPart { abbreviation } => format!("no part with abbreviation '{abbreviation}' in parts [ ... ]"),
            Self::LyricsLineEmpty => "lyrics line cannot be empty; use '_' for no lyrics".to_string(),
            Self::LyricsNoNotesTrack { abbrev } => format!("lyrics line for '{abbrev}' has no matching notes track"),
            Self::LyricsBracketNotOneSyllable { found } => format!("a lyric bracket must hold exactly one syllable, found {found}"),
            Self::LyricsPinyinInvalid { text } => format!("`{text}` is not a pinyin syllable"),
            Self::LyricsTargetsDiffer { paired, other } => format!("[{other}] doesn't take its syllables at the same times as [{paired}] in this measure; the parts a lyric part is sung on must share their rhythm"),
            Self::LyricsBracketUnclosed => "lyric bracket is never closed".to_string(),
            Self::LyricsBracketUnmatched => "closing lyric bracket has no opening bracket".to_string(),
            Self::LyricsBracketCountMismatch { brackets, attacks, first_syllable_takes_beat_one } => format!(
                "lyrics under a chord part need at most one bracketed item per chord attack, found {brackets} bracketed item{} for {attacks} chord attack{}{}",
                if *brackets == 1 { "" } else { "s" },
                if *attacks == 1 { "" } else { "s" },
                if *first_syllable_takes_beat_one { "; the unbracketed first syllable already takes the beat-1 chord" } else { "" },
            ),
            Self::PartMeasureCountMismatch { part, got, expected } => format!("part {part:?} has {got} measures but the first part has {expected}; all parts must have the same number of measures"),
            Self::ExtensionNoPrecedingEvent { chord_track: true } => "chord extension '-' with no preceding event; '-' ignored".to_string(),
            Self::ExtensionNoPrecedingEvent { chord_track: false } => "extension '-' without a preceding note or rest; '-' ignored".to_string(),
            Self::NoteExpectedPitchDigit { ch } => format!("expected pitch digit (0-7), got: {ch}"),
            Self::PercussionExpectedHitOrRest { ch } => format!("expected 'x' (hit) or '0' (rest), got: {ch}"),
            Self::DurationMixedOctaveMarkers => "mixed octave markers: use ' for up or , for down, not both; octave shift ignored".to_string(),
            Self::DurationCannotDotQuarterBeat => "cannot dot a quarter-beat (=) note; dot ignored, duration stays at 1 beat".to_string(),
            Self::DurationCannotDoubleDot => "cannot apply a second dot here; the note's duration doesn't divide evenly into quarter-beats; second dot ignored".to_string(),
            Self::GroupUnexpectedCloseParen => "unexpected `)` — no open group; `)` ignored".to_string(),
            Self::UnclosedGroupAtEnd { part } => format!("unclosed '(' group at end of score in part '{part}'"),
            Self::PartKeyUnknown { key } => format!("`[{key}]` does not match any declared part abbreviation; line dropped"),
            Self::ScoreLineMissingKeyPrefix => "score line has no [Abbrev] prefix; line dropped".to_string(),
            Self::TieOnRest => "~ cannot be applied to a rest; ~ ignored".to_string(),
            Self::DanglingTie => "~ has no following note to tie to; ~ ignored".to_string(),
            Self::TiePitchMismatch { expected, got } => format!("tied notes must have the same pitch and octave; expected {expected}, got {got}; ~ ignored"),
            Self::PartsFollowUnknownTarget { target } => format!("no part with abbreviation '{target}'; follow targets must match a part declared earlier in parts"),
            Self::PartsLyricsInvalidTarget { target } => format!("lyrics [{target}] must name a notes or chords part declared earlier in parts; a lyric part sung on several parts names notes parts only"),
            Self::PartsFirstPartCannotFollow => "the first declared part cannot use follow[...]".to_string(),
            Self::PartsUnknownSoundfont { soundfont, suggestions } => Self::parts_unknown_soundfont_message(soundfont, suggestions),
            Self::PartsVolumeOutOfRange { volume } => Self::parts_volume_out_of_range_message(*volume),
            Self::PartsOctaveOffsetOutOfRange { offset } => Self::parts_octave_offset_out_of_range_message(*offset),
            Self::PartsUnknownInstrument { name } => format!(
                "unknown instrument \"{name}\"; expected one of: {}",
                crate::ast::parsed::RangeInstrument::ALL.iter().map(|instrument| instrument.keyword()).join(", ")
            ),
            Self::PartsInstrumentOnNonNotesPart => "only notes parts can name an instrument; ignored".to_string(),
            Self::PartsSingingPartWithoutLyrics { part } => format!("part [{part}] sings, but no lyric part is sung on it; add one, like `\"Words\" [w] : lyrics [{part}]`, or give it a General MIDI sound, like `sound : 52`"),
            Self::PartsSingingPartWithSeveralLyrics { part, lyric_parts } => format!("part [{part}] sings, but {} lyric parts are sung on it ({}); keep one, and write each verse as another line of it", lyric_parts.len(), lyric_parts.iter().map(|lyric_part| format!("[{lyric_part}]")).join(", ")),
            Self::PartsMalformedRange => "a range is two pitches joined by `to`, like `range : C3 to G4`; ignored".to_string(),
            Self::PartsMalformedRangePitch { pitch } => format!("`{pitch}` is not a pitch; write a letter, an optional `b` for flat and an octave, like `C3` or `Bb4`, or quote a sharp, like `\"F#3\"`; range ignored"),
            Self::PartsRangeLowAboveHigh { lowest, highest } => format!("the range's low note {lowest} is above its high note {highest}; range ignored"),
            Self::PartsRangeOnNonNotesPart => "only notes parts can have a range; ignored".to_string(),
            Self::RepeatNoPriorNote => "no prior note/chord to repeat; token ignored".to_string(),
            Self::TupletAmbiguousRatio { num } => format!("tuplet ratio for {num} is ambiguous; use {{{num}:M:...}} to specify explicitly"),
            Self::TupletNoteCountMismatch { expected, got } => format!("tuplet declared {expected} notes but got {got}; note count must match"),
        }
    }

    fn parts_volume_out_of_range_message(volume: u16) -> String {
        let PartSettingRange { min, max } = PART_SETTING_LIMITS.volume;
        let clamped = PART_SETTING_LIMITS.volume.clamp(i32::from(volume)).value;
        format!(
            "volume {volume}% is out of range; valid range is {min}% to {max}%; clamped to {clamped}%"
        )
    }

    fn parts_octave_offset_out_of_range_message(offset: i16) -> String {
        let PartSettingRange { min, max } = PART_SETTING_LIMITS.octave_offset;
        let clamped = PART_SETTING_LIMITS
            .octave_offset
            .clamp(i32::from(offset))
            .value;
        format!(
            "octave offset {offset:+} is out of range; valid range is {min:+} to {max:+}; clamped to {clamped:+}"
        )
    }

    fn parts_unknown_soundfont_message(soundfont: &str, suggestions: &[String]) -> String {
        if suggestions.is_empty() {
            format!("unknown soundfont \"{soundfont}\"")
        } else {
            format!(
                "unknown soundfont \"{soundfont}\". Did you mean:\n  {}",
                suggestions.join("\n  ")
            )
        }
    }
}
