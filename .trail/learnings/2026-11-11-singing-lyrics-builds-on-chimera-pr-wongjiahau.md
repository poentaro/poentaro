---
title: "Singing lyrics builds on chimera PR #34, pinned through our own fork"
author: Wongjiahau
date: 2026-11-11
branch: spike/sing-lyrics
id: l-0602ab18
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
Sung lyrics in exported audio (issue #144) use chimera-vocal-synth with PR #34's Mandarin front end (jshph's language router), not PR #48. The maintainer endorsed #34's router as the shared language design and asked for #48's Mandarin to be folded into it; #34 was also the only one measured on *sung* Mandarin.

#34's head (955665cf59) was pushed to wongjiahau/chimera-vocal-synth branch `jianpu/mandarin-singing-pr34` and Cargo pins that rev, because upstream asked jshph to split and rebase #34, so its commit may be force-pushed away and a git dependency on it would stop fetching.

## Tried and ruled out
Writing our own hanzi → pinyin → IPA: #34 already reads characters with phrase-level readings (银行 vs 行走), tones as pitch bends and erhua.

## Watch out
- Read a whole phrase's lyrics in one call (CJK characters joined without spaces) so phrase readings work; fall back to note-by-note when the syllable count doesn't match the note count (erhua merges two characters).
- A lyric part soloed without its target makes every measure zero seconds long in the MIDI timing, so the sung timing also uses the lyric target's own events for the measure length.
- The Mandarin lexicon is runtime data in chimera (`public/data/mandarin.txt.gz`), not inside the crate; we vendor it next to the sing module.

In the code: src/sing/ (voice.rs is the only file that touches chimera), Cargo feature `sing`, CLI `jianpu generate mp3 --sing L=female`.
