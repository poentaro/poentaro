//! Resolving a `lyrics [X]` / `lyrics [S, A, T, B]` declaration against the
//! parts declared before it.

use crate::ast::parsed::{PartDecl, PartKind};
use crate::error::{RecoverableError, Span};

use super::{LyricTarget, DEFAULT_PART_OCTAVE_OFFSET, DEFAULT_PART_VOLUME};

/// A lyric part's own fields, before its targets are resolved.
pub(super) struct LyricsDeclarationHeader {
    pub(super) abbreviation: String,
    pub(super) abbreviation_span: Span,
    pub(super) display_name: String,
}

/// Resolves a lyric part against the parts declared so far, reporting an
/// error (and yielding nothing) when a target isn't an earlier notes or
/// chords part. Several targets must all be notes parts: chords have no
/// notes to share syllables with.
pub(super) fn resolve_lyrics_declaration(
    declarations: &[PartDecl],
    header: LyricsDeclarationHeader,
    targets: &[LyricTarget],
    errors: &mut Vec<RecoverableError>,
) -> Option<PartDecl> {
    let lyric_targets = targets
        .iter()
        .map(|target| target_index(declarations, target, targets.len() > 1))
        .collect::<Result<Vec<usize>, RecoverableError>>()
        .map_err(|error| errors.push(error))
        .ok()?;
    // The row is drawn under the bottommost target, and its syllables are
    // paired against that target's notes.
    let target_part_index = lyric_targets.iter().copied().max()?;
    Some(PartDecl {
        abbreviation: header.abbreviation,
        abbreviation_span: header.abbreviation_span,
        display_name: header.display_name,
        kind: PartKind::Lyrics { target_part_index },
        follow_target: None,
        range_instrument: None,
        written_range: None,
        soundfont: PartKind::Lyrics { target_part_index }.default_soundfont(),
        volume: DEFAULT_PART_VOLUME,
        octave_offset: DEFAULT_PART_OCTAVE_OFFSET,
        singing_voice: None,
        lyric_targets,
        alternative_line: None,
    })
}

fn target_index(
    declarations: &[PartDecl],
    target: &LyricTarget,
    one_of_several: bool,
) -> Result<usize, RecoverableError> {
    declarations
        .iter()
        .position(|declaration| {
            declaration.abbreviation == target.abbreviation
                && match declaration.kind {
                    PartKind::Notes => true,
                    PartKind::Chords => !one_of_several,
                    _ => false,
                }
        })
        .ok_or_else(|| {
            RecoverableError::parts_lyrics_invalid_target(target.span, &target.abbreviation)
        })
}
