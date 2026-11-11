// ── Span-aware column spring weight ───────────────────────────────────────────
//
// `measure_column_sizes` now splits an element's *weight* (spring) across
// every raw grid column any row actually anchors content to within that
// element's own span, while its *rod* stays concentrated entirely on its own
// start column (see the `column_geometry` docs). These tests
// exercise that split directly, distinct from `tests_column_rod_spring.rs`'s
// rod-floor invariants and `tests_measure_spacing.rs`'s span-1 proportional
// invariants (both of which stay numerically unaffected by this change).

use crate::ast::parsed::{NumberedPitch, Offset};
use crate::compiler::types::{ColumnElement, ElementContent, MeasureBlock, MeasureRow, RowId};
use crate::font_metrics;
use crate::grid_layout::layout::{build_measure_column_layout, measure_column_weights};
use crate::render_config::RenderConfig;

fn test_config() -> RenderConfig {
    RenderConfig {
        row_height: 30,
        note_number_width: 12,
        part_label_width_pt: 40,
        max_measures_per_system: 48,
        lyrics_font_size: 18,
        notes_font_size: 18,
        note_dash_font_size: 18,
        chords_font_size: 18,
        hide_system_dividers: false,
        directive_row_offset: Offset::default(),
        measure_number_font_size: 10,
        section_label_font_size: 12,
        part_label_font_size: 12,
        page_number_font_size: 18,
        lyric_click_target_padding_pt: 12,
        notes_vertical_padding_pt: 0,
        section_label_vertical_padding_pt: 0,
        page_number_vertical_padding_pt: 0,
        notes_horizontal_padding_pt: 4,
        chords_horizontal_padding_pt: 4,
        lyrics_horizontal_padding_pt: 4,
        note_dash_horizontal_padding_pt: 4,
        ..Default::default()
    }
}

/// Mirrors `layout_spacing_weights::note_glyph_weight`.
fn notehead_weight(config: &RenderConfig) -> f32 {
    font_metrics::monospace_char_advance_width('0', config.notes_font_size())
}

/// Mirrors `layout_spacing_weights::chord_symbol_weight`.
fn chord_weight(text: &str, config: &RenderConfig) -> f32 {
    font_metrics::monospace_text_width(text, config.chords_font_size()).max(notehead_weight(config))
}

fn row(id: &str, elements: Vec<ColumnElement>) -> MeasureRow {
    MeasureRow {
        kind: crate::ast::parsed::PartKind::Notes,
        absorbed_rows: Vec::new(),
        id: RowId(id.to_string()),
        label: id.to_string(),
        elements,
        source_part_index: 0,
    }
}

fn note_head(column: u32) -> ColumnElement {
    ColumnElement {
        column,
        content: ElementContent::NoteHead {
            pitch: NumberedPitch::One,
            accidental: crate::ast::parsed::Accidental::Natural,
            octave: 0,
            dotted: false,
            double_dotted: false,
            head_label: None,
        },
        note_id: None,
    }
}

fn rest(column: u32) -> ColumnElement {
    ColumnElement {
        column,
        content: ElementContent::Rest {
            dotted: false,
            double_dotted: false,
            implicit_fill: false,
        },
        note_id: None,
    }
}

fn chord(column: u32, text: &str) -> ColumnElement {
    ColumnElement {
        column,
        content: ElementContent::ChordSymbol {
            text: text.to_string(),
            degrees: crate::compiler::test_chord_degrees::tonic_major(),
            guitar_voicing: None,
            dotted: false,
            double_dotted: false,
        },
        note_id: None,
    }
}

fn bar_line(column: u32) -> ColumnElement {
    ColumnElement {
        column,
        content: ElementContent::BarLine,
        note_id: None,
    }
}

fn block(rows: Vec<MeasureRow>) -> MeasureBlock {
    MeasureBlock {
        rows,
        decorations: vec![],
        diagnostics: vec![],
        represents_measures: 1,
        merge_duplicate_measures_across_parts: true,
        system_break: false,
        source_span: crate::error::Span::new(0, 0),
    }
}

