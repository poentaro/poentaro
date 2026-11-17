# Tuplet syntax implementation plan

Branch: `feature/tuplets`. This file tracks atomic, sequential steps. Each step is meant to
be implemented in its own fresh-context session/commit. **After finishing a step, check its
box, commit, and stop** (unless told to continue) — the next session picks up from here.

Full background/design rationale lives in git history of this file's first commit and in the
original plan below. Read the "Full plan" section at the bottom for design details not
repeated in the step descriptions.

## Steps

- [x] **Step 1 — Lexer: `{`/`}` tokens**
  `src/parser/score/timed_parser/timed_lexer.rs`, `timed_lexer/directive_lexing.rs`:
  add `LBrace { num: u32, den: Option<u32> }` / `RBrace` tokens (mirror `LParen`/`RParen`).
  Add `try_lex_tuplet_open` (parallel to `try_lex_time_signature`) recognizing `N:{` or
  `N:M:{` at a word boundary, consuming digits/colons/brace together. Update
  `timed_recursive_descent_parser.rs::parse_timed_unit`'s `text.find(...)` / `unit_end_abs`
  boundary scanners to also stop at `{`/`}`. Add lexer unit tests in
  `timed_lexer_tests.rs`.

- [x] **Step 2 — Groups: `TupletStack`, ratio resolution, errors**
  `src/parser/score/timed_parser/groups.rs`: add `TupletStack`/`TupletFrame { note_count,
  segment_start, num, den }` alongside `GroupStack`, with `open_tuplet`/`close_tuplet`.
  `src/parser/score/timed_parser/timed_recursive_descent_parser.rs` /
  `timed_recursive_descent_parser/group_and_repeat.rs`: generalize `parse_atoms`'s
  `stop_at_rparen: bool` to also stop at `RBrace` when inside a tuplet. No cross-line tuplets:
  unclosed `{` at end of line is a hard parse error (no `finalize_open_frames` equivalent).
  Implicit ratio table `2→3, 3→2, 4→3, 5→4, 6→4, 7→4, 9→8`; anything else without explicit
  `:M` is `RecoverableError::tuplet_ambiguous_ratio`. Add `RecoverableErrorKind` variants
  `TupletAmbiguousRatio`, `TupletNoteCountMismatch` in `src/error/recoverable_kind.rs` +
  constructors in `recoverable_error.rs` (style of `duration_unexpected_char`). Note-count
  mismatch on close is a recoverable error. Add tests in a new
  `src/parser/score/timed_parser/tuplet_tests.rs`.

- [x] **Step 3 — AST: `TupletInfo` on parsed nodes**
  `src/ast/parsed.rs`: add `pub struct TupletInfo { pub num: u32, pub den: u32 }` (no tuples)
  and `pub tuplet: Option<TupletInfo>` field to `ParsedNote`, `ParsedChordNote`,
  `ParsedPercussionHit`, `ParsedRest`. Wire it up in
  `timed_recursive_descent_parser.rs::parse_timed_unit` right after `duration_meta.duration`
  is computed, before `H::to_event(...)`, and in
  `group_and_repeat.rs::parse_repeat_unit` for repeat atoms opened inside a tuplet. Update all
  other construction sites of these structs (test helpers, desugar.rs, etc.) to set
  `tuplet: None` — grep for struct-literal construction of these 4 types.

