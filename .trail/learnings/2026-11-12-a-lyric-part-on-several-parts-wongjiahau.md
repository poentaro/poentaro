---
title: A lyric part on several parts pairs each bar with the bottommost one that has notes there
author: Wongjiahau
date: 2026-11-12
branch: feat/sing-lyrics
id: l-e77fa3fd
scope: repo
status: active
src: agent
touches:
  - ARCHITECTURE.md
  - src/ast/grouped.rs
  - src/combiner.rs
  - src/compiler/mod.rs
  - src/compiler/part_slice.rs
  - src/grid_layout/click_targets_lyric.rs
  - src/grouper/lyrics_pairing.rs
  - src/grouper/mod.rs
  - src/grouper/part_grouper_group.rs
  - src/lib.rs
---

## Decided
- `lyrics [Vo, A2]` pairs each measure's syllables with the bottommost target that has a note in that measure, falling back to the bottommost. The user wanted one lyric part shared by a solo voice and a choir part that only joins in some sections; pairing always with the bottommost made the words underflow wherever it rested.
- The row is still drawn under the bottommost target (`LyricsSlice::drawn_under`, used for `LyricLink`), so layout grouping didn't change.
- The tie entering a measure travels on the slice (`LyricsSlice::entering_tied`, from the paired part's previous measure), so the grouper, compiler, lyric spans, MEI and singing all read one value instead of each walking ties through its own measure order.

## Watch out
- Lyric note ids count the paired part's events, so after a switch they no longer match the row the lyrics are drawn under. Click-target widening is therefore keyed by the note's start column, not by note id.
- Targets must be declared before the lyric part: `lyrics [Vo, A2]` has to come after `[A2]`, or the lyric lines are dropped as unknown parts.

In the code: src/lyric_paired_target.rs (paired_target_index, enters_tied), src/grouper/part_grouper_group.rs group_lyrics_track, src/combiner.rs lyrics_slice, src/part_stream/lyrics.rs lyric_slot_counts, src/grid_layout/click_targets_lyric.rs.
