use crate::compositor::types::FontFamily;
use crate::coordinate_resolver::LyricFontSizes;

/// A lyric row's font sizes plus its click-target padding, bundled together
/// (rather than passed as two arguments) since every grid_layout function
/// that needs one needs the other too — keeps those functions under the
/// repo's max-argument-count lint.
#[derive(Clone, Copy)]
pub(crate) struct LyricSizing {
    pub(crate) font_sizes: LyricFontSizes,
    /// See `Metadata::lyric_click_target_padding_pt` / `lyric_row_height`.
    pub(crate) click_target_padding_pt: f32,
    /// Extra vertical padding in points added to a note/chord part's
    /// note-head sub-row (see `Metadata::notes.vertical_padding_pt` /
    /// `note_part_sub_row_heights`). Bundled onto this struct — despite its
    /// lyric-specific name — purely to stay under the grid_layout threading
    /// functions' max-argument-count lint: `LyricSizing` already flows
    /// everywhere a system's per-part vertical sizing needs to reach, so
    /// riding along on it avoids adding a new parameter at every hop.
    pub(crate) notes_vertical_padding_pt: f32,
}

/// Returns the 7 sub-row_heights for a Note/Chord part, in order:
/// [tuplet_bracket, arc, above_dot, note_head, half_ul, quarter_ul, below_dot]
/// The underline bands sit directly under the note head so that below-octave
/// dots of an underlined note hang beneath its underlines, per numbered-notation
/// engraving convention (see `GridContent::LowOctaveDots`).
/// `vertical_padding_pt` (see `Metadata::notes.vertical_padding_pt`) is added
/// on top of the note-head band only, additively — so a score with no
/// override (`vertical_padding_pt == 0.0`) renders at exactly the height it
/// always has.
pub(crate) fn note_part_sub_row_heights(base: f32, vertical_padding_pt: f32) -> [f32; 7] {
    [
        base * 1.0,                 // tuplet bracket (label + short bracket path)
        base * 0.30,                // tie/slur arc
        base * 0.25,                // above-octave dots
        base + vertical_padding_pt, // note head (main)
        base * 0.25,                // half-beat underline
        base * 0.25,                // quarter-beat underline
        base * 0.25,                // below-octave dots
    ]
}

/// Total height in points for a Note/Chord part's musical sub-rows, omitting
/// the `tuplet_bracket` band's height when the part has no tuplet in this
/// system — so a part without a tuplet doesn't reserve dead space for a
/// bracket it never draws (see `expand_note_part`, which mirrors this by
/// skipping the sub-row itself).
pub(crate) fn note_part_height_pt(
    base: f32,
    has_tuplet_bracket: bool,
    vertical_padding_pt: f32,
) -> f32 {
    let heights = note_part_sub_row_heights(base, vertical_padding_pt);
    if has_tuplet_bracket {
        heights.iter().sum()
    } else {
        heights[1..].iter().sum()
    }
}

/// Returns the 4 sub-row_heights for a Chord-symbol-only part, in order:
/// [arc, chord_main, half_ul, quarter_ul]
pub(crate) fn chord_part_sub_row_heights(base: f32) -> [f32; 4] {
    [
        base * 0.30, // tie/slur arc
        base * 0.75, // chord symbol (main)
        base * 0.25, // half-beat underline
        base * 0.25, // quarter-beat underline
    ]
}

/// Height for a lyric verse row (and its click-target rect — see
/// `coordinate_resolver::highlights::resolve_lyric_click_target`).
/// `row_height * 1.5` is the row's normal vertical rhythm — unchanged from
/// before, so a row with no CJK text and no `lyrics_font_size` override
/// renders at exactly the height it always has. The other side of the
/// `max` is a floor sized to actually contain the glyph: the pinned lyric
/// font's real ascender+descender span at `font_size`, measured at runtime
/// via `ttf_parser` (`font_metrics::text_vertical_extent`) rather than a
/// hardcoded ratio, plus `click_target_padding` (points, additive rather
/// than multiplicative so it doesn't overstate padding at large font sizes
/// the way a ratio would — see `Metadata::lyric_click_target_padding_pt`)
/// for a click-target margin — only the larger of the two applies, so this
/// floor only kicks in once `font_size` (the largest resolved size — Latin
/// or CJK, see `RenderConfig::lyric_font_size`/`lyric_cjk_font_size` —
/// among the row's own syllables) grows large enough, relative to
/// `row_height`, to actually risk overflowing the row's normal height.
pub(crate) fn lyric_row_height(row_height: f32, font_size: f32, click_target_padding: f32) -> f32 {
    (row_height * 1.5).max(
        crate::font_metrics::text_vertical_extent(FontFamily::Serif, font_size)
            + click_target_padding,
    )
}

pub(crate) fn decoration_row_height(base: f32) -> f32 {
    base * 1.5
}

pub(crate) fn separator_row_height() -> f32 {
    4.0
}

/// One row of blank vertical space: between the title/subtitle/author block
/// and what follows it, and above the sequence line.
pub(crate) fn header_gap_row_height(base: f32) -> f32 {
    base
}

/// Height of a header row holding one line of text: one em (the text's own
/// `font_size`) plus `vertical_padding_pt` above and below, so `0` padding
/// means a row exactly one font size tall, whatever the font's own ascender,
/// descender and line gap. The row grows and shrinks with `font_size` and
/// never lets the text spill into the rows around it.
pub(crate) fn text_row_height(font_size: f32, vertical_padding_pt: f32) -> f32 {
    font_size + 2.0 * vertical_padding_pt
}