/// The worked example from conversation: a one-beat chord symbol
/// (`6m/3`-style, no dash) co-occurring with two eighth notes in another
/// row. Today the chord's whole weight piles onto its own column alone;
/// after the fix it must be shared with the second eighth note's column
/// too, since that column sits inside the chord's own span. The chord's
/// *rod*, by contrast, must stay entirely on its own start column — an
/// explicit regression check against reintroducing visual overlap.
#[test]
fn chord_weight_splits_across_eighth_note_columns_in_its_span_but_rod_stays_on_its_own_column() {
    let config = test_config();
    let text = "6m/3";
    let b = block(vec![
        row("Chord", vec![chord(0, text), bar_line(4)]),
        row("Notes", vec![note_head(0), note_head(2), bar_line(4)]),
    ]);

    let weights = measure_column_weights(&b, 5, &config);
    let chord_share = chord_weight(text, &config) / 2.0;
    let note_weight = notehead_weight(&config);

    // Column 2 (the second eighth note's own start) must now carry a share
    // of the chord's weight, not just its own notehead weight.
    assert!(
        weights[2] > note_weight + 0.01,
        "column 2 should carry a share of the chord's weight on top of its \
         own notehead weight: got {}, notehead alone would be {}",
        weights[2],
        note_weight
    );
    assert!(
        (weights[2] - chord_share.max(note_weight)).abs() < 0.01,
        "column 2's weight should be exactly max(chord share, notehead weight): \
         got {}, expected {}",
        weights[2],
        chord_share.max(note_weight)
    );
    assert!(
        (weights[0] - chord_share.max(note_weight)).abs() < 0.01,
        "column 0's weight should also be max(chord share, notehead weight): \
         got {}, expected {}",
        weights[0],
        chord_share.max(note_weight)
    );

    // Rod stays unsplit: the chord's own rod lands entirely on column 0,
    // and column 2's rod comes only from its own notehead. This measure is
    // the system's first, so its `column_rods` is prefixed by one leading
    // placeholder column absorbing the system's leading bar line (see
    // `build_measure_column_layout`'s doc comment) — skip past it to reach
    // this block's own column 0.
    let layout = build_measure_column_layout(&[b], &config);
    let leading_extra = 1; // MUSIC_START_COL - LABEL_COLS
    let chord_rod = chord_weight(text, &config) + config.chords_horizontal_padding_pt(); // column_rod's element_clearance_pt
    let note_rod = note_weight + config.notes_horizontal_padding_pt();
    assert!(
        (layout[0].column_rods[leading_extra] - chord_rod).abs() < 0.01,
        "column 0's rod should be exactly the chord's own (unsplit) rod: \
         got {}, expected {}",
        layout[0].column_rods[leading_extra],
        chord_rod
    );
    assert!(
        (layout[0].column_rods[leading_extra + 2] - note_rod).abs() < 0.01,
        "column 2's rod should come only from its own notehead, with no \
         contribution from the chord's rod: got {}, expected {}",
        layout[0].column_rods[leading_extra + 2],
        note_rod
    );
}

/// A dash-free long chord with *nothing* else in its span: its weight share
/// still lands entirely on its own single active column, since span
/// collapses to 1 when no other row's content falls inside the range. No
/// behavior change for this common case.
#[test]
fn lone_chord_with_nothing_in_its_span_keeps_its_full_weight_on_its_own_column() {
    let config = test_config();
    let text = "6m/3";
    let b = block(vec![row("Chord", vec![chord(0, text), bar_line(4)])]);

    let weights = measure_column_weights(&b, 5, &config);
    let expected = chord_weight(text, &config);

    assert!(
        (weights[0] - expected).abs() < 0.01,
        "lone chord's weight should land entirely on its own column: got {}, expected {}",
        weights[0],
        expected
    );
    for (col, &w) in weights.iter().enumerate().skip(1).take(3) {
        assert_eq!(
            w, 0.0,
            "column {col} has no content anywhere and must stay at weight 0.0"
        );
    }
}

/// Mixed-granularity regression: one beat of eighth notes (columns 0, 2)
/// next to a run of plain quarter rests (columns 4, 8, 12) in the same
/// measure. Tick 6 — inside the `0@4` rest's own span at column 4 — is
/// never anchored by any row and must stay a non-existent (zero-weight)
/// column, not become a phantom half-populated column from a naive global
/// GCD/divisor re-splitting the coarser rest region to match the finer
/// eighth-note region.
#[test]
fn rest_region_ticks_never_become_active_from_a_neighboring_finer_region() {
    let config = test_config();
    let b = block(vec![row(
        "Notes",
        vec![
            note_head(0),
            note_head(2),
            rest(4),
            rest(8),
            rest(12),
            bar_line(16),
        ],
    )]);

    let weights = measure_column_weights(&b, 17, &config);

    assert_eq!(
        weights[6], 0.0,
        "tick 6 sits inside the quarter rest at column 4's own span and must \
         never become active just because the eighth-note region nearby has a \
         finer 2-tick granularity"
    );
    assert_eq!(weights[5], 0.0, "tick 5 must likewise stay inactive");
    assert_eq!(weights[7], 0.0, "tick 7 must likewise stay inactive");

    // The rest region's own columns are unaffected by the fix (span 1 each,
    // since no other row's content falls inside a quarter rest's span).
    let rest_weight = notehead_weight(&config);
    assert!((weights[4] - rest_weight).abs() < 0.01);
    assert!((weights[8] - rest_weight).abs() < 0.01);
    assert!((weights[12] - rest_weight).abs() < 0.01);
}

/// An `Underline` beam-mark element sharing a column with its note (as
/// `compiler::beam::flush_beam_buffer` produces) doesn't get double-counted
/// as a second "next distinct column" boundary that would wrongly truncate
/// a neighboring wide element's span.
#[test]
fn beam_underline_sharing_a_column_with_its_note_does_not_truncate_a_neighboring_span() {
    let config = test_config();
    let text = "6m/3";
    let b = block(vec![
        row("Chord", vec![chord(0, text), bar_line(4)]),
        row(
            "Notes",
            vec![
                note_head(0),
                ColumnElement {
                    column: 0,
                    content: ElementContent::Underline {
                        from_column: 0,
                        to_column: 2,
                        last_head_column: 0,
                        level: 1,
                    },
                    note_id: None,
                },
                note_head(2),
                bar_line(4),
            ],
        ),
    ]);

    let weights = measure_column_weights(&b, 5, &config);
    let chord_share = chord_weight(text, &config) / 2.0;
    let note_weight = notehead_weight(&config);

    // The chord's span still reaches column 2 exactly as in the two-row
    // case above — the extra `Underline` element sharing column 0 in the
    // other row must not be mistaken for a second active-column boundary
    // that would shrink the chord's own span.
    assert!(
        (weights[2] - chord_share.max(note_weight)).abs() < 0.01,
        "beam underline sharing a column should not truncate the chord's \
         span into column 2: got {}, expected {}",
        weights[2],
        chord_share.max(note_weight)
    );
}
