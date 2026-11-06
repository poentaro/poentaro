use super::{block_column_width, LABEL_COLS, MUSIC_START_COL};
use crate::compiler::types::{ColumnElement, ElementContent, MeasureBlock};
use crate::grid_layout::layout::{directive_line_rod_width, directive_line_should_emit};
use crate::grid_layout::types::MeasureColumnLayout;
use crate::render_config::RenderConfig;
use std::collections::BTreeSet;

#[path = "layout_spacing_weights.rs"]
mod weights;
use weights::{
    accidental_extra_weight, column_weight, measure_note_weight, multi_measure_rest_weight,
    THIN_MARK_WEIGHT,
};

/// Minimum floor a measure's column-region gets, in points, regardless of its
/// spacing weight or its own columns' rods — a degenerate-case safety net
/// (e.g. an empty measure) now that [`MeasureColumnLayout::rod_pt`] is
/// normally content-derived (see that field's doc comment).
pub(crate) const MIN_MEASURE_WIDTH_PT: f32 = 24.0;

/// Per-content-type clearance (in points) added on top of a note-ish
/// column's own [`column_weight`] to form its hard-minimum rod (see
/// [`column_rod`]) — the "spring and rod" model's rod is real content width
/// plus a little breathing room, not just the bare glyph width, so a column
/// never renders flush against whatever follows it (see the `column_geometry`
/// docs).
///
/// This has to match the padding `coordinate_resolver::resolve::flush_left_padding`
/// actually uses for that content type (`Metadata::notes_horizontal_padding_pt`/
/// `chords_horizontal_padding_pt`/`lyrics_horizontal_padding_pt`/
/// `note_dash_horizontal_padding_pt`), not some smaller hand-tuned value:
/// every flush-left glyph (note head, rest, chord symbol, note dash, lyric
/// syllable) is drawn starting that padding (minus its own left-side
/// bearing) past the column's left edge — `ColumnGeometry::glyph_left_anchor_x`
/// — so a rod smaller than that leading offset plus the glyph's own width
/// leaves the column's own rod too small to contain what actually gets
/// drawn in it. That's invisible most of the time (slack usually pads
/// columns well past their rod, and a glyph bleeding into a neighboring
/// *note* column reads as normal spacing), but becomes visible
/// ink-on-stroke overlap when the column is this tight *and* immediately
/// followed by a `BarLine`, whose own rod ([`BARLINE_MIN_WIDTH_PT`]) is far
/// smaller than a glyph needs to safely finish inside it.
///
/// `BarLine`/`MultiMeasureRest`/`Underline` never reach this — `column_rod`
/// special-cases them before falling into the `_` arm that calls this.
fn element_clearance_pt(content: &ElementContent, config: &RenderConfig) -> f32 {
    match content {
        ElementContent::ChordSymbol { .. } => config.chords_horizontal_padding_pt(),
        ElementContent::NoteDash { .. } => config.note_dash_horizontal_padding_pt(),
        ElementContent::Lyric { .. } | ElementContent::LyricLine { .. } => {
            config.lyrics_horizontal_padding_pt()
        }
        _ => config.notes_horizontal_padding_pt(),
    }
}

/// Hard-minimum rod (in points) for a `BarLine` column. Unlike note-ish
/// columns, a bar line's [`THIN_MARK_WEIGHT`] is an arbitrary relative ratio
/// rather than a measured width (see that constant's doc comment), so it
/// can't double as a rod the way [`column_weight`] does elsewhere — this is
/// a small dedicated floor instead, just enough for the drawn stroke plus a
/// sliver of clearance on each side.
const BARLINE_MIN_WIDTH_PT: f32 = 4.0;

/// Hard-minimum rod (in points) for a column carrying `content` — the floor
/// [`column_geometry`](super::geometry) gives that column regardless of how
/// tightly its system is packed. Note-ish columns get their real rendered
/// width ([`column_weight`]) plus a little clearance
/// ([`element_clearance_pt`]); a `BarLine` gets its own small dedicated floor
/// ([`BARLINE_MIN_WIDTH_PT`]) since [`THIN_MARK_WEIGHT`] isn't a real width;
/// `Underline` needs no floor of its own, matching its zero spring weight.
/// `MultiMeasureRest` never actually reaches this per-element match (its row
/// is always caught by `measure_column_sizes`'s own early-return special
/// case, since a `MultiMeasureRest` row never mixes with other content), but
/// keeps a defensive `0.0` here rather than falling into the `_` arm's
/// [`column_weight`], which has no case of its own for it either.
fn column_rod(content: &ElementContent, config: &RenderConfig) -> f32 {
    match content {
        ElementContent::BarLine => BARLINE_MIN_WIDTH_PT,
        ElementContent::MultiMeasureRest { .. } | ElementContent::Underline { .. } => 0.0,
        // The diagram above a chord symbol starts where the symbol does, so
        // the wider of the two decides how much room the column needs.
        ElementContent::ChordSymbol {
            guitar_voicing: Some(voicing),
            ..
        } => {
            column_weight(content, config)
                .max(crate::font_metrics::guitar_diagram::width_pt(voicing))
                + element_clearance_pt(content, config)
        }
        _ => column_weight(content, config) + element_clearance_pt(content, config),
    }
}

