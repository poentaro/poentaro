//! [`format_measures`]: a best-effort formatter for a score's measures. Three
//! independent cleanups run per measure group:
//!
//! 1. **Redundant-line removal.** A `[Key]` data line is dropped when it is
//!    exactly what the existing implicit-fill machinery
//!    (`desugar::implicit_fill`) would already produce if that key were not
//!    mentioned at all in this measure group — an all-rest line for a
//!    `Notes`/`Chord`-role occurrence, or an all-`_` line for a `Lyrics`-role
//!    occurrence. `follow[X]` parts are never touched (their
//!    implicit fill is the follow target's content, not rest).
//! 2. **Part-order sorting.** A key's surviving line is moved to the position
//!    matching its part's order in `parts` (a lyric part right after its
//!    target), regardless of how the lines were interleaved in the source. A key
//!    that can't be resolved to a declared part keeps its original relative
//!    position, ordered after every recognised part.
//! 3. **Whitespace normalization.** Every directive line and surviving data
//!    line has its runs of internal whitespace collapsed to one space
//!    (quote-aware for directive lines, so `label="Two Words"` survives as a
//!    single token).
//!
//! The redundant-line removal reuses `desugar::desugar_groups` rather than
//! re-implementing "how would this resolve": once the eligible trailing
//! lines are stripped from the raw input, feeding the result back through
//! `desugar_groups` fills the dropped slots correctly for free.
//!

use crate::ast::parsed::{PartDecl, ScoreLineRole};
use crate::desugar;
use crate::parser;

/// A raw, not-yet-desugared score line paired with its byte offset within
/// its containing section, matching `measure_group::collect_groups`'s output.
type RawSourceLine = (String, usize);

/// Formats measures in the current score syntax (measure groups split by
/// blank lines) with the three cleanups in the module docs. `None` when the
/// result would no longer desugar.
pub(crate) fn format_measures(
    score_content: &str,
    score_offset: usize,
    declarations: &[PartDecl],
) -> Option<String> {
    let raw_groups = parser::score::measure_group::collect_groups(score_content);
    let filtered_groups: Vec<Vec<RawSourceLine>> = raw_groups
        .iter()
        .map(|group| format_group(group, declarations))
        .collect();

    // Validate that the filtered/reordered groups still desugar cleanly
    // (best-effort safety net — `desugar_groups` is not used for rendering
    // here: it always materializes exactly one line per declared part per
    // group regardless of whether that part had an explicit line, which
    // would put every dropped fixed-schema (Notes/Chords/Percussion) rest
    // line right back. The actual output is built directly from
    // `filtered_groups` below, so intentionally-dropped lines stay dropped
    // and real `.poentaro` parsing's own implicit-fill covers them, exactly as
    // it already does for any measure group that omits a part's line.
    if desugar::desugar_groups(filtered_groups.clone(), declarations, score_offset).is_err() {
        return None;
    }

    let group_texts: Vec<String> = filtered_groups
        .iter()
        .map(|group| {
            group
                .iter()
                .map(|(content, _offset)| content.as_str())
                .collect::<Vec<_>>()
                .join("\n")
        })
        .collect();
    Some(if group_texts.is_empty() {
        String::new()
    } else {
        format!("{}\n", group_texts.join("\n\n"))
    })
}

/// The declaration a `[Key]` addresses.
fn declaration_index_of(key: &str, declarations: &[PartDecl]) -> Option<usize> {
    declarations
        .iter()
        .position(|decl| decl.abbreviation == key)
}

/// Which data lines are redundant with implicit fill: a line whose content is
/// exactly what its part would be filled with when unmentioned.
/// `follow[X]` lines are never removable (their implicit fill is the
/// target's content, not rest).
fn removable_lines(parsed: &[Option<(&str, &str)>], declarations: &[PartDecl]) -> Vec<bool> {
    parsed
        .iter()
        .map(|entry| {
            entry
                .and_then(|(key, content)| {
                    let decl = declarations.get(declaration_index_of(key, declarations)?)?;
                    Some(
                        decl.follow_target.is_none()
                            && is_removable(decl.score_line_role(), content),
                    )
                })
                .unwrap_or(false)
        })
        .collect()
}

/// Every whitespace-split token is a rest: `0` optionally followed by a run
/// of `_`/`=`/`.`/`-` suffix characters, or a bare `-`/`-.` dash-only
/// extension atom (extends a preceding rest without repeating `0`).
fn is_all_rest(content: &str) -> bool {
    let tokens: Vec<&str> = content.split_whitespace().collect();
    !tokens.is_empty()
        && tokens
            .iter()
            .all(|token| *token == "-" || *token == "-." || is_rest_token(token))
}

fn is_rest_token(token: &str) -> bool {
    token
        .strip_prefix('0')
        .is_some_and(|rest| rest.chars().all(|c| matches!(c, '_' | '=' | '.' | '-')))
}

/// Every whitespace-split token is exactly `_` (no lyrics for this measure).
fn is_all_no_lyrics(content: &str) -> bool {
    let tokens: Vec<&str> = content.split_whitespace().collect();
    !tokens.is_empty() && tokens.iter().all(|token| *token == "_")
}

fn is_removable(role: ScoreLineRole, content: &str) -> bool {
    match role {
        ScoreLineRole::Notes | ScoreLineRole::Chord => is_all_rest(content),
        ScoreLineRole::Lyrics => is_all_no_lyrics(content),
    }
}