- [x] **Step 4 — Grouped IR + new rescale pass**
  `src/ast/grouped_notes.rs`: add `resolution_multiplier: u32` (default 1) to
  `GroupedMeasure` (find this type — may need to check `src/grouper/` for where measures are
  assembled if not in this file), and `tuplet: Option<TupletInfo>` on `GroupedNote`,
  `GroupedRest`, `GroupedChordNote`, `GroupedPercussionHit`.
  New file `src/grouper/tuplet_rescale.rs` (check actual grouper module path first): a pass
  that runs on one line's `Vec<Spanned<ScoreEvent>>` *before* `PartGrouper` consumes them.
  Scans for `TupletInfo` tags (including nested), computes `multiplier = LCM(all N's found)`,
  multiplies every event's `duration` by `multiplier` (plain notes too), and for
  tuplet-tagged events further multiplies by `den/num`. Returns the multiplier alongside the
  rescaled events for attachment to the produced `GroupedMeasure`. Add grouper-level tests:
  a tuplet correctly filling a beat (e.g. `{3:1_1_1_}` filling exactly 1 beat in 4/4).

  Implemented as: `rescale_tuplets` in `src/grouper/tuplet_rescale.rs`, wired into
  `part_grouper_group::group_timed_track` (called once per `ParsedMeasureSlot::Real`,
  before its events reach `PartGrouper`). `PartGrouper` gained a `resolution_multiplier`
  field (`begin_measure_slot`/`effective_capacity`) so its capacity/flush/`handle_extension`
  math stays correct for a rescaled measure — a small slice of Step 5's job pulled forward
  here because it was needed both for a genuinely passing test and to satisfy
  `#![forbid(dead_code)]` (the new `resolution_multiplier`/`tuplet` fields need a real
  production reader, not just a writer). Step 5 should still do its own pass over
  `grouping.rs`/`compiler/part_slice*.rs`.

  **Discovered gap, not yet fixed (unassigned to a step)**: the parser's own per-measure
  capacity check (`interleaved_beat_padding::validate_and_pad_beats`, upstream of
  `rescale_tuplets`) compares each tuplet atom's *written* (nominal, uncompressed)
  duration against the bar's raw capacity — it has no concept of tuplet compression. A
  tuplet written with its natural/nominal duration (the common case — e.g. an eighth-note
  triplet in `syntax.md`'s own suggested notation, `3:{1_1_1_}`, written as three eighth
  notes) sums to *more* raw quarter-beats than it should occupy once compressed, so once
  other notes fill out the rest of the bar, the parser's raw-sum check can truncate or
  reject the measure before `rescale_tuplets` ever gets a chance to compress it down to
  size. The grouper-level test added for this step (`measure_with_a_triplet_groups_with_correctly_rescaled_durations`
  in `src/grouper/tests_tuplets.rs`) works around this by choosing note counts whose raw,
  pre-compression sum already equals the bar's capacity exactly — real-world tuplet usage
  will need `interleaved_beat_padding.rs` made tuplet-aware (comparing against each
  tuplet's *compressed* duration, not its written one) before this fully works
  end-to-end. See the **Tuplet** glossary entry in `ARCHITECTURE.md` for more detail.

- [x] **Step 5 — Thread `multiplier` through grouping/compiler**
  `PartGrouper` (find in `src/grouper/`), `src/grouper/grouping.rs`,
  `src/compiler/part_slice.rs`, `src/compiler/part_slice_unit.rs`: every literal that
  hardcodes "quarter-beat = 4 sixteenths" (`4`, `8`, `16`, `3`, `1`) becomes
  `BASE_CONST * multiplier`, with `multiplier` threaded in as a parameter/field. Both
  `handle_extension` (`-` suffix) in `PartGrouper` and the equivalent `+= 4` in
  `grouping.rs` become `+= 4 * multiplier`. `column: u32` in `part_slice_unit.rs`/
  `part_slice.rs` is a pure additive grid index — safe to scale uniformly per-measure. Add
  half-bar-boundary / dotted-eighth-tail tests around a tuplet to confirm rules still fire
  correctly.

- [x] **Step 6 — Grid width + MIDI ticks divide back out**
  `src/grid_layout/layout_spacing.rs`: `column_weight` (~line 24) and any other grid-width
  literal (`MULTI_MEASURE_REST_WIDTH` etc. in `compiler/types.rs`/`layout_spacing.rs`) must
  divide by that measure's `multiplier` so pixel width reflects real musical content, not
  inflated column count.
  `src/midi/midi_notes.rs::duration_to_ticks`: becomes `quarter_beats * TPQ / (4 *
  multiplier)`. Known limitation: multipliers whose prime factors don't divide 480 (e.g.
  involving 7 or 11) round — document, don't bump TPQ.
  Add a MIDI test: tick output for a triplet measure sums to the same total ticks as the
  equivalent non-tuplet measure.
  **Verify**: existing demo files (none use tuplets) render pixel-identically before/after —
  multiplier defaults to 1, should be a no-op diff. Diff generated SVG output for
  `demo/01-pitches.poentaro` etc. before/after this step.

  Implemented as: `midi::midi_notes::duration_to_ticks(quarter_beats, multiplier)` now takes
  `multiplier` and computes `quarter_beats * TPQ / (4 * multiplier)`, threaded from
  `PartSlice::resolution_multiplier` through `midi::event_processing`'s
  `process_measure_notes`/`process_chord_events`/`process_percussion_events` and
  `midi::timing_note_events`'s `PartEventContext`/`TickSpanEvent`. New tests in
  `src/midi/tests_tuplets.rs` confirm a rescaled triplet's ticks sum to the same total as
  the equivalent non-tuplet measure, both at the `duration_to_ticks` unit level and
  end-to-end through `measure_start_times_seconds`.

  Grid width turned out to need **no code change**: investigated `column_weight`/
  `measure_column_weights`/`measure_note_weight` in `src/grid_layout/layout_spacing.rs` and
  confirmed (both by reading the width-allocation math and empirically, via generated SVG
  x-coordinates for a single-measure triplet) that pixel width is already
  multiplier-invariant — `measure_note_weight` counts written note *occurrences*, not
  columns or duration, and `measure_column_weights` normalizes within-measure by
  `column_weight_sum`, so a tuplet-inflated `col_count`'s extra columns (all
  zero-weight, since no element sits at them) never reach a raw pixel width. Documented
  this invariant with comments on both functions and in the **Tuplet** glossary entry
  (`ARCHITECTURE.md`) rather than force an artificial division. All 14 `demo/*.poentaro`
  files render byte-identical SVG *and* MIDI output before/after this step.