/// A column's spring-and-rod sizing: `weight` is its spring (relative share
/// of any slack left after every column's rod is satisfied — see
/// [`column_weight`]), `rod_pt` is its hard-minimum floor in points (see
/// [`column_rod`]). See the `column_geometry` docs.
/// `accidental_lead_pt` is the column's accidental lead: the widest
/// sharp/flat glyph any part's note head draws ahead of its digit in this
/// column (see [`accidental_lead`]).
#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) struct ColumnSizing {
    pub weight: f32,
    pub rod_pt: f32,
    pub accidental_lead_pt: f32,
}

/// Room (in points) `content` needs ahead of its own digit for a sharp/flat
/// accidental — `render_note_head` draws the accidental immediately before
/// the digit, so every flush-left glyph in the column is pushed right by the
/// column's widest such lead (see `ColumnGeometry::glyph_left_anchor_x`),
/// keeping every part's digits vertically aligned on the same beat. A chord
/// symbol whose root carries a sharp/flat (`♯1m`) leads with it the same
/// way (see `render_chord_symbol`). `0.0` for anything else, including a
/// letter name, whose accidental trails its letter (`F♯`).
fn accidental_lead(content: &ElementContent, config: &RenderConfig) -> f32 {
    match content {
        // The guitar chord view's label follows the digit (`♯4 (F♯)`), so
        // its digit still leads with the accidental.
        ElementContent::NoteHead {
            accidental,
            head_label,
            ..
        } if head_label
            .as_ref()
            .is_none_or(|label| !label.replaces_degree()) =>
        {
            accidental_extra_weight(accidental, config)
        }
        ElementContent::ChordSymbol { text, .. } => {
            crate::font_metrics::chord_leading_accidental_width_for_family(
                config.glyph_font_families.chords,
                text,
                config.chords_font_size(),
            )
        }
        _ => 0.0,
    }
}

