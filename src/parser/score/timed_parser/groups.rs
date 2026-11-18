use crate::ast::parsed::{ParsedSlurGroup, ScoreEvent};
use crate::error::{Span, Warning};

#[derive(Debug, Default, Clone, PartialEq)]
pub struct GroupStack {
    pub frames: Vec<GroupFrame>,
    /// The most recently parsed Note/Chord event on this track (rests never update it),
    /// used to resolve `r`/bare `_`/`=` repeat atoms. Persists across measures and lines,
    /// like ties/slurs already do.
    pub last_pitched_event: Option<ScoreEvent>,
    /// Every `(…)` group closed so far on this track, in closing order.
    pub closed_groups: Vec<ParsedSlurGroup>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GroupFrame {
    pub note_count: usize,
    has_pitched_note: bool,
    pub segment_start: usize,
    /// Source span of this group's opening `(`.
    pub open_paren_span: Span,
}

impl GroupStack {
    pub fn is_open(&self) -> bool {
        !self.frames.is_empty()
    }

    pub fn push(&mut self, segment_start: usize, open_paren_span: Span) {
        self.frames.push(GroupFrame {
            note_count: 0,
            has_pitched_note: false,
            segment_start,
            open_paren_span,
        });
    }

    /// Pops the innermost frame, closing it with the `)` at `close_paren_span`.
    pub fn close(&mut self, close_paren_span: Span) -> Option<GroupFrame> {
        let frame = self.frames.pop()?;
        self.closed_groups.push(ParsedSlurGroup {
            open_paren_span: frame.open_paren_span,
            close_paren_span,
        });
        Some(frame)
    }

    pub(super) fn mark_slur_continuation(&mut self, event: &mut ScoreEvent) {
        if let ScoreEvent::Note(note) = event {
            note.slur_continuation = self
                .frames
                .first_mut()
                .is_some_and(|outer| std::mem::replace(&mut outer.has_pitched_note, true));
        }
    }

    pub fn increment_note_count(&mut self) {
        // Every open frame (at every nesting level) gets the count, because each
        // note belongs to all enclosing groups.
        for frame in self.frames.iter_mut() {
            frame.note_count += 1;
        }
    }
}

/// Tracks currently-open `{...}` tuplet brackets on a single line. Unlike `GroupStack`,
/// tuplets never span lines, so a fresh stack is created per `parse_line` call.
#[derive(Debug, Default, Clone, PartialEq)]
pub struct TupletStack {
    pub frames: Vec<TupletFrame>,
    /// Monotonically increasing counter, incremented on every `open_tuplet`, so each
    /// bracket gets a distinct `TupletFrame::id` even when two brackets on the same
    /// line share the same `num`/`den` ratio.
    next_id: u32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TupletFrame {
    pub note_count: usize,
    pub segment_start: usize,
    pub num: u32,
    pub den: u32,
    pub id: u32,
}

impl TupletStack {
    pub fn is_open(&self) -> bool {
        !self.frames.is_empty()
    }

    pub fn open_tuplet(&mut self, segment_start: usize, num: u32, den: u32) {
        let id = self.next_id;
        self.next_id += 1;
        self.frames.push(TupletFrame {
            note_count: 0,
            segment_start,
            num,
            den,
            id,
        });
    }

    pub fn close_tuplet(&mut self) -> Option<TupletFrame> {
        self.frames.pop()
    }

    pub fn increment_note_count(&mut self) {
        for frame in self.frames.iter_mut() {
            frame.note_count += 1;
        }
    }
}

/// Standard implied "against" count for a bare `{N:...}` tuplet with no explicit `:M`
/// override. Returns `None` when `N` has no conventional ratio, in which case the caller
/// must require an explicit `{N:M:...}`.
pub fn implicit_tuplet_ratio(num: u32) -> Option<u32> {
    match num {
        2 => Some(3),
        3 => Some(2),
        4 => Some(3),
        5 => Some(4),
        6 => Some(4),
        7 => Some(4),
        9 => Some(8),
        _ => None,
    }
}

pub fn validate_group_note_count(count: usize, span: &Span) -> Option<Warning> {
    if count < 2 {
        Some(Warning::group_too_few_notes(*span))
    } else {
        None
    }
}

pub trait HasGroupDepth {
    fn group_membership(&self) -> u8;
    fn group_continuation(&self) -> u8;
    fn set_group_membership(&mut self, value: u8);
    fn set_group_continuation(&mut self, value: u8);
}

pub fn apply_closed_group_depth<T: HasGroupDepth>(atoms: &mut [T]) {
    let continuation_count = atoms.len().saturating_sub(1);
    for atom in atoms.iter_mut() {
        atom.set_group_membership(atom.group_membership().saturating_add(1));
    }
    for atom in atoms.iter_mut().take(continuation_count) {
        atom.set_group_continuation(atom.group_continuation().saturating_add(1));
    }
}

pub fn apply_open_group_depth<T: HasGroupDepth>(atoms: &mut [T]) {
    for atom in atoms.iter_mut() {
        atom.set_group_membership(atom.group_membership().saturating_add(1));
        atom.set_group_continuation(atom.group_continuation().saturating_add(1));
    }
}
