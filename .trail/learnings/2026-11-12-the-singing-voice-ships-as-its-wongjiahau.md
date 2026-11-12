---
title: "The singing voice ships as its own wasm module, and only the whole-score MP3 export sings"
author: Wongjiahau
date: 2026-11-12
branch: feat/sing-lyrics
cost: { minutes: 15, tokens: 25376 }
id: l-b3fb5e98
scope: repo
status: active
src: agent
touches:
  - web/package.json
---

## Decided
- The chimera voice lives in a separate wasm component (poentaro-singer-wasm), fetched only when a score has a voiced lyric part. Linking it into the main app wasm would add about 630 KB gzipped for every user, singing or not; on its own it is about 450 KB gzipped.
- The main wasm only builds the singing plan (plain notes and syllables, passed as JSON bytes); the singer turns the plan into samples, and generate-mp3 mixes them in. The two crates share only the plan types.
- Only the whole-score MP3 export (and the CLI's generate mp3/wav) sings. Singing a full song takes about 20 s natively and about 50 s in the browser, far too slow for Play or measure-range playback, so those stay instrumental. Split exports don't sing yet.
- A voice is written on the lyric part (`lyrics [S] { voice: female, volume: 80 % }`), not on the notes part, so each verse can have its own voice.

## Watch out
- Timbre tweaks through chimera's SourcePatch (pressure, tension and so on) made no difference the user could hear, so the defaults stay.

In the code: crates/poentaro-singer (plan types, `render` feature = chimera), crates/poentaro-singer-wasm, src/sing/, web/src/worker/singer.ts, `build:wasm-singer` chained from `build:wasm-component` in web/package.json.
