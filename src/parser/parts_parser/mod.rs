mod declaration_parsing;
mod instrument_matching;
mod lyrics_declaration;
mod setting_limits;

use crate::ast::parsed::{PartDecl, PartKind, RangeInstrument, Soundfont};
use crate::error::{RecoverableError, Span};

pub use instrument_matching::InstrumentInfo;
pub use setting_limits::{
    ClampedSetting, PartSettingLimits, PartSettingRange, PART_SETTING_LIMITS,
};

pub(crate) use declaration_parsing::{
    checked_octave_offset, checked_range_instrument, checked_volume,
};
pub(crate) use instrument_matching::validate_soundfont;
use lyrics_declaration::{resolve_lyrics_declaration, LyricsDeclarationHeader};

#[cfg(test)]
mod default_soundfont_tests;
#[cfg(test)]
mod lyrics_part_tests;
#[cfg(test)]
mod percussion_tests;
#[cfg(test)]
mod setting_limits_tests;
#[cfg(test)]
mod source_level_tests;
#[cfg(test)]
mod tests;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SourcePartMode {
    Chords,
    Notes,
    Percussion,
    Follow,
    Lyrics,
}

/// Source-level part declaration before follow inheritance is applied.
/// A part's volume when its `parts` line has no `N%` suffix.
pub const DEFAULT_PART_VOLUME: u8 = 100;
/// A part's octave offset when its `parts` line has no `+N`/`-N` suffix.
pub const DEFAULT_PART_OCTAVE_OFFSET: i8 = 0;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourceRawPartDecl {
    pub display_name: String,
    pub abbreviation: String,
    pub line_number: u32,
    pub mode: SourcePartMode,
    pub follow_target: Option<String>,
    pub range_instrument: Option<RangeInstrument>,
    pub soundfont: Option<Soundfont>,
    pub volume: Option<u8>,
    pub octave_offset: Option<i8>,
}

pub(crate) struct RawDecl {
    pub(crate) display_name: String,
    pub(crate) abbreviation: String,
    pub(crate) abbreviation_span: Span,
    pub(crate) span: Span,
    pub(crate) kind: RawKind,
    /// `None` when omitted on the declaration line (follow parts inherit from target).
    pub(crate) range_instrument: Option<RangeInstrument>,
    /// `None` when omitted on the declaration line (follow parts inherit from target).
    pub(crate) soundfont: Option<Soundfont>,
    /// `None` when omitted on the declaration line (follow parts inherit from target).
    pub(crate) volume: Option<u8>,
    /// `None` when omitted on the declaration line (follow parts inherit from target).
    pub(crate) octave_offset: Option<i8>,
}

pub(crate) enum RawKind {
    Concrete(PartKind),
    Follow { target: String, target_span: Span },
    Lyrics { target: String, target_span: Span },
}

fn byte_offset_to_line_number(source: &str, byte_offset: usize) -> u32 {
    source
        .as_bytes()
        .iter()
        .take(byte_offset.min(source.len()))
        .filter(|&&byte| byte == b'\n')
        .count() as u32
        + 1
}

fn raw_kind_to_source_mode(kind: &RawKind) -> (SourcePartMode, Option<String>) {
    match kind {
        RawKind::Concrete(PartKind::Chords) => (SourcePartMode::Chords, None),
        RawKind::Concrete(PartKind::Notes) => (SourcePartMode::Notes, None),
        RawKind::Concrete(PartKind::Percussion) => (SourcePartMode::Percussion, None),
        // A lyric part is only ever built as `RawKind::Lyrics`.
        RawKind::Concrete(PartKind::Lyrics { .. }) => (SourcePartMode::Lyrics, None),
        RawKind::Follow { target, .. } => (SourcePartMode::Follow, Some(target.clone())),
        RawKind::Lyrics { target, .. } => (SourcePartMode::Lyrics, Some(target.clone())),
    }
}

/// `raw` as source-level part declarations of `full_source`.
pub(crate) fn source_declarations(raw: Vec<RawDecl>, full_source: &str) -> Vec<SourceRawPartDecl> {
    raw.into_iter()
        .map(|raw_decl| {
            let (mode, follow_target) = raw_kind_to_source_mode(&raw_decl.kind);
            SourceRawPartDecl {
                display_name: raw_decl.display_name,
                abbreviation: raw_decl.abbreviation,
                line_number: byte_offset_to_line_number(full_source, raw_decl.span.start),
                mode,
                follow_target,
                range_instrument: raw_decl.range_instrument,
                soundfont: raw_decl.soundfont,
                volume: raw_decl.volume,
                octave_offset: raw_decl.octave_offset,
            }
        })
        .collect()
}

