//! The "spring" half of `layout_spacing`'s spring-and-rod model: each
//! column/measure's proportional width weight, split out of
//! `layout_spacing.rs` to keep that file under the max line-count lint. See
//! **Rod and spring** in `ARCHITECTURE.md`.

use crate::ast::parsed::{Accidental, NumberedPitch};
use crate::compiler::types::{
    ElementContent, MeasureBlock, NoteHeadLabel, MULTI_MEASURE_REST_WIDTH,
};
use crate::compositor::types::FontFamily;
use crate::font_metrics;
use crate::render_config::RenderConfig;

/// Relative width weight of a `BarLine` column — just a thin mark, so it
/// gets much less than a fresh note. Elements that don't occupy their own
/// column-worth of ink (`Underline`) or that are handled separately
/// (`MultiMeasureRest`) contribute nothing here. Kept as an arbitrary flat
/// ratio rather than a measured width: a bar line is a drawn stroke, not a
/// glyph, so "actual rendered width" doesn't apply to it.
pub(super) const THIN_MARK_WEIGHT: f32 = 0.25;

/// Real advance width (in points) of one notehead/rest/percussion-hit digit
/// glyph at `config`'s notes font size and `font_family` — the unit every
/// other weight below is expressed in relative terms of, since notehead/
/// rest/percussion glyphs all render as a single character in that same
/// font (see `render_note_head`/`render_rest`/`render_percussion_hit` in
/// `glyph_renderers.rs`), so any one of their digits/`0`/`x` characters
/// measures the same.
fn note_glyph_weight(config: &RenderConfig) -> f32 {
    font_metrics::advance_width_for_family(
        config.glyph_font_families.notes,
        '0',
        config.notes_font_size(),
    )
}

/// Extra weight given to a dotted `NoteHead`/`Rest`/`NoteDash`/`ChordSymbol`
/// column to make room for its augmentation dot(s) — `render_note_head`/
/// `render_rest`/`render_note_dash`/`render_chord_symbol` (`glyph_renderers.rs`,
/// `glyph_renderers_note_dash.rs`) draw the dot(s) as circles positioned
/// immediately after the glyph's own rendered width (see
/// `glyph_renderers::augmentation_dot_glyphs`), each one given exactly one
/// [`font_metrics::augmentation_dot_suffix`] character's worth of advance
/// width — so the extra column width needed is exactly that measured text
/// width, with no clearance or reach-vs-base-glyph reconciliation needed.
/// `family` must match the glyph the dot(s) are drawn after
/// (`config.glyph_font_families.notes` for a `NoteHead`/`Rest`, `.chords`
/// for a `ChordSymbol`, `.note_dash` for a `NoteDash`), same as the font it
/// actually renders in.
fn dot_extra_weight(
    dotted: bool,
    double_dotted: bool,
    dot_font_size: f32,
    family: FontFamily,
) -> f32 {
    font_metrics::text_width_for_family(
        family,
        font_metrics::augmentation_dot_suffix(dotted, double_dotted),
        dot_font_size,
    )
}

/// Extra weight given to a `NoteHead` column carrying a sharp/flat
/// accidental, to make room for the `♯`/`♭` glyph — `render_note_head`
/// (`glyph_renderers.rs`) draws it immediately *before* the note digit, at
/// the same font size/family as the digit — so the extra width needed is
/// exactly the accidental's own real rendered width. `Natural` renders no
/// glyph, so it needs no extra weight either. Also the column's accidental
/// lead (see `layout_spacing::measure_column_sizes`), the room every
/// flush-left glyph in the column is pushed right by so the accidental has
/// somewhere to draw.
pub(super) fn accidental_extra_weight(accidental: &Accidental, config: &RenderConfig) -> f32 {
    font_metrics::accidental_width_for_family(
        config.glyph_font_families.notes,
        accidental,
        config.notes_font_size(),
    )
}

/// Real advance width (in points) of the note-dash glyph (`—`), measured at
/// `config`'s note-dash font size/family, matching what `render_note_dash`
/// actually draws.
fn dash_weight(config: &RenderConfig) -> f32 {
    font_metrics::advance_width_for_family(
        config.glyph_font_families.note_dash,
        '\u{2014}',
        config.note_dash_font_size(),
    )
}

