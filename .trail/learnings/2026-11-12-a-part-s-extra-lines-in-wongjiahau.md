---
title: "A part's extra lines in a measure become hidden parts of their own, swapped in by the sequence"
author: Wongjiahau
date: 2026-11-12
branch: feat/sing-lyrics
cost: { minutes: 11, tokens: 25376 }
id: l-e27d4bf4
scope: repo
status: active
src: agent
touches:
  - demo/15-singing.poentaro
  - ARCHITECTURE.md
  - Cargo.lock
  - Cargo.toml
  - crates/poentaro-singer-wasm/Cargo.toml
  - crates/poentaro-singer-wasm/src/bindings.rs
  - crates/poentaro-singer-wasm/src/lib.rs
  - crates/poentaro-singer-wasm/wit/world.wit
  - crates/poentaro-singer/Cargo.toml
  - crates/poentaro-singer/src/lib.rs
---

## Decided
- A second (third, …) `[Key]` line for a part in one measure group is that part's line 2 (3, …), for extra verses or a part that changes on a repeat. The user chose the syntax `Verse WITH ([w] 2, [a] 1) WITHOUT [S]` (lines count from 1, default 1); a missing line falls back to line 1 with a warning.
- Each extra line is declared as a synthetic part `Key.N` right after its part, before the score is read, so rendering, tallying and lyric pairing all work unchanged. `.` can't appear in a written abbreviation, so the names never clash.
- Alternative rows never sound on their own: MIDI and the singing plan skip them by name. Sequence expansion copies the chosen line's notes or syllables into the part's own row for that occurrence, so everything downstream (singing, channels, track filters) still sees only the part.
- They are left out of the part list (toggles, legend); hiding a part hides its lines with it.

## Tried and ruled out
- Stripping alternative rows during navigation expansion: several playback paths return the written score unexpanded (no sequence, measure-range playback without respecting the sequence), so a strip there would miss them. Filtering in the MIDI writer covers every path.

## Watch out
- A one-letter sequence label like `V` is all caps, so anyfix reads it as a keyword and `V WITH …` fails to parse. Use a mixed-case or quoted label.

In the code: src/parser/alternative_lines.rs, src/desugar/key_map.rs, src/midi/navigation/chosen_lines.rs, src/grouper/sequence_resolution.rs (resolve_line_choices), PartRow::is_alternative_line.