/// Per-column [`ColumnSizing`] across `block`'s `col_count` columns — the
/// combined weight/rod computation [`measure_column_weights`] and
/// [`build_measure_column_layout`] both draw from, so the per-row
/// max-across-parts logic lives in one place. See
/// [`measure_column_weights`]'s doc comment for the full column-splitting
/// rationale (multi-measure-rest handling, tuplet multiplier-invariance,
/// etc.) — this differs from it only in also carrying each column's rod.
fn measure_column_sizes(
    block: &MeasureBlock,
    col_count: u32,
    config: &RenderConfig,
) -> Vec<ColumnSizing> {
    let multi_measure_rest_count = block.rows.iter().find_map(|row| {
        row.elements.iter().find_map(|e| match &e.content {
            ElementContent::MultiMeasureRest { count } => Some(*count),
            _ => None,
        })
    });
    if let Some(count) = multi_measure_rest_count {
        // Mirrors `measure_note_weight`'s own special case: the whole span's
        // real-point-width need ([`weights::multi_measure_rest_weight`],
        // which grows with the count label's own rendered width) is spread
        // evenly across the bar's columns for `column_weights`, and
        // concentrated on the span's own start column (column `0`) for its
        // rod — matching `column_rod`'s "rod sits at the element's own start
        // tick" rule elsewhere in this function — so the block's guaranteed
        // minimum width actually grows to fit its label instead of staying
        // pinned at `MIN_MEASURE_WIDTH_PT` regardless of digit count.
        let total_weight = multi_measure_rest_weight(count, config);
        let bar_column_count = col_count.saturating_sub(1).max(1);
        let per_bar_column_weight = total_weight / bar_column_count as f32;
        return (0..col_count)
            .map(|col| ColumnSizing {
                weight: if col + 1 == col_count {
                    THIN_MARK_WEIGHT
                } else {
                    per_bar_column_weight
                },
                rod_pt: if col == 0 { total_weight } else { 0.0 },
                accidental_lead_pt: 0.0,
            })
            .collect();
    }
    // The set of ticks any row actually anchors an element to — an
    // empirical scan of what's in use, not a uniform GCD/divisor of the
    // measure's durations (see this function's doc comment and the
    // `column_geometry` docs for why a global divisor would wrongly
    // re-split a coarser region that sits next to a finer one).
    let active_columns: BTreeSet<u32> = block
        .rows
        .iter()
        .flat_map(|row| row.elements.iter().map(|e| e.column))
        .collect();

    let mut weight_by_col = vec![0.0f32; col_count as usize];
    // A column's rod is its widest element's rod *excluding* that element's
    // own accidental, plus the column's widest accidental lead: the lead
    // pushes every flush-left glyph in the column right (not just the
    // accidental's own note head), so e.g. a wide chord symbol sharing the
    // column with a sharp note needs the lead *and* its own width.
    let mut bare_rod_by_col = vec![0.0f32; col_count as usize];
    let mut lead_by_col = vec![0.0f32; col_count as usize];
    for row in &block.rows {
        let mut elements: Vec<&ColumnElement> = row.elements.iter().collect();
        elements.sort_by_key(|e| e.column);
        let mut distinct_columns: Vec<u32> = elements.iter().map(|e| e.column).collect();
        distinct_columns.dedup();
        for e in &elements {
            // Rod stays concentrated entirely on the element's own start
            // tick — a wide glyph genuinely needs that much *contiguous*
            // space before anything else can safely start (see the `column_geometry` docs).
            let lead = accidental_lead(&e.content, config);
            if let Some(rod) = bare_rod_by_col.get_mut(e.column as usize) {
                *rod = rod.max(column_rod(&e.content, config) - lead);
            }
            if let Some(max_lead) = lead_by_col.get_mut(e.column as usize) {
                *max_lead = max_lead.max(lead);
            }

            // Weight, by contrast, is shared across every *active* column
            // in this element's span — up to (but excluding) the next
            // distinct column this same row anchors something to, or
            // `col_count` if it's the row's last.
            let next_distinct_column = distinct_columns
                .iter()
                .find(|&&c| c > e.column)
                .copied()
                .unwrap_or(col_count);
            let span_columns: Vec<u32> = active_columns
                .range(e.column..next_distinct_column)
                .copied()
                .collect();
            let span = span_columns.len().max(1) as f32;
            let share = column_weight(&e.content, config) / span;
            for col in span_columns {
                if let Some(weight) = weight_by_col.get_mut(col as usize) {
                    *weight = weight.max(share);
                }
            }
        }
    }

    (0..col_count)
        .map(|col| {
            let accidental_lead_pt = lead_by_col.get(col as usize).copied().unwrap_or(0.0);
            ColumnSizing {
                weight: weight_by_col.get(col as usize).copied().unwrap_or(0.0),
                rod_pt: bare_rod_by_col.get(col as usize).copied().unwrap_or(0.0)
                    + accidental_lead_pt,
                accidental_lead_pt,
            }
        })
        .collect()
}

/// Per-column width weight across `block`'s `col_count` columns, used to
/// split a measure's total pixel width unevenly among its own columns (e.g.
/// a notehead column wider than the dash column following it). Each
/// column's weight is the max [`column_weight`] of any row's element at
/// that column, so a column isn't under-weighted just because one part
/// sustains a dash there while another part has a fresh note. A collapsed
/// `MultiMeasureRest` row instead gets [`multi_measure_rest_weight`]'s total
/// spread evenly across every column of its own span, keeping its previous
/// even (undifferentiated) sizing while still summing to a real point width;
/// its trailing `BarLine` column still gets the usual thin weight.
///
/// `col_count` is not divided by `block`'s tuplet `resolution_multiplier` before use here.
/// A multiplier > 1 inflates `col_count` (more raw grid columns stand in for the same real
/// duration — see the `crate::tuplet` docs), but every one of those extra columns
/// that has no element gets weight `0.0` from `column_weight` and so contributes nothing to
/// `column_geometry`'s `column_weight_sum` split; the columns that *do* carry weight
/// (notehead/dash/bar-line) are exactly as many as an equivalent non-tuplet measure would
/// have, so the resulting proportional column widths already come out multiplier-invariant
/// without any explicit division. Confirmed empirically: a single-measure triplet
/// (`3:{1_1_1_} 2_ 3_ 4_ 5_ 6_`, multiplier 3) renders its 8 notes at uniform column
/// spacing, identical in kind to a non-tuplet measure of 8 same-weight notes.
#[cfg(test)]
pub(crate) fn measure_column_weights(
    block: &MeasureBlock,
    col_count: u32,
    config: &RenderConfig,
) -> Vec<f32> {
    measure_column_sizes(block, col_count, config)
        .into_iter()
        .map(|s| s.weight)
        .collect()
}

