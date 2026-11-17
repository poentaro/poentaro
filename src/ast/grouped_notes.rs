use crate::ast::parsed::{
    Accidental, BassDegree, Extension, NumberedPitch, TriadQuality, TupletInfo,
};
use crate::error::Span;

#[derive(Clone, PartialEq)]
pub struct TimeSignature {
    pub numerator: u8,
    pub denominator: u8,
}

#[derive(Clone)]
pub enum NoteEvent {
    Note(GroupedNote),
    Rest(GroupedRest),
    Chord(GroupedChordNote),
    Percussion(GroupedPercussionHit),
}

impl NoteEvent {
    /// Duration in quarter-beats (including `-` extensions).
    pub fn duration(&self) -> u32 {
        match self {
            Self::Note(note) => note.duration,
            Self::Rest(rest) => rest.duration,
            Self::Chord(chord) => chord.duration,
            Self::Percussion(hit) => hit.duration,
        }
    }
}

#[derive(Clone)]
pub struct GroupedChordNote {
    pub degree: NumberedPitch,
    pub accidental: Accidental,
    pub triad: TriadQuality,
    pub extension: Option<Extension>,
    pub bass: Option<BassDegree>,
    pub duration: u32,
    pub slur: bool,
    pub tie_to_next_span: Option<Span>,
    pub event_span: Span,
    pub group_membership: u8,
    pub group_continuation: u8,
    pub dotted: bool,
    pub double_dotted: bool,
    pub slur_group_close_at_duration: Option<u32>,
    /// The innermost `{...}` tuplet bracket this chord note belonged to before
    /// rescaling, if any.
    pub tuplet: Option<TupletInfo>,
}

#[derive(Clone)]
pub struct GroupedNote {
    pub pitch: NumberedPitch,
    pub accidental: Accidental,
    pub octave: i8,
    /// Duration in quarter-beats, including any beats added by `-` extensions.
    pub duration: u32,
    /// True if this note is tied/slurred to the next note.
    pub slur: bool,
    /// Source span of the `~` suffix when this note is tied to the next note.
    pub tie_to_next_span: Option<Span>,
    /// Byte range of this note token in the original source.
    pub event_span: Span,
    /// Number of nested `(…)` groups this note belongs to.
    pub group_membership: u8,
    /// Number of those groups that continue past this note.
    pub group_continuation: u8,
    /// True if this note was written with `*` (dotted duration).
    pub dotted: bool,
    /// True if this note was written with a second dot (double-dotted duration).
    /// Only ever `true` when `dotted` is also `true`.
    pub double_dotted: bool,
    pub slur_group_close_at_duration: Option<u32>,
    /// The innermost `{...}` tuplet bracket this note belonged to before rescaling,
    /// if any.
    pub tuplet: Option<TupletInfo>,
}

impl GroupedNote {
    pub fn tie_to_next(&self) -> bool {
        self.tie_to_next_span.is_some()
    }
}

#[derive(Clone)]
pub struct GroupedPercussionHit {
    /// Duration in quarter-beats, including any beats added by `-` extensions.
    pub duration: u32,
    /// True if this hit is tied/slurred to the next hit.
    pub slur: bool,
    /// Source span of the `~` suffix when this hit is tied to the next hit.
    pub tie_to_next_span: Option<Span>,
    /// Byte range of this hit token in the original source.
    pub event_span: Span,
    /// Number of nested `(…)` groups this hit belongs to.
    pub group_membership: u8,
    /// Number of those groups that continue past this hit.
    pub group_continuation: u8,
    /// True if this hit was written with `*` (dotted duration).
    pub dotted: bool,
    /// True if this hit was written with a second dot (double-dotted duration).
    pub double_dotted: bool,
    pub slur_group_close_at_duration: Option<u32>,
    /// The innermost `{...}` tuplet bracket this hit belonged to before rescaling,
    /// if any.
    pub tuplet: Option<TupletInfo>,
}

impl GroupedPercussionHit {
    pub fn tie_to_next(&self) -> bool {
        self.tie_to_next_span.is_some()
    }
}

impl GroupedChordNote {
    pub fn tie_to_next(&self) -> bool {
        self.tie_to_next_span.is_some()
    }
}

#[derive(Clone)]
pub struct GroupedRest {
    /// Duration in quarter-beats, including any beats added by `-` extensions.
    pub duration: u32,
    /// True if this rest was written with `*` (dotted duration).
    pub dotted: bool,
    /// True if this rest was written with a second dot (double-dotted duration).
    pub double_dotted: bool,
    /// Byte range of this rest token in the original source.
    pub event_span: Span,
    pub group_membership: u8,
    pub group_continuation: u8,
    /// The innermost `{...}` tuplet bracket this rest belonged to before rescaling,
    /// if any.
    pub tuplet: Option<TupletInfo>,
    /// Copied from `ParsedRest::implicit_fill` — true when this rest fills a
    /// part not mentioned in this measure, rather than being written by the
    /// composer. See `ParsedRest::implicit_fill`.
    pub implicit_fill: bool,
}