/// Width weight for a chord symbol's own glyph, measured from its real
/// rendered width in `config`'s chords font family (see
/// `render_chord_symbol`). Floored at [`note_glyph_weight`] so a
/// single-character chord (e.g. `1`) keeps the same weight as any other
/// note-like event.
fn chord_symbol_weight(symbol: &str, config: &RenderConfig) -> f32 {
    font_metrics::text_width_for_family(
        config.glyph_font_families.chords,
        symbol,
        config.chords_font_size(),
    )
    .max(note_glyph_weight(config))
}

/// Width weight for a lyric syllable, measured from its real rendered width
/// — the CJK font/size if the syllable contains a CJK codepoint, the
/// monospace font/size otherwise — mirroring the same check `render_lyric`
/// already does to pick a font size.
fn lyric_weight(text: &str, config: &RenderConfig) -> f32 {
    if text.chars().any(font_metrics::is_cjk_char) {
        font_metrics::cjk_text_width(text, config.lyric_cjk_font_size())
    } else {
        font_metrics::monospace_text_width(text, config.lyric_font_size())
    }
}

/// Real rendered width (in points) a merged rest run's block needs: at
/// least enough to hold its count label (`render_multi_measure_rest`'s
/// centered text, measured in the same monospace font/size as
/// [`note_glyph_weight`] — see `render_multi_measure_rest` in
/// `glyph_renderers.rs`), and never less than `MULTI_MEASURE_REST_WIDTH`
/// note glyphs' worth of bar, so a short run (single/double-digit count)
/// keeps its previous width. Also reserves `config.notes_horizontal_padding_pt()`
/// on both ends — the same clearance every other note-ish column keeps
/// before whatever follows it — so the bar's own end ticks don't end up
/// flush against the enclosing measure dividers; `resolve_multi_measure_rest`
/// (`coordinate_resolver::resolve`) insets the drawn bar by that same amount
/// so the two can't drift apart. Unlike the flat, unitless weight this
/// replaces, every term here is a real point width — matching every other
/// element's weight in this module — so a merged rest block sharing a
/// system with real-point-weighted measures no longer gets squeezed down to
/// a fraction of what its own label needs to render without overflowing.
pub(super) fn multi_measure_rest_weight(count: usize, config: &RenderConfig) -> f32 {
    let label_width =
        font_metrics::monospace_text_width(&count.to_string(), config.notes_font_size());
    let bar_floor = note_glyph_weight(config) * MULTI_MEASURE_REST_WIDTH as f32;
    label_width.max(bar_floor) + config.notes_horizontal_padding_pt() * 2.0
}

/// Width weight for a note head's own glyph(s): one digit plus its leading
/// sharp/flat (see [`accidental_extra_weight`]), or — in the letter-name
/// view — its letter name's real rendered width, accidental included, since
/// `render_note_head` draws the letter name as one text run in the notes
/// font in place of the digit. In the guitar chord view, the leading
/// sharp/flat plus the real width of the digit followed by its letter name
/// (`1 (C)`), drawn as the same one text run.
fn note_head_weight(
    pitch: &NumberedPitch,
    accidental: &Accidental,
    head_label: Option<&NoteHeadLabel>,
    config: &RenderConfig,
) -> f32 {
    let run_width = |head_label: &NoteHeadLabel| {
        font_metrics::text_width_for_family(
            config.glyph_font_families.notes,
            &head_label.run_text(pitch.to_digit()),
            config.notes_font_size(),
        )
    };
    match head_label {
        Some(head_label) if head_label.replaces_degree() => run_width(head_label),
        Some(head_label) => run_width(head_label) + accidental_extra_weight(accidental, config),
        None => note_glyph_weight(config) + accidental_extra_weight(accidental, config),
    }
}

pub(super) fn column_weight(content: &ElementContent, config: &RenderConfig) -> f32 {
    match content {
        ElementContent::NoteHead {
            pitch,
            accidental,
            dotted,
            double_dotted,
            head_label,
            ..
        } => {
            note_head_weight(pitch, accidental, head_label.as_ref(), config)
                + dot_extra_weight(
                    *dotted,
                    *double_dotted,
                    config.notes_font_size(),
                    config.glyph_font_families.notes,
                )
        }
        ElementContent::Rest {
            dotted,
            double_dotted,
            ..
        } => {
            note_glyph_weight(config)
                + dot_extra_weight(
                    *dotted,
                    *double_dotted,
                    config.notes_font_size(),
                    config.glyph_font_families.notes,
                )
        }
        ElementContent::ChordSymbol {
            text,
            dotted,
            double_dotted,
            ..
        } => {
            chord_symbol_weight(text, config)
                + dot_extra_weight(
                    *dotted,
                    *double_dotted,
                    config.chords_font_size(),
                    config.glyph_font_families.chords,
                )
        }
        ElementContent::PercussionHit => note_glyph_weight(config),
        ElementContent::NoteDash {
            dotted,
            double_dotted,
        } => {
            dash_weight(config)
                + dot_extra_weight(
                    *dotted,
                    *double_dotted,
                    config.note_dash_font_size(),
                    config.glyph_font_families.note_dash,
                )
        }
        ElementContent::Lyric { text, .. } => lyric_weight(text, config),
        // A `LyricLine` spans the whole measure via `column_span` rather than
        // occupying one grid column, so it contributes no per-column weight here;
        // its width is instead folded into the measure's total via `measure_note_weight`.
        ElementContent::LyricLine { .. } => 0.0,
        ElementContent::BarLine => THIN_MARK_WEIGHT,
        ElementContent::MultiMeasureRest { .. } | ElementContent::Underline { .. } => 0.0,
    }
}

