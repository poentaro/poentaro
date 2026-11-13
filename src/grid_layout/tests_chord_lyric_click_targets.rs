use super::tests_lyric_click_targets::{no_header, test_render_config};
use crate::compiler::types::{
    ColumnElement, CompileResult, ElementContent, MeasureBlock, MeasureRow, RowId,
};

/// A chord row with one chord (attack at column 0, dash-continuation at
/// column 3) followed by a bar line, paired with a lyric row whose bracketed
/// syllable shares the chord's note id and whose unbracketed syllables sit
/// inside the chord's span.
fn chord_row_and_chord_lyric_row() -> Vec<MeasureRow> {
    let element = |column, content, note_id| ColumnElement {
        column,
        content,
        note_id,
    };
    let lyric = |column, text: &str, note_id| {
        element(
            column,
            ElementContent::Lyric {
                text: text.to_string(),
                ruby: None,
                note_id,
            },
            None,
        )
    };
    vec![
        MeasureRow {
            kind: crate::ast::parsed::PartKind::Chords,
            absorbed_rows: Vec::new(),
            id: RowId("C".to_string()),
            label: String::new(),
            elements: vec![
                element(
                    0,
                    ElementContent::ChordSymbol {
                        text: "C".to_string(),
                        degrees: crate::compiler::test_chord_degrees::tonic_major(),
                        guitar_voicing: None,
                        dotted: false,
                        double_dotted: false,
                    },
                    Some(0),
                ),
                element(
                    3,
                    ElementContent::NoteDash {
                        dotted: false,
                        double_dotted: false,
                    },
                    Some(0),
                ),
                element(5, ElementContent::BarLine, None),
            ],
            source_part_index: 0,
        },
        MeasureRow {
            kind: crate::ast::parsed::PartKind::Lyrics {
                target_part_index: 0,
            },
            absorbed_rows: Vec::new(),
            id: RowId("C-lyrics-0".to_string()),
            label: String::new(),
            elements: vec![
                lyric(0, "(陪)", 0),
                lyric(1, "我", 1 << 28),
                lyric(2, "你", (1 << 28) + 1),
                element(5, ElementContent::BarLine, None),
            ],
            source_part_index: 1,
        },
    ]
}

/// Under a chord part the lyric line carries its own timing: a bracketed
/// syllable must not be widened to its chord's full span, or it would
/// overlap the unbracketed syllables spread inside that span.
#[test]
fn chord_lyric_click_targets_stay_one_column_wide() {
    let block = MeasureBlock {
        rows: chord_row_and_chord_lyric_row(),
        decorations: vec![],
        diagnostics: vec![],
        represents_measures: 1,
        merge_duplicate_measures_across_parts: true,
        system_break: false,
        source_span: crate::error::Span::new(0, 0),
    };
    let link = crate::compiler::types::LyricLink {
        lyric_row_id: RowId("C-lyrics-0".to_string()),
        target_row_id: RowId("C".to_string()),
    };
    let pages = crate::grid_layout::layout(
        &CompileResult {
            blocks: vec![block],
            slur_spans: vec![],
            tuplet_spans: vec![],
            lyric_links: vec![link],
        },
        &test_render_config(),
        &no_header(),
        crate::grid_layout::PageLayout::A4,
        None,
    )
    .pages;

    let targets = &pages[0].lyric_click_targets;
    assert_eq!(targets.len(), 3, "one click target per syllable");
    let bracketed = targets.iter().find(|t| t.note_id == 0).expect("bracketed");
    let floating = targets
        .iter()
        .find(|t| t.note_id == 1 << 28)
        .expect("floating");
    // Widened to its chord's span it would end at column 6.0; instead it is
    // one column past its (bar-line-snapped) start.
    assert_eq!(bracketed.column_end - bracketed.column_start, 1.0);
    assert_eq!(floating.column_end - floating.column_start, 1.0);
}
