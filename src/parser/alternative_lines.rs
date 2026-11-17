//! Alternative lines: a measure group may write a part's `[Key]` line more
//! than once, and its Nth line (N ≥ 2) belongs to a part of its own,
//! `Key.N`, declared right after the part (see `AlternativeLine`). The
//! extra parts are drawn as extra rows, and play only where a `sequence`
//! entry picks them (`Verse WITH ([w] 2)`).

use std::collections::HashMap;

use crate::ast::parsed::{AlternativeLine, PartDecl, PartKind};
use crate::desugar::parse_key_prefix;
use crate::parser::score::measure_group::collect_groups;

/// `declarations` with an alternative-line part after each part for every
/// extra line some measure group of `score_text` writes for it.
pub(crate) fn with_alternative_lines(
    declarations: Vec<PartDecl>,
    score_text: &str,
) -> Vec<PartDecl> {
    let line_counts = most_lines_per_key(score_text);
    let extra_lines = |declaration: &PartDecl| {
        line_counts
            .get(&declaration.abbreviation)
            .copied()
            .unwrap_or(1)
            .saturating_sub(1)
    };
    if declarations
        .iter()
        .all(|declaration| extra_lines(declaration) == 0)
    {
        return declarations;
    }
    let new_index_of = declarations
        .iter()
        .scan(0usize, |next, declaration| {
            let index = *next;
            *next += 1 + extra_lines(declaration) as usize;
            Some(index)
        })
        .collect::<Vec<_>>();
    let remap = |index: usize| new_index_of.get(index).copied().unwrap_or(index);
    declarations
        .iter()
        .flat_map(|declaration| {
            let mut declaration = declaration.clone();
            if let PartKind::Lyrics { target_part_index } = &mut declaration.kind {
                *target_part_index = remap(*target_part_index);
            }
            declaration.lyric_targets = declaration
                .lyric_targets
                .iter()
                .map(|&i| remap(i))
                .collect();
            let alternatives = (2..=extra_lines(&declaration) + 1)
                .map(|number| alternative_of(&declaration, number))
                .collect::<Vec<_>>();
            std::iter::once(declaration).chain(alternatives)
        })
        .collect()
}

fn alternative_of(declaration: &PartDecl, number: u32) -> PartDecl {
    let line = AlternativeLine {
        base: declaration.abbreviation.clone(),
        number,
    };
    PartDecl {
        abbreviation: line.abbreviation(),
        alternative_line: Some(line),
        ..declaration.clone()
    }
}

/// The most `[Key]` lines any one measure group writes, per key.
fn most_lines_per_key(score_text: &str) -> HashMap<String, u32> {
    let mut most: HashMap<String, u32> = HashMap::new();
    for group in collect_groups(score_text) {
        let mut counts: HashMap<&str, u32> = HashMap::new();
        for (line, _) in &group {
            if let Some((key, _)) = parse_key_prefix(line) {
                *counts.entry(key).or_default() += 1;
            }
        }
        for (key, count) in counts {
            let entry = most.entry(key.to_string()).or_default();
            *entry = (*entry).max(count);
        }
    }
    most
}

#[cfg(test)]
#[path = "alternative_lines_tests.rs"]
mod tests;