- [x] **Step 7 — Rendering: tuplet brackets**
  Follow the slur-arc pipeline as template (`SlurSpan` → `resolve_slur_spans` in
  `src/grid_layout/slur_placement.rs` → `resolve_span_marking` in
  `src/coordinate_resolver/resolve.rs` → `render_tie_or_slur` in
  `src/renderer/new_renderer/glyph_renderers.rs`):
  1. `grid_layout`: derive `TupletSpan { from_column, to_column, part_index, label: String }`
     per contiguous tuplet group (label = `N` digit e.g. `"3"`), as a `GridElement` with new
     `GridContent::TupletBracket { label }` variant.
  2. `src/grid_layout/layout.rs::note_part_sub_row_heights`: prepend `tuplet_bracket` to the
     row order (`[arc, above_dot, note_head, below_dot, half_ul, quarter_ul]` →
     `[tuplet_bracket, arc, ...]`) so brackets stack above slur arcs.
  3. `src/coordinate_resolver/resolve.rs`: resolve grid columns to pixel x/width like
     `resolve_span_marking` does for `TieOrSlur`, producing `AbsoluteContent::TupletBracket {
     label, width }`.
  4. `src/renderer/new_renderer/glyph_renderers.rs`: add `render_tuplet_bracket` — two short
     vertical ticks + horizontal line (or shallow bracket path) spanning `width`, with an
     `SvgKind::Text` label centered above the midpoint (copy `SvgKind::Text` construction from
     `render_note_head`). Dispatch alongside `render_tie_or_slur` in
     `src/renderer/new_renderer.rs`.
  Tuplets don't span line/system breaks, so no `TieOrSlurTail`/`Head` cross-system-break
  equivalent is needed.

  Implemented as described, with `TupletSpan` built one step earlier than the plan's "grid_layout"
  wording implies: `compiler::part_slice`/`part_slice_unit` accumulate it directly (mirroring
  `SlurSpan`'s own construction site — `slur_chains.rs`'s `extend_note_chains`), in a new
  `compiler::tuplet_spans` module (`record_tuplet_tag`/`finish_tuplet_spans`, tracking one
  `PendingTupletSpan` per part slice — never carried cross-measure, unlike `PendingSlurOpen`).
  `TupletSpan { part_index, measure_index, from_column, to_column, label }` (one `measure_index`,
  not `from_measure`/`to_measure`, since a tuplet never crosses a measure) lands on
  `CompileResult.tuplet_spans`, remapped through `merge_rest_runs`'s `measure_to_block` exactly
  like `slur_spans`. `grid_layout::tuplet_placement::resolve_tuplet_spans` (new file, mirroring
  `slur_placement.rs`) then resolves it to a `GridElement` per system — only ever the
  same-system case of `resolve_slur_spans`, no tail/head split. `TimedUnit::tuplet()` was added to
  the trait (`compiler::timed_unit`) alongside `duration()`/`slur_key()` etc. so
  `compile_timed_unit`/`compile_unit` can read a note/chord-note/percussion-hit's tag the same way
  as its other span-relevant fields; `GroupedRest.tuplet` is read directly in `compile_rest` (rests
  aren't `TimedUnit`s). `note_part_sub_row_heights` grew from 6 to 7 elements; every fixed sub-row
  index downstream that assumed 6 (`expand_note_part`'s `head_sub`/arc-row index,
  `highlight::system_musical_row_count`, `note_highlight::part_row_ranges`) was updated to 7 — a
  chord-only row is unaffected (still 4, no tuplet-bracket row, since chord notes carry no
  practical tuplet use case in current syntax). This reserves the tuplet-bracket row's height for
  *every* note-part row unconditionally (matching how the arc row already reserves space whether
  or not that part has any slurs), so — unlike Step 6 — non-tuplet scores do *not* render
  pixel-identical to before this step; every note row grows taller by one sub-row. Visually
  verified via `cargo run -- generate svg` on a `3:{1_1_1_} 2_ 3_ 4_ 5_ 6_` measure: the bracket's
  two ticks + horizontal line span exactly the first-to-third-triplet-note columns, with the `"3"`
  label centered above, sitting cleanly above the note-head row with no collision against the
  underlines rendered below the plain eighth notes that follow. The known `underline_count = 0`
  gap for tuplet-tagged notes (documented in `PartState::multiplier`'s doc comment and the
  **Tuplet** glossary entry) was deliberately left alone — the bracket alone reads clearly as a
  tuplet grouping without a beam underneath it, and fixing beaming was explicitly out of scope for
  this step.

- [x] **Step 8 — Docs**
  `syntax.md`: new "Tuplets" subsection under Duration suffixes — `{N:notes}`,
  `{N:M:notes}`, implicit-ratio table, nesting with `(...)`, no-cross-line-tuplets rule.
  `ARCHITECTURE.md`: document `TupletInfo`, `resolution_multiplier` on `GroupedMeasure`,
  `GridContent::TupletBracket`/`AbsoluteContent::TupletBracket`, and the tuplet-rescale pass
  in the pipeline-layers section + glossary.

- [x] **Step 9 — E2E fixture + final verification**
  New `demo/12-tuplets.poentaro` (check existing numbered `demo/*.poentaro` files for the next
  free number first) exercising a triplet, a duplet, and a quintuplet.
  `cargo run -- generate svg demo/12-tuplets.poentaro` — visually confirm brackets render above
  the note row without colliding with slur arcs, octave dots, or beam underlines.
  If the project has SVG snapshot/golden-file tests under `tests/`, add one for this fixture.
  Run full `cargo test` and the e2e suite. Delete this file (`TUPLET_PLAN.md`) once all steps
  are checked off and merged, or leave it — check with the user.

  Implemented as: `tests/main/demo_source.rs` already iterates every file under `demo/` and
  asserts each parses/renders with zero diagnostics, so no separate snapshot test was needed —
  `demo/12-tuplets.poentaro` is automatically covered by `demo_files_parse_and_render`,
  `demo_files_have_no_diagnostics`, and `demo_files_render_expected_content` the moment it
  exists in that directory.

  Getting a zero-diagnostic fixture took substantially more empirical trial-and-error than the
  known duplet/`flush_measure` bug alone predicted, because the "solo duplet in its own measure"
  workaround from this plan's own instructions is necessary but **not sufficient**. Discovered
  two more failure modes while iterating (neither fixed — both out of scope, same as the
  documented one; recorded here for whoever tackles the tuplet-capacity gap for real):

  1. **A tuplet measure that isn't a part's very last measure corrupts the *next* measure's
     flush, regardless of tuplet type.** `PartGrouper::begin_measure_slot` (called when the
     next measure's events start arriving) overwrites `self.resolution_multiplier` *before*
     the previous, still-pending measure has been flushed — and any tuplet measure whose
     scaled duration doesn't exactly equal its effective capacity (which, per Step 4's gap,
     is *every* tuplet measure today: compressing ratios always land short, and duplet's
     expanding ratio always overflows if written to nominally fill the bar) stays pending
     rather than self-flushing. The result ranges from a spurious `extension '-' without a
     preceding note or rest` warning to the same `flush_measure` `debug_assert` panic this
     plan already documents, depending on multiplier/timing. Empirically, *any* tuplet
     measure — triplet and quintuplet included, not just duplet — is only safe if it is the
     last measure ever pushed to its part's track.
  2. **Grid column width is not multiplier-invariant when a system combines parts with
     different tuplet multipliers.** Two parts in the same system, one holding a triplet
     (multiplier 3) and the other a quintuplet (multiplier 5), render with columns wide
     enough to push content far past the page's viewBox (observed x-coordinates up to ~3400
     against a 595-wide A4 page) — contradicting Step 6's "grid width is multiplier-invariant"
     verification, which only checked *uniform*, non-tuplet (multiplier-1) content across
     parts.

  The shipped fixture sidesteps both: all three tuplets (duplet `2:{...}`, triplet `3:{...}`,
  quintuplet `5:4:{...}`) live in **one measure of one single part**, immediately followed by
  three plain eighth notes, with every note count and value chosen so the measure's nominal
  (pre-rescale) duration sums to exactly 16 quarter-beats — satisfying the parse-time capacity
  check with no padding or truncation, and being both the first and only measure means there's
  no next `begin_measure_slot` call to corrupt it. Visually verified (via a headless-Chromium
  screenshot of the generated SVG, `rsvg-convert`/`resvg`/`inkscape` all being unavailable in
  this environment): the duplet/triplet/quintuplet brackets each sit cleanly above their own
  note span with a centered digit label, and the trailing beam underline under the three plain
  eighth notes sits in a disjoint column range — no collisions. Full `cargo test` passes (762
  tests: 709 lib + 53 integration). `cargo clippy --all-targets` fails on this branch
  independent of this step's change (`git diff --stat` for this commit touches only
  `demo/12-tuplets.poentaro` and this file) — the failures are pre-existing `too_many_arguments`/
  `too_many_lines` lints plus a `disallowed_macros` hit on the `debug_assert!` in
  `part_grouper.rs` added by Step 4, all predating this step and out of scope to fix here.

