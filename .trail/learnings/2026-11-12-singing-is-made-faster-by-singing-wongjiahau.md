---
title: "Singing is made faster by singing a song's voice segments in parallel, not by compiler flags"
author: Wongjiahau
date: 2026-11-12
branch: feat/sing-lyrics
id: l-cf98537f
scope: repo
status: active
src: agent
touches:
  - ARCHITECTURE.md
  - crates/poentaro-singer-wasm/src/bindings.rs
  - crates/poentaro-singer-wasm/src/lib.rs
  - crates/poentaro-singer-wasm/wit/world.wit
  - crates/poentaro-singer/src/lib.rs
  - crates/poentaro-singer/src/render/mod.rs
  - crates/poentaro-singer/src/render/tests.rs
  - web/src/worker/singer.ts
---

## Decided
Play used to sing every singing part one after another in the score's web worker before any sound started: 48 s for the 20 s SATB demo. Each fresh voice segment (a part's stretch between rests of 2 s or more) is now a piece sung on its own worker (native: its own thread) and then mixed. The output is byte-identical, and the demo now starts playing after 14 s, bounded by the longest single piece.

## Tried and ruled out
- Fat LTO with one codegen unit (how chimera builds itself): no measurable change in the browser.
- wasm `+simd128`: about 5% faster, not worth a separate build flag.
- Oversample 1 instead of 2: twice as fast, but chimera's own live app keeps 2× for quality, so it is the user's call by ear.

## Watch out
- In wasm one voice sings at about 0.6× real time (about 13 s for a 20 s part), roughly 3× slower than native. A part sung without a 2 s rest is one piece, so what remains is per-voice cost: chimera recomputes the tract geometry every 64 output samples (about half the time), and the rest is the per-sample airway step.
- Chimera's bass voice produces NaN at an 8 kHz sample rate, so tests with a male voice must use 44.1 kHz.

In the code: crates/poentaro-singer/src/render/mod.rs (piece_count, sing_piece, mix), web/src/worker/singer.ts (choir), web/src/worker/singer.worker.ts.
