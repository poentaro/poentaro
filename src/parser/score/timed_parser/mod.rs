mod chord_head;
mod depth_event;
mod directives;
mod duration;
mod groups;
mod note_head;
mod percussion_head;
mod timed_lexer;
mod timed_recursive_descent_parser;

#[path = "timed_lexer_tests.rs"]
#[cfg(test)]
mod timed_lexer_tests;

#[path = "timed_recursive_descent_parser_tests.rs"]
#[cfg(test)]
mod timed_recursive_descent_parser_tests;

#[path = "repeat_tests.rs"]
#[cfg(test)]
mod repeat_tests;

#[path = "tuplet_tests.rs"]
#[cfg(test)]
mod tuplet_tests;

pub use timed_lexer::{lex_line, LexContext, TimedLexToken};
pub use timed_recursive_descent_parser::TimedRecursiveDescentParser;

pub use chord_head::ChordHead;
pub use note_head::NoteHead;
pub use percussion_head::PercussionHead;

pub use duration::{parse_duration_suffixes, DurationParse};
pub use groups::{
    apply_closed_group_depth, apply_open_group_depth, implicit_tuplet_ratio,
    validate_group_note_count, GroupFrame, GroupStack, HasGroupDepth, TupletFrame, TupletStack,
};

use crate::ast::parsed::{ScoreEvent, TupletInfo};
use crate::error::{Diagnostic, IrrecoverableError, RecoverableError, Span, Spanned};

/// Error returned by `parse_head`.
///
/// `Recoverable(diagnostic)` means: emit the optional diagnostic and skip this timed unit.
/// `Irrecoverable(error)` means: propagate the error upward.
#[derive(Debug)]
pub enum ParseHeadError {
    Recoverable(Option<Diagnostic>),
    Irrecoverable(IrrecoverableError),
}

impl From<IrrecoverableError> for ParseHeadError {
    fn from(e: IrrecoverableError) -> Self {
        Self::Irrecoverable(e)
    }
}

type ParseHeadResult<H> = Result<(H, usize, bool, Vec<Diagnostic>), ParseHeadError>;

/// Parsed events from one timed notation line, plus any recoverable errors collected while parsing.
pub struct TimedLineParse {
    pub events: Vec<Spanned<ScoreEvent>>,
    pub chord_errors: Vec<Diagnostic>,
    pub lex_errors: Vec<RecoverableError>,
}

/// Parse a single line of timed notation using the lexer + recursive-descent parser.
pub fn parse_timed_line<H: TimedUnitHead>(
    line: &str,
    base_offset: usize,
    stack: &mut GroupStack,
    context: LexContext,
) -> Result<TimedLineParse, IrrecoverableError> {
    let (tokens, lex_errors) = lex_line(line, base_offset, context)?;
    let (events, chord_errors) =
        TimedRecursiveDescentParser::<H>::parse_line(line, base_offset, &tokens, stack)?;
    Ok(TimedLineParse {
        events,
        chord_errors,
        lex_errors,
    })
}

pub trait TimedUnitHead: Sized {
    /// Parse one head starting at `chars[start]`. Returns (head, index after head, is_rest, recoverable warnings).
    ///
    /// On recoverable failure, return `Err(ParseHeadError::Recoverable(diagnostic))`.
    /// On irrecoverable failure, return `Err(ParseHeadError::Irrecoverable(error))`.
    fn parse_head(chars: &[char], start: usize, span: &Span) -> ParseHeadResult<Self>;

    /// True when the next atom should start (note: next digit 0-7; chord: always after suffixes end).
    fn head_boundary(chars: &[char], i: usize) -> bool;

    fn allows_octave_suffixes() -> bool {
        true
    }

    fn allows_fermata() -> bool {
        false
    }

    fn to_event(head: &Self, attrs: EventAttrs) -> ScoreEvent;
}

/// Shared attributes computed by `parse_timed_unit` and threaded through to
/// `TimedUnitHead::to_event`, which maps them onto the concrete `Parsed*` event fields.
#[derive(Debug, Clone, Copy)]
pub struct EventAttrs {
    pub duration: u32,
    pub dotted: bool,
    pub double_dotted: bool,
    pub fermata: bool,
    pub octave: i8,
    pub group_membership: u8,
    pub group_continuation: u8,
    pub tuplet: Option<TupletInfo>,
}

/// Head-boundary check shared by `NoteHead`/`ChordHead`: a fresh degree digit always starts a
/// new head. `r`/`_`/`=` immediately after a tie (`~`) also starts a new head — that's what lets
/// a repeat atom glued right after `~` (e.g. `5~_`) be parsed as its own unit instead of being
/// swallowed as a duration suffix of the tied note. Likewise, a `_`/`=` immediately after another
/// occurrence of the *same* character (`5__`, `5==`) starts a new head instead of being silently
/// absorbed as a no-op duration suffix — the repeat becomes a fresh repeat atom.
pub(crate) fn repeat_atom_boundary(chars: &[char], i: usize) -> bool {
    match chars.get(i) {
        Some('0'..='7') => true,
        Some('r') => i > 0 && chars.get(i - 1) == Some(&'~'),
        Some(c @ ('_' | '=')) => {
            i > 0 && matches!(chars.get(i - 1), Some(&prev) if prev == '~' || prev == *c)
        }
        _ => false,
    }
}
