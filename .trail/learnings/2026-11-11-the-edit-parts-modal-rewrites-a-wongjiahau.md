---
title: "The Edit Parts modal rewrites a part's range from two pitch pickers instead of keeping the written text"
author: Wongjiahau
date: 2026-11-11
branch: feat/part-written-range
cost: { minutes: 36, tokens: 504440 }
id: l-babbc5f8
scope: repo
status: active
src: agent
touches:
  - ARCHITECTURE.md
  - crates/poentaro-wasm/src/component/guest_impl.rs
  - crates/poentaro-wasm/src/component/guest_metadata_and_misc.rs
  - crates/poentaro-wasm/src/component/parts_symbols_conversion.rs
  - crates/poentaro-wasm/wit/world.wit
  - src/ast/parsed/pitch_range.rs
  - src/ast/parsed/pitch_range_tests.rs
  - src/document_tests.rs
  - src/lib.rs
  - src/parser/anyfix/write.rs
---

## Decided
The modal holds the range as two MIDI bounds (lowest, highest), and every rewrite writes them back in canonical spelling. If only one bound is picked, the other comes from the instrument's range, or from the picked pitch when there is no instrument, and the two are then put in order. If no bound is picked, no range is written: an instrument alone must not turn into an explicit range.

## Tried and ruled out
Preserving the range's source text across modal rewrites. It worked while the modal couldn't show the range, but once the range is editable there, keeping two sources of truth (the text and the pickers) made edits ambiguous.

## Watch out
The pickers spell pitches with sharps, so a written flat (`Bb3`) shows as `A#3`, and it is rewritten as `"A#3"` (quoted, since `#` starts an anyfix comment) the next time that part is edited.

In the code: `PitchRange::from_bounds` / `source_text` in src/ast/parsed/pitch_range.rs; `kind_text` in src/source_edit/anyfix_part_edit.rs; the Range cell in web/src/components/PartRow.tsx.
