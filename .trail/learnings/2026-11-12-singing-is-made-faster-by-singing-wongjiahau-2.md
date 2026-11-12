---
title: "Singing is made faster by singing a song's voice segments in parallel, not by compiler flags or lower quality"
author: Wongjiahau
date: 2026-11-12
branch: feat/sing-lyrics
id: l-e11ba64f
scope: repo
status: active
replaces: l-cf98537f
src: agent
touches:
  - crates/poentaro-singer/src/render/mod.rs
  - web/src/worker/singer.ts
  - web/src/worker/singer.worker.ts
---

## Decided
Play used to sing every singing part one after another in the score's web worker before any sound started: 48 s for the 20 s SATB demo. Each fresh voice segment (a part's stretch between rests of 2 s or more) is now a piece sung on its own worker (native: its own thread) and then mixed. The output is byte-identical, and the demo now starts playing after 14 s, bounded by the longest single piece.

## Tried and ruled out
- Fat LTO with one codegen unit (how chimera builds itself): no measurable change in the browser.
- wasm `+simd128`: about 5% faster, not worth a separate build flag.
- Oversample 1 instead of 2: chimera's tract simulation is unstable at 44.1 kHz, and about 98% of the samples come out NaN. The same happens with a male voice at 8 kHz in tests. Keep 2× oversampling.
- Recomputing the tract less often (a control block of 128 or 256 output samples instead of 64; needs a chimera setter): only 10–15% less CPU at 2× oversampling.

## Watch out
- In wasm one voice sings at about 0.6× real time, roughly 3× slower than native. At 2× oversampling about 72% of the time is `AirwayNetwork::step` (the per-sample solver), and about 23% is tract geometry. Real gains need either a faster solver in chimera or playback that starts while singing continues.
- A profile taken at oversample 1 measures a voice that has gone NaN, so it is misleading.
- `poentaro_singer` doesn't guard against non-finite samples. If chimera ever goes NaN, the NaN reaches the mix.

In the code: crates/poentaro-singer/src/render/mod.rs (OVERSAMPLE, piece_count, sing_piece, mix), web/src/worker/singer.ts (choir), web/src/worker/singer.worker.ts.
