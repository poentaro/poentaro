---
title: Lyric pinyin readings count only when attached to a Chinese character
author: Wongjiahau
date: 2026-11-13
branch: feat/lyric-pinyin-override
cost: { minutes: 28, tokens: 112266 }
id: l-8fd4f155
scope: repo
status: active
src: agent
touches:
  - ".github/workflows/ci.yml"
  - ARCHITECTURE.md
  - Cargo.lock
  - Cargo.toml
  - crates/live-share-worker/Cargo.toml
  - crates/live-share-worker/cloudflare.config.ts
  - crates/live-share-worker/queries/delete_api_key.sql
  - crates/live-share-worker/queries/delete_api_keys_of_users.sql
  - crates/live-share-worker/queries/get_api_key.sql
  - crates/live-share-worker/queries/get_file_by_name.sql
---

## Decided
`字(pinyin)` is a reading only when the parenthesis touches a Han character (no space) and holds one pinyin syllable. Anything else keeps its old meaning: literal lyric text in a plain lyric line (`(la)`, `oh(hang)`, `行 (háng)`), and a bracketed item under a chord part (`我(陪)`). A Han-attached bracket holding non-pinyin is an error in a plain line, but stays a bracketed item under a chord part.

Why: production scores couldn't be checked for existing `(…)` use in lyric lines (the prod read was denied), and chord-lyric lines already use `( )` for timing. Making every stray parenthesis an error, as issue #154 first proposed, risked breaking saved scores.

## Watch out
- The pinyin syllable table in `src/pinyin/syllables.rs` is copied from chimera's Mandarin front end; keep the two in step if chimera's changes.
- Staff notation (Verovio) has no ruby, so MEI writes `行(háng)` inline.
- The lyric row grows by a ruby band per system, not per measure: row heights are per system, and the click-target walks count rows by index, so the ruby lives in the same row rather than in a new row.

In the code: `utils::pinyin_override_at`, `parser/lyrics/bracketed.rs::attached_pinyin_end`, `grid_layout/layout_systems.rs::lyric_ruby_band_pt`, `coordinate_resolver/resolve.rs::lyric_ruby_shift`.