/// How much horizontal room `block` should get relative to *other measures*
/// in its system — not to be confused with `measure_column_weights`, which
/// splits width *within* one measure. Only counts real note-starting
/// elements (notehead, rest, percussion hit, chord symbol); dashes and bar
/// lines don't contribute, so a measure of quarter notes gets roughly
/// double the aggregate weight of a measure of half notes spanning the same
/// duration (4 fresh notes vs. 2 notes + 2 dashes) — the whole point being
/// that dash-extended measures shouldn't out-compete note-dense measures for
/// width just because a dash happens to occupy its own column. A chord
/// symbol contributes its own [`chord_symbol_weight`] (its real rendered
/// width) rather than a flat [`note_glyph_weight`], so a slash chord with a
/// bass note (e.g. `2m/5`) out-competes a bare-degree chord (e.g. `1`) for
/// width. Weight is the max (not sum) across the block's part rows, so a
/// measure isn't penalized for having many parts, only sized for its
/// densest one. A collapsed `MultiMeasureRest` row gets
/// [`multi_measure_rest_weight`] instead of being counted as one note, so
/// its own weight stays a real point width comparable to every other
/// measure's — including growing with its count label's own rendered width
/// — rather than an arbitrary flat number. Clamped to a minimum of one note
/// glyph's width
/// ([`note_glyph_weight`]) so an empty/rest-only measure never collapses to
/// zero weight, and so two equal-density measures always compare equal
/// regardless of which one happens to open the system (see
/// `build_measure_column_layout`'s leading bar-line column, which never
/// contributes here).
///
/// Deliberately **not** divided by the measure's tuplet `resolution_multiplier` (see
/// **Tuplet** in `ARCHITECTURE.md`): this counts *written note occurrences*, not raw grid
/// columns, and a tuplet's rescaled duration never changes how many `NoteHead`/`Rest`/
/// `PercussionHit` elements a measure has — 3 triplet-eighth notes still count as 3, the
/// same as 3 plain notes elsewhere, matching this function's existing note-count (not
/// note-duration) philosophy. A tuplet measure's grid column *count* is inflated by its
/// multiplier (see `block_column_width`), but that inflation never reaches a raw pixel
/// width anywhere in this module — every consumer of `col_count` below only ever indexes
/// or iterates it, then folds back down to a proportional (multiplier-invariant) split via
/// `measure_column_weights`/`column_weight`.
pub(super) fn measure_note_weight(block: &MeasureBlock, config: &RenderConfig) -> f32 {
    block
        .rows
        .iter()
        .map(|row| {
            let multi_measure_rest_count = row.elements.iter().find_map(|e| match &e.content {
                ElementContent::MultiMeasureRest { count } => Some(*count),
                _ => None,
            });
            if let Some(count) = multi_measure_rest_count {
                multi_measure_rest_weight(count, config)
            } else {
                row.elements
                    .iter()
                    .map(|e| match &e.content {
                        ElementContent::NoteHead {
                            pitch,
                            accidental,
                            head_label,
                            ..
                        } => note_head_weight(pitch, accidental, head_label.as_ref(), config),
                        ElementContent::Rest { .. } | ElementContent::PercussionHit => {
                            note_glyph_weight(config)
                        }
                        ElementContent::ChordSymbol { text, .. } => {
                            chord_symbol_weight(text, config)
                        }
                        ElementContent::LyricLine { text, .. } => lyric_weight(text, config),
                        _ => 0.0,
                    })
                    .sum::<f32>()
            }
        })
        .fold(note_glyph_weight(config), f32::max)
}