/// Per-measure column layout (position, column count, weights) for every
/// measure in `system`, shared by every row of the system (the decoration
/// row and every part/lyric row built by `expand_system_to_rows`) so they
/// all resolve the same proportional `ColumnGeometry`.
///
/// The first measure's `start_col`/`col_count` are widened by one column to
/// absorb the system's leading bar-line column (at `LABEL_COLS`, ahead of
/// `MUSIC_START_COL` where the first measure's own elements begin) — that
/// column belongs to no measure block, but still needs to fall inside a
/// `ColumnSegment` so `x_start(LABEL_COLS)` keeps landing exactly on
/// `label_width_pt` (where the leading bar line is drawn) regardless of the
/// first measure's weight. It's given `THIN_MARK_WEIGHT` for the intra-
/// measure weight split (like any other bar line) but rod `0.0` (see
/// `build_measure_column_layout`'s inline comment) — like `weight`, which
/// is purely `measure_note_weight`, neither ever counts the leading column.
pub(crate) fn build_measure_column_layout(
    system: &[MeasureBlock],
    config: &RenderConfig,
) -> Vec<MeasureColumnLayout> {
    let mut start_col = MUSIC_START_COL;
    system
        .iter()
        .enumerate()
        .map(|(idx, block)| {
            let col_count = block_column_width(block);
            let leading_extra = if idx == 0 {
                MUSIC_START_COL - LABEL_COLS
            } else {
                0
            };
            let sizes = measure_column_sizes(block, col_count, config);
            let mut column_weights = vec![THIN_MARK_WEIGHT; leading_extra as usize];
            column_weights.extend(sizes.iter().map(|s| s.weight));
            // The leading placeholder column (like `weight` above) never
            // gets a rod of its own — it still takes a proportional slice
            // of whatever slack the measure ends up with, via its
            // `column_weights` entry, but doesn't inflate this measure's
            // `rod_pt` relative to a measure with no leading column. That
            // keeps two equal-density measures at equal width regardless of
            // which one opens the system (see
            // `equal_density_measures_render_at_equal_width`).
            let mut column_rods = vec![0.0; leading_extra as usize];
            column_rods.extend(sizes.iter().map(|s| s.rod_pt));
            let content_rod: f32 = column_rods.iter().sum();
            let mut column_accidental_leads = vec![0.0; leading_extra as usize];
            column_accidental_leads.extend(sizes.iter().map(|s| s.accidental_lead_pt));
            let directive_width_pt = block
                .decorations
                .first()
                .filter(|dec| directive_line_should_emit(idx, dec))
                .map(|dec| {
                    directive_line_rod_width(
                        dec,
                        config.measure_number_font_size as f32,
                        config.section_label_font_size as f32,
                        config.section_label_bold,
                    )
                })
                .unwrap_or(0.0);
            // The directive line draws starting at this block's own leading
            // anchor (the previous block's trailing bar line, or the
            // system's leading placeholder column for the first block — see
            // `make_decoration_row`) and must finish before this block's OWN
            // trailing bar-line column starts, since that's where the
            // *next* block's directive begins drawing. The rescale below
            // proportionally inflates every column's rod — including the
            // trailing bar line's own — so solve in closed form for how
            // much this block's rod needs to grow so the directive still
            // fits after that rescale: with `C = content_rod` and
            // `B = BARLINE_MIN_WIDTH_PT`, the bar line's post-scale rod is
            // `B * rod_pt / C`, leaving `rod_pt * (C - B) / C` of non-bar-
            // line width for the directive — setting that `>= D` gives
            // `rod_pt >= D * C / (C - B)`.
            let directive_target_rod_pt =
                if directive_width_pt > 0.0 && content_rod > BARLINE_MIN_WIDTH_PT {
                    directive_width_pt * content_rod / (content_rod - BARLINE_MIN_WIDTH_PT)
                } else {
                    directive_width_pt
                };
            let rod_pt = MIN_MEASURE_WIDTH_PT
                .max(content_rod)
                .max(directive_target_rod_pt);
            if rod_pt > content_rod && content_rod > 0.0 {
                // `MIN_MEASURE_WIDTH_PT`'s degenerate-case floor (e.g. an
                // almost-empty measure) pushed `rod_pt` above what this
                // measure's own columns actually need. Scale every column's
                // rod up proportionally so `column_rods` still sums to
                // exactly `rod_pt` — preserving the measure's internal
                // proportions while keeping `column_geometry`'s segments
                // gapless (see `proportional_widths_sum_to_full_usable_music_width`).
                let scale = rod_pt / content_rod;
                for r in &mut column_rods {
                    *r *= scale;
                }
            }
            let layout = MeasureColumnLayout {
                start_col: start_col - leading_extra,
                col_count: col_count + leading_extra,
                weight: measure_note_weight(block, config),
                column_weights,
                column_rods,
                column_accidental_leads,
                rod_pt,
            };
            start_col += col_count;
            layout
        })
        .collect()
}