pub(crate) fn resolve_declarations(
    raw: Vec<RawDecl>,
    errors: &mut Vec<RecoverableError>,
) -> Vec<PartDecl> {
    let mut declarations = Vec::new();
    for (index, raw_decl) in raw.into_iter().enumerate() {
        let RawDecl {
            display_name,
            abbreviation,
            abbreviation_span,
            span,
            kind,
            range_instrument,
            soundfont,
            volume,
            octave_offset,
        } = raw_decl;
        let soundfont = soundfont.or(range_instrument.map(RangeInstrument::default_soundfont));
        match kind {
            RawKind::Follow {
                target,
                target_span,
            } => {
                if index == 0 {
                    errors.push(RecoverableError::parts_first_part_cannot_follow(span));
                    continue;
                }
                let found = declarations
                    .iter()
                    .find(|declaration: &&PartDecl| declaration.abbreviation == target);
                match found {
                    None => {
                        errors.push(RecoverableError::parts_follow_unknown_target(
                            target_span,
                            &target,
                        ));
                        continue;
                    }
                    Some(target_decl) => declarations.push(PartDecl {
                        abbreviation,
                        abbreviation_span,
                        display_name,
                        kind: target_decl.kind,
                        follow_target: Some(target),
                        range_instrument: range_instrument.or(target_decl.range_instrument),
                        soundfont: soundfont.unwrap_or(target_decl.soundfont),
                        volume: volume.unwrap_or(target_decl.volume),
                        octave_offset: octave_offset.unwrap_or(target_decl.octave_offset),
                    }),
                }
            }
            RawKind::Concrete(kind) => declarations.push(PartDecl {
                abbreviation,
                abbreviation_span,
                display_name,
                kind,
                follow_target: None,
                range_instrument,
                soundfont: soundfont.unwrap_or(kind.default_soundfont()),
                volume: volume.unwrap_or(DEFAULT_PART_VOLUME),
                octave_offset: octave_offset.unwrap_or(DEFAULT_PART_OCTAVE_OFFSET),
            }),
            RawKind::Lyrics {
                target,
                target_span,
            } => {
                let declaration = resolve_lyrics_declaration(
                    &declarations,
                    LyricsDeclarationHeader {
                        abbreviation,
                        abbreviation_span,
                        display_name,
                    },
                    &target,
                    target_span,
                    errors,
                );
                declarations.extend(declaration);
            }
        }
    }
    lyric_parts_after_their_targets(declarations)
}

/// Reorders `declarations` so each lyric part sits directly after its target
/// (and after earlier lyric parts of the same target), keeping declaration
/// order otherwise, then re-points every `target_part_index` at the new order.
/// A lyric part's row is drawn directly under its target's, so part order is
/// row order.
fn lyric_parts_after_their_targets(declarations: Vec<PartDecl>) -> Vec<PartDecl> {
    let lyric_target = |declaration: &PartDecl| match declaration.kind {
        PartKind::Lyrics { target_part_index } => Some(target_part_index),
        _ => None,
    };
    let old_order: Vec<usize> = declarations
        .iter()
        .enumerate()
        .filter(|(_, declaration)| lyric_target(declaration).is_none())
        .flat_map(|(index, _)| {
            std::iter::once(index).chain(
                declarations
                    .iter()
                    .enumerate()
                    .filter(move |(_, declaration)| lyric_target(declaration) == Some(index))
                    .map(|(lyric_index, _)| lyric_index),
            )
        })
        .collect();
    let new_index_of = |old_index: usize| {
        old_order
            .iter()
            .position(|&candidate| candidate == old_index)
            .unwrap_or(old_index)
    };
    let mut slots: Vec<Option<PartDecl>> = declarations.into_iter().map(Some).collect();
    old_order
        .iter()
        .filter_map(|&old_index| slots.get_mut(old_index).and_then(Option::take))
        .map(|mut declaration| {
            if let PartKind::Lyrics { target_part_index } = &mut declaration.kind {
                *target_part_index = new_index_of(*target_part_index);
            }
            declaration
        })
        .collect()
}