## Full plan

(Original approved plan, kept verbatim for reference — see step descriptions above for the
condensed, sequenced version of the same content.)

### Context

The `.poentaro` grammar has no way to notate tuplets (triplets, duplets, quintuplets...).
Duration is currently a plain `u32` "quarter-beat" count (1 = a sixteenth note) used
identically across parsing, measure-capacity validation, beat-grouping rules, SVG
grid-column layout, and MIDI tick export. Tuplets need fractional beat subdivisions (e.g. an
eighth-note triplet = 4/3 quarter-beats), which this integer grid cannot represent today.

Agreed syntax:
- `{N:notes}` — N notes in the time normally taken by the implied "against" count (standard
  tuplet meaning: 3-in-2, 2-in-3, 5-in-4, 6-in-4, 7-in-4, 9-in-8, ...).
- `{N:M:notes}` — explicit override, N notes in the time of M notes of the same written
  value.
- Any `N` without a standard implied ratio and without an explicit `:M` is a parse error
  ("ambiguous tuplet ratio, use `{N:M:...}`").
- Scope: fully general N-tuplets, not just triplets/duplets. Internal grid resolution is
  rescaled dynamically per measure (quintuplet measure gets a ×5 grid, combined
  triplet+quintuplet gets ×15, etc.) without hardcoding a single global multiplier.

