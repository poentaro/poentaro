//! A whole-system selection paints these boxes directly. Every syllable
//! must be inside its own box, and consecutive syllables must meet.
use itertools::Itertools;
use quickcheck::QuickCheck;

use super::tests_lyric_click_targets::{no_header, test_render_config};
use crate::compiler::types::ElementContent;
use crate::grid_layout::click_targets::compute_all_lyric_click_targets;
use crate::grid_layout::layout::{block_column_width, MUSIC_START_COL};

#[test]
fn chord_lyric_highlights_cover_every_syllable_without_gaps_or_overlaps() {
    fn property(counts: Vec<u8>) -> bool {
        let counts = counts
            .into_iter()
            .take(8)
            .map(|count| count % 12 + 1)
            .collect_vec();
        if counts.is_empty() {
            return true;
        }
        let measures = counts
            .iter()
            .map(|count| {
                let syllables = std::iter::repeat_n("word", usize::from(*count)).join(" ");
                format!("[C] 1---\n[L] {syllables}")
            })
            .join("\n\n");
        let source = format!(
            "parts [\"C\": chords, \"L\": lyrics [C]],\nscore \"\"\"\n time=4/4\n{measures}\n\"\"\","
        );
        let score = crate::compile(&source, "test.poentaro", &[]).unwrap();
        let compiled = crate::compiler::compile(&score);
        assert!(compiled
            .blocks
            .iter()
            .all(|block| block.diagnostics.is_empty()));
        // Exercise both leading/trailing and interior measure boundaries.
        let systems = vec![vec![compiled.blocks.clone()]];
        let targets = compute_all_lyric_click_targets(
            &compiled.lyric_links,
            &systems,
            &Default::default(),
            &no_header(),
            14.0,
            false,
        );
        let syllables = compiled.blocks.iter().scan(MUSIC_START_COL, |offset, block| {
            let columns = block.rows.iter().flat_map(|row| &row.elements)
                .filter(|element| matches!(&element.content, ElementContent::Lyric { text, .. } if !text.is_empty()))
                .map(|element| (*offset + element.column) as f32).collect_vec();
            *offset += block_column_width(block);
            Some(columns)
        }).flatten().collect_vec();
        let end =
            (MUSIC_START_COL + compiled.blocks.iter().map(block_column_width).sum::<u32>()) as f32;
        targets.len() == syllables.len()
            && targets.iter().zip(&syllables).all(|((_, target), column)| {
                target.column_start <= *column && *column < target.column_end
            })
            && targets
                .iter()
                .tuple_windows()
                .all(|((_, left), (_, right))| left.column_end == right.column_start)
            && targets
                .first()
                .is_some_and(|(_, target)| target.column_start == 1.0)
            && targets
                .last()
                .is_some_and(|(_, target)| target.column_end == end)
    }
    QuickCheck::new()
        .tests(256)
        .quickcheck(property as fn(Vec<u8>) -> bool);
}

#[test]
fn dotted_note_lyric_highlight_reaches_the_next_note() {
    let source = "parts [\"M\": notes, \"L\": lyrics [M]],\nscore \"\"\"\n time=4/4\n[M] 1. 2_ 3 4\n[L] one two three four\n\"\"\",";
    let score = crate::compile(source, "test.poentaro", &[]).unwrap();
    let compiled = crate::compiler::compile(&score);
    let pages = crate::grid_layout::layout(
        &compiled,
        &test_render_config(),
        &no_header(),
        crate::grid_layout::PageLayout::A4,
        None,
    )
    .pages;
    let targets = &pages[0].lyric_click_targets;
    assert_eq!(targets.len(), 4);
    assert!(targets
        .iter()
        .tuple_windows()
        .all(|(left, right)| left.column_end == right.column_start));
}
