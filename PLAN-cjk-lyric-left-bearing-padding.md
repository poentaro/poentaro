# Compensate CJK lyric syllables' built-in left-side bearing in flush-left padding

## Context

`GLYPH_LEFT_PADDING` (`src/font_metrics.rs`, currently `10.0` points, see the
flush-left glyph anchoring change it landed in) is a single flat gap applied
to every flush-left glyph's anchor: `x = x_start(column) + GLYPH_LEFT_PADDING`
(`ColumnGeometry::glyph_left_anchor_x`, called from
`coordinate_resolver::resolve::resolve_row_element`'s `HAlign::Center` arm).

This treats every glyph as if its own ink started exactly at its text
anchor. That's approximately true for the pinned monospace font (used for
note heads/rests/percussion hits/chord symbols/dashes/Latin lyrics) but not
for CJK lyric syllables, rendered in the pinned CJK font ("Source Han Sans
SC", see `font_metrics::directive_line_font`/`cjk_text_width`). CJK glyphs
conventionally carry their own built-in left-side bearing — visible
whitespace between the character cell's left edge and where the glyph's ink
actually begins — baked into the font by design (full-width characters are
drawn inset within their advance box). The practical effect: a CJK lyric
syllable's *visible* left gap from the bar line ends up larger than a Latin
syllable's or a note's, even though both are placed at the same computed
`x`.

## Approach

### 1. Measure the bearing — `src/font_metrics.rs`

`ttf_parser::Face` (already used here for `hmtx`-based advance widths) also
exposes `glyph_bounding_box(glyph_id) -> Option<Rect>`, the glyph's actual
ink extents in font units. `bbox.x_min` *is* the left-side bearing: the gap
between the glyph's advance-box origin and where its ink starts. Add, next
to `face_char_advance_width`:

```rust
/// Left-side bearing (in points) of one character in `face` at `font_size`
/// — the gap between the glyph's advance-box origin and where its ink
/// actually starts, per `glyph_bounding_box`. `0.0` if the glyph is
/// missing, has no outline (e.g. a space), or the font failed to parse.
fn face_glyph_left_bearing(
    face: Option<&ttf_parser::Face<'static>>,
    c: char,
    font_size: f32,
) -> f32 {
    let bearing = face.and_then(|face| {
        let glyph_id = face.glyph_index(c)?;
        let bbox = face.glyph_bounding_box(glyph_id)?;
        Some(bbox.x_min.max(0) as f32 / face.units_per_em() as f32 * font_size)
    });
    bearing.unwrap_or(0.0)
}
```

(`.max(0)` — a handful of glyphs, e.g. some diacritics, have negative
`x_min`; a glyph that overhangs left of its origin shouldn't ever *reduce*
the padding below what a plain glyph gets.)

Add the public wrapper alongside `char_advance_width`:

```rust
/// Left-side bearing (in points) of one character in the pinned CJK font
/// (see `directive_line_font`), used to compensate `GLYPH_LEFT_PADDING` for
/// CJK lyric syllables' own built-in inset — see
/// `coordinate_resolver::resolve::lyric_left_padding`.
pub(crate) fn cjk_glyph_left_bearing(c: char, font_size: f32) -> f32 {
    face_glyph_left_bearing(font_source::directive_line_font(), c, font_size)
}
```

### 2. Apply it to lyric syllables only — `src/coordinate_resolver/resolve.rs`

Only `GridContent::LyricSyllable` needs this; note heads/rests/percussion
hits/chord symbols/dashes stay on the flat `GLYPH_LEFT_PADDING` (monospace
glyphs don't have this problem, and ties/underlines/tuplet brackets never
touch lyric syllables — they key off notes, so they're unaffected either
way).

Add a helper next to `is_flush_left_glyph`:

```rust
/// The padding between a flush-left glyph's column and its anchor. For a
/// CJK lyric syllable, `GLYPH_LEFT_PADDING` is reduced by that syllable's
/// own leading character's left-side bearing (floored at `0.0`), so the
/// *visible* gap from the bar line reads the same as a Latin syllable's or
/// a note's, rather than stacking the font's own inset on top of the flat
/// padding. Every other flush-left glyph gets the flat padding unchanged.
fn flush_left_padding(content: &GridContent, cjk_font_size: f32) -> f32 {
    let GridContent::LyricSyllable { text, .. } = content else {
        return crate::font_metrics::GLYPH_LEFT_PADDING;
    };
    let Some(leading_char) = text.chars().next() else {
        return crate::font_metrics::GLYPH_LEFT_PADDING;
    };
    if !('\u{4E00}'..='\u{9FFF}').contains(&leading_char) {
        return crate::font_metrics::GLYPH_LEFT_PADDING;
    }
    let bearing = crate::font_metrics::cjk_glyph_left_bearing(leading_char, cjk_font_size);
    (crate::font_metrics::GLYPH_LEFT_PADDING - bearing).max(0.0)
}
```

The CJK-range check (`'\u{4E00}'..='\u{9FFF}'`) duplicates the heuristic
already in `layout_spacing::lyric_weight` and
`glyph_renderers_lyric::lyric_font_size`. Not this plan's job to fix, but
worth extracting a shared `font_metrics::is_cjk_text(&str) -> bool` (or
`is_cjk_char`) while touching this — three independent copies of the same
one-line check is one too many.

Call site — `resolve_row_element`'s `HAlign::Center` arm:

```rust
HAlign::Center => {
    if is_flush_left_glyph(&el.content) {
        PAGE_MARGIN
            + geometry.glyph_left_anchor_x(
                el.column as f32,
                flush_left_padding(&el.content, config.lyric_font_sizes.cjk),
            )
    } else {
        x_start + span_width * 0.5
    }
}
```

`resolve_span_marking`'s 5 call sites (`Underline`, `TieOrSlur`,
`TieOrSlurTail`, `TieOrSlurHead`, `TupletBracket`) never see
`LyricSyllable` content, so they keep using the flat
`crate::font_metrics::GLYPH_LEFT_PADDING` unchanged — no change needed
there.