### Design: per-measure resolution rescaling (no whole-file pre-scan)

`PartGrouper` already processes one measure/line at a time and the recursive-descent parser
materializes a full list of `ScoreEvent`s per line before grouping happens. Insert a new
rescaling pass per measure, between "parser produces `ScoreEvent`s for this line" and
"`PartGrouper` consumes them":

1. Parse the line normally. `duration.rs`'s suffix parser (`_`, `=`, `.`, `-`) stays
   untouched — it keeps producing base-scale quarter-beats exactly as today, independent of
   tuplets.
2. A tuplet-carrying event gets an attached `TupletInfo { num, den }` set by the parser when
   inside an open `{...}` bracket — no arithmetic yet, just the ratio tag.
3. Before handing the line's events to `PartGrouper`, a new pass scans for any `TupletInfo`,
   computes `multiplier = LCM(all N's found, including nested tuplets)`, and:
   - Multiplies every event's `duration` in that line by `multiplier` (plain notes too).
   - For tuplet-tagged events, further multiplies by `den/num` (guaranteed to divide evenly).
   - Attaches the resulting `multiplier` to the produced `GroupedMeasure`.
4. `PartGrouper`, `grouping.rs`, `compiler/part_slice*.rs` keep existing logic unchanged, but
   every literal hardcoding "quarter-beat = 4 sixteenths" is expressed as
   `BASE_CONST * multiplier`.
5. Two places divide back out using the measure's `multiplier` so non-tuplet music renders
   pixel-identically to today: `layout_spacing.rs` column weight, and
   `midi_notes.rs::duration_to_ticks`.
6. `handle_extension` (`-` suffix) needs `+= 4 * multiplier`.

### Testing

Parser-level (implicit ratio table, explicit `{N:M:...}`, ambiguous-N error, note-count
mismatch error, nesting with `(...)` both directions, unclosed-tuplet error), grouper-level
(measure-capacity validation, half-bar-boundary/dotted-eighth-tail around a tuplet), e2e
(`demo/12-tuplets.poentaro` rendered via `cargo run -- generate svg`), MIDI export (tick sum
round-trip check).