/// Builds the filtered/reordered/whitespace-normalized raw measure group for
/// `group`: directive line(s) normalized, unparseable data lines passed
/// through untouched, eligible trailing redundant `[Key]` lines dropped, and
/// surviving `[Key]` blocks sorted into `parts` declaration order (see
/// module docs).
fn format_group(group: &[RawSourceLine], declarations: &[PartDecl]) -> Vec<RawSourceLine> {
    let directive_count = parser::score::measure_group::directive_line_count(group);
    let directive_lines = group.get(..directive_count).unwrap_or(&[]);
    let data_lines = group.get(directive_count..).unwrap_or(&[]);

    // Parsed `(key, content)` per data line, `None` for lines left untouched.
    let parsed: Vec<Option<(&str, &str)>> = data_lines
        .iter()
        .map(|(line, _offset)| desugar::parse_key_prefix(line))
        .collect();

    let mut removable = removable_lines(&parsed, declarations);

    let remaining_count = removable.iter().filter(|r| !**r).count();
    if remaining_count == 0 {
        if let Some(last_removable) = removable.iter().rposition(|r| *r) {
            if let Some(slot) = removable.get_mut(last_removable) {
                *slot = false;
            }
        }
    }

    let normalized_data_lines: Vec<RawSourceLine> = data_lines
        .iter()
        .zip(parsed.iter())
        .map(|((line, offset), entry)| {
            let normalized = match entry {
                Some((key, content)) => format!("[{key}] {}", normalize_data_line(content)),
                None => line.clone(),
            };
            (normalized, *offset)
        })
        .collect();

    let mut result: Vec<RawSourceLine> = directive_lines
        .iter()
        .map(|(content, offset)| (normalize_directive_line(content), *offset))
        .collect();
    result.extend(sort_data_lines_by_declaration(
        normalized_data_lines,
        &parsed,
        &removable,
        declarations,
    ));
    result
}

/// A key's surviving data lines, or one unparseable line, as a unit that
/// moves together when reordering.
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
enum DataLineBlockKey<'a> {
    Key(&'a str),
    /// Carries the line's original index so each unparseable line is its
    /// own block, distinct from every other.
    Unparsed(usize),
}

/// Groups surviving data lines into blocks — every occurrence of a key
/// travels together, regardless of how interleaved with other keys' lines
/// it was in the source — and orders the blocks by the part's position in
/// `parts`. A block for an unrecognised or unparseable key keeps its
/// original relative position, ordered after every recognised part (its
/// content isn't touched here; any resulting invalid document is caught by
/// `format_score`'s `desugar_groups` safety net).
fn sort_data_lines_by_declaration<'a>(
    normalized_data_lines: Vec<RawSourceLine>,
    parsed: &[Option<(&'a str, &'a str)>],
    removable: &[bool],
    declarations: &[PartDecl],
) -> Vec<RawSourceLine> {
    // Blocks in first-seen order; a linear scan per line is fine since a
    // measure group only ever has a handful of distinct keys.
    let mut blocks: Vec<(DataLineBlockKey<'a>, Vec<RawSourceLine>)> = Vec::new();
    for (index, ((line, drop), entry)) in normalized_data_lines
        .into_iter()
        .zip(removable.iter())
        .zip(parsed.iter())
        .enumerate()
    {
        if *drop {
            continue;
        }
        let block_key = match entry {
            Some((key, _)) => DataLineBlockKey::Key(key),
            None => DataLineBlockKey::Unparsed(index),
        };
        match blocks
            .iter_mut()
            .find(|(existing, _)| *existing == block_key)
        {
            Some((_, lines)) => lines.push(line),
            None => blocks.push((block_key, vec![line])),
        }
    }

    // `sort_by_key` is stable, so blocks with equal rank (including every
    // unrecognised-key block, which all rank `usize::MAX`) keep their
    // first-seen relative order.
    blocks.sort_by_key(|(block_key, _)| match block_key {
        DataLineBlockKey::Key(key) => declaration_index_of(key, declarations).unwrap_or(usize::MAX),
        DataLineBlockKey::Unparsed(_) => usize::MAX,
    });

    blocks.into_iter().flat_map(|(_, lines)| lines).collect()
}

/// Collapses whitespace to single spaces, leaving unparseable lines and
/// non-`[Key]`-prefixed content untouched by callers (this only runs on
/// lines already confirmed to parse).
fn normalize_data_line(content: &str) -> String {
    content.split_whitespace().collect::<Vec<_>>().join(" ")
}

/// Collapses whitespace to single spaces outside of `"..."` quoted spans, so
/// a `label="Two Words"` token survives as one token.
fn normalize_directive_line(content: &str) -> String {
    let mut tokens: Vec<String> = Vec::new();
    let mut current = String::new();
    let mut in_quotes = false;
    for c in content.chars() {
        if c == '"' {
            in_quotes = !in_quotes;
            current.push(c);
        } else if c.is_whitespace() && !in_quotes {
            if !current.is_empty() {
                tokens.push(std::mem::take(&mut current));
            }
        } else {
            current.push(c);
        }
    }
    if !current.is_empty() {
        tokens.push(current);
    }
    tokens.join(" ")
}

#[cfg(test)]
#[path = "format_source_tests.rs"]
mod tests;