### 3. Revive `RowResolveConfig.lyric_font_sizes`

The prior flush-left-anchoring change left `RowResolveConfig` with only
`note_number_width`, dropping `lyric_font_sizes`/`notes_font_size` because
nothing inside `resolve.rs` read them any more (`#![forbid(dead_code)]`
required it) — `resolve()`'s public signature and `resolve_page`'s
`_lyric_font_sizes`/`_notes_font_size` params were left in place
specifically as a hook for this follow-up. Restore the field:

```rust
#[derive(Clone, Copy)]
struct RowResolveConfig {
    note_number_width: f32,
    lyric_font_sizes: LyricFontSizes,
}
```

drop the `_` prefixes on `resolve_page`'s `lyric_font_sizes` param (now read
again) — `notes_font_size` stays unread/`_`-prefixed, since nothing in this
plan needs it.

### 4. Tests

- `src/font_metrics.rs` (inline `#[cfg(test)]` module, or wherever this
  file's existing tests for `char_advance_width`/`monospace_text_width`
  live) — `cjk_glyph_left_bearing` returns `> 0.0` for a real CJK character
  (e.g. `'漢'` or `'的'`) at a plausible font size, and `0.0` for a
  character missing from the CJK font's coverage (or use a stub/mock face
  if the real font makes this awkward to assert precisely — an inequality
  against a hand-picked character is enough; exact font-unit values would
  make the test brittle against font file changes).
- `src/coordinate_resolver/tests_lyrics.rs` — new test asserting a CJK lyric
  syllable's resolved `x` differs from (specifically, is less than) what
  `lyric_syllable_shares_the_note_head_padding_formula`'s plain-`GLYPH_LEFT_PADDING`
  formula would give, by exactly the leading character's measured bearing.
  Keep the existing Latin-text tests as regression: they must still resolve
  to flat `x_start + GLYPH_LEFT_PADDING`, confirming non-CJK syllables are
  untouched.

### 5. Verification

1. `cargo build`/`cargo test` — RowResolveConfig's restored field and the
   new helper compile and existing tests still pass.
2. Visual check (same method as the flush-left anchoring change):
   ```
   cargo run -- generate svg demo/05-lyrics.poentaro
   ```
   `05-lyrics.poentaro` is Latin-only — grep a CJK-lyrics fixture instead (or
   write a throwaway one) if none exists in `demo/`, to actually exercise
   the new branch. Confirm the CJK syllable's `x` sits visibly closer to its
   note's `x` than a Latin syllable would at the same padding, without
   crossing past the bar line to its left (still `x_start + max(0, ...)`,
   so it can't). Delete generated SVGs afterward, don't commit them.

## Critical files

- `src/font_metrics.rs`
- `src/coordinate_resolver/resolve.rs`
- `src/coordinate_resolver/tests_lyrics.rs`
