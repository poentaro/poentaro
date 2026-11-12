//! Resolving a `lyrics[X]` declaration against the parts declared before it.

use crate::ast::parsed::{PartDecl, PartKind, SingingVoice};
use crate::error::{RecoverableError, Span, Spanned};

use super::{DEFAULT_PART_OCTAVE_OFFSET, DEFAULT_PART_VOLUME};

/// A `lyrics[X]` declaration's own fields, before its target is resolved.
pub(super) struct LyricsDeclarationHeader {
    pub(super) abbreviation: String,
    pub(super) abbreviation_span: Span,
    pub(super) display_name: String,
    pub(super) singing_voice: Option<Spanned<SingingVoice>>,
    pub(super) volume: Option<u8>,
}

/// Resolves a `lyrics[target]` declaration against the parts declared so far,
/// reporting an error (and yielding nothing) when `target` isn't an earlier
/// notes or chords part.
pub(super) fn resolve_lyrics_declaration(
    declarations: &[PartDecl],
    header: LyricsDeclarationHeader,
    target: &str,
    target_span: Span,
    errors: &mut Vec<RecoverableError>,
) -> Option<PartDecl> {
    let target_part_index = declarations.iter().position(|declaration| {
        declaration.abbreviation == target
            && matches!(declaration.kind, PartKind::Notes | PartKind::Chords)
    });
    let Some(target_part_index) = target_part_index else {
        errors.push(RecoverableError::parts_lyrics_invalid_target(
            target_span,
            target,
        ));
        return None;
    };
    let targets_chords = declarations
        .get(target_part_index)
        .is_some_and(|target| target.kind == PartKind::Chords);
    let singing_voice = match header.singing_voice {
        // Syllables under chords have no pitch of their own to sing.
        Some(voice) if targets_chords => {
            errors.push(RecoverableError::parts_voice_on_chord_lyrics(voice.span));
            None
        }
        voice => voice.map(|voice| voice.value),
    };
    Some(PartDecl {
        abbreviation: header.abbreviation,
        abbreviation_span: header.abbreviation_span,
        display_name: header.display_name,
        kind: PartKind::Lyrics { target_part_index },
        follow_target: None,
        range_instrument: None,
        written_range: None,
        soundfont: PartKind::Lyrics { target_part_index }.default_soundfont(),
        volume: header.volume.unwrap_or(DEFAULT_PART_VOLUME),
        octave_offset: DEFAULT_PART_OCTAVE_OFFSET,
        singing_voice,
    })
}
