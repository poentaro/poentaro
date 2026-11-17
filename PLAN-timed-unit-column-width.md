# Findings: timed-unit column width has no real minimum, and can overlap

## Background

While adding sus2/sus4 chord support (`TriadQuality::Sus2`/`Sus4`), we found
that wide chord symbols (`1sus4`, `1sus2`, and to a lesser extent existing
slash chords like `6m/5`) can render with their text bleeding into the next
column. This file captures what causes it, so it can be fixed in a
follow-up session without re-deriving the investigation.

**This is not sus-chord-specific.** Any wide `ElementContent` (long chord
symbol, in principle even a note with an accidental at a small
`note_number_width`) is susceptible. Sus symbols just make it easy to
trigger, since `sus2`/`sus4` (5 chars) is 2-3x wider than a typical chord
symbol (`1`, `6m`, `1⁷` — 1-2 chars).

## How column width is actually computed today

1. **`chord_symbol_weight`** (`src/grid_layout/layout_spacing.rs:81-84`)
   measures a chord symbol's *true* rendered width via
   `font_metrics::monospace_text_width`, floored at one note-glyph width.
   This is real, correct glyph measurement — not the bug.

   Unlike accidentals (which get an explicit reserve —
   `accidental_extra_weight`, `ACCIDENTAL_RIGHT_PADDING_RATIO`, see
   `layout_spacing.rs:42-66`), a chord symbol gets **no extra padding/gap**
   beyond its own measured width. There is no equivalent of "reach beyond
   what the glyph itself covers" for chord symbols.

2. **That "weight" is never used as an absolute pixel width.** It's only a
   relative share in a justify-to-fit split:
   - `column_geometry.rs:39-46` — a measure's total pixel width is
     `MIN_MEASURE_WIDTH_PT + (usable_music_width - n * MIN_MEASURE_WIDTH_PT)
     * measure.weight / total_weight`, i.e. a **fixed** page-width budget
     (`usable_music_width`) split proportionally across measures by weight
     share.
   - Within a measure, `col_width = measure_width * column_weight / column_weight_sum`
     — same proportional-split logic, one level down.
   - Net effect: a column's real pixel width equals its "true" glyph width
     **only when** the system's total weight happens to equal the system's
     available width. Any denser-than-assumed system compresses every
     column below its glyph's real width; any sparser system stretches
     every column wider than its glyph needs (visible as loose spacing on
     sparse pages).

3. **Line breaking is count-based, not width-aware.**
   `pack_into_systems` (`src/grid_layout/layout_systems.rs:81-108`) puts up
   to `max_measures_per_system` measures on one system row purely by
   *count* (plus a row-identity check), with **zero awareness of how wide
   the accumulated content actually is**. A row of measures dense with
   `sus4`/`sus2` symbols still gets forced into the same fixed page width
   as a row of plain `1`/`6m` chords — real content width has no say in
   where the line wraps.

Combined, (2) + (3) mean: there is no guarantee anywhere in the pipeline
that a column's assigned pixel width is `>= ` the text it needs to draw.
The system always fills exactly `usable_music_width` (a "justify" layout),
and the only thing preventing collapse is `MIN_MEASURE_WIDTH_PT` (a
*measure*-level floor, not a per-column one, and not content-aware either).

## Why this is "not perfect"

- No minimum padding/gap is reserved after a chord symbol (or, more
  generally, after any timed unit) — nothing stops the next column's
  content from starting exactly where this one's glyph ends, so any
  measurement drift (font substitution in the actual SVG viewer vs. the
  pinned font used for measurement, rounding, etc.) immediately shows as
  visible overlap, not just tight spacing.
- The bigger issue is structural: because `pack_into_systems` wraps by
  measure *count* rather than accumulated *width*, the proportional squeeze
  in `column_geometry.rs` can push a column's real pixel width arbitrarily
  far below its glyph's true width whenever a system's total content is
  denser than what the fixed page width was implicitly sized for. Nothing
  currently detects or prevents this — it fails silently as visual overlap,
  not as a layout error.

## Directions for a fix (not decided/scoped yet)

- **Width-aware line breaking**: make `pack_into_systems` accumulate real
  measure width (reusing `measure_note_weight`/`measure_column_weights`
  logic) and start a new system once adding the next measure would exceed
  `usable_music_width`, capped by `max_measures_per_system` as an upper
  bound rather than the sole criterion. This is the architecturally correct
  fix but is a real behavior change — it would reflow every existing
  `.poentaro` file's line breaks, so it needs deliberate scoping (e.g. does
  `max_measures_per_system` become a soft cap or stay a hard cap that can
  still force overflow/squeeze when content is unusually dense?).
- **Reserve minimum padding per column**, mirroring
  `accidental_extra_weight`'s approach, so a chord symbol's weight always
  includes a small trailing gap — reduces the frequency of visible overlap
  from measurement drift, but doesn't fix the underlying proportional
  squeeze if a system is genuinely too dense for its fixed width.
- These two are complementary, not alternatives: width-aware wrapping fixes
  the systemic squeeze, while per-column padding guards against drift even
  when a system's content does fit.

## Repro

Any measure with several `sus4`/`sus2` chords packed into a system with
several other measures (e.g. `max_measures_per_system`'s default of 6, per
`src/render_config.rs:76`) alongside typical short chord symbols will show
the compression. Confirmed visually by the user in the web editor; not
re-verified against a specific generated SVG in this session.
