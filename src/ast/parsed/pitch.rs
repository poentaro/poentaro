/// A translation in points, applied to a rendered element after its layout
/// position has been resolved. Does not affect the position of any other
/// element.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Offset {
    pub x: i32,
    pub y: i32,
}

impl std::fmt::Display for Offset {
    /// As the metadata form's text input shows it: `0 -4`.
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(formatter, "{} {}", self.x, self.y)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum NumberedPitch {
    One,
    Two,
    Three,
    Four,
    Five,
    Six,
    Seven,
}

impl NumberedPitch {
    /// The numbered digit glyph (`1`-`7`) a pitch renders as, shared by the
    /// renderer (`render_note_head`) and the coordinate resolver (which needs
    /// the same leading character to measure the note head's own left-side
    /// bearing — see `coordinate_resolver::resolve::flush_left_padding`).
    pub(crate) fn to_digit(&self) -> char {
        use NumberedPitch::*;
        match self {
            One => '1',
            Two => '2',
            Three => '3',
            Four => '4',
            Five => '5',
            Six => '6',
            Seven => '7',
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct KeyChange {
    pub note: Note,
}

impl Default for KeyChange {
    /// The key in effect before any `key=` directive: C4.
    fn default() -> Self {
        Self {
            note: Note {
                name: NoteName::C,
                octave: 4,
                accidental: Accidental::Natural,
            },
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Note {
    pub name: NoteName,
    pub octave: u8,
    pub accidental: Accidental,
}

#[derive(Debug, Clone, PartialEq)]
pub enum NoteName {
    A,
    B,
    C,
    D,
    E,
    F,
    G,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Accidental {
    Flat,
    Sharp,
    Natural,
}
