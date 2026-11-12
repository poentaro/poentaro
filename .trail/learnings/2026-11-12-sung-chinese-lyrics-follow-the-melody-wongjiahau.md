---
title: "Sung Chinese lyrics follow the melody only, not each character's spoken tone"
author: Wongjiahau
date: 2026-11-12
branch: spike/sing-lyrics
id: l-c269ad5b
scope: repo
status: active
src: agent
touches:
  - Cargo.lock
  - Cargo.toml
  - src/cli/generate.rs
  - src/cli/mod.rs
  - src/lib.rs
  - src/main.rs
  - src/midi/mod.rs
  - src/wav.rs
---

## Decided
When singing lyric parts with chimera, the melody alone sets each note's pitch. Chimera's Mandarin reader (from chimera PR #34) bends every syllable toward its character's spoken tone. In a song that sounds like the voice is speaking the tones over the tune, and the user heard it straight away. We drop those bends before handing syllables to chimera.

## Tried and ruled out
- Keeping #34's tone bends as they come. Its authors measured slightly better intelligibility with them (about 10% syllable errors with bends against 15% without, judged by a recognizer), but it sounds wrong to a listener.

## Watch out
- If lyrics become hard to make out, scale the bends down rather than bringing them back in full.
- Any new Chinese front end that fills a syllable's pitch bends needs the same treatment.

In the code: `read_text` in `src/sing/voice.rs` maps each `chimera::phonetics::Syllable` to `Syllable { pitch: Vec::new(), ..syllable }`.
