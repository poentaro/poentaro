---
title: "Pitch names in anyfix options: sharps are quoted, and uppercase pitches nest as keywords"
author: Wongjiahau
date: 2026-11-11
branch: feat/part-written-range
cost: { minutes: 16, tokens: 174338 }
id: l-b6dd2e6f
scope: repo
status: active
src: agent
touches:
  - src/parser/parts_parser/mod.rs
---

## Decided
A part's `range` option names pitches as letter + optional `b` + octave (`C3`, `Bb4`). A sharp has to be quoted (`"F#3"`), because `#` starts a comment in anyfix, so a bare `F#3` swallows the rest of the line (including the closing `}`).

## Watch out
Uppercase words like `C3` and `G4` are anyfix keywords, and keywords split a run: `C3 to G4` parses as the call `C3 _ G4` with `to` as its argument, not as three words. `value_text` then reads `C3toG4`. Walk the call tree and flatten slots instead of reading the joined text.

In the code: src/parser/anyfix/read/written_range.rs (`range_words`), src/ast/parsed/pitch_range.rs (`parse_midi_note_name`).
