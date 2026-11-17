---
title: "Score text written into the local database with sqlite3 must be stored as text, or the local backend hangs"
author: Wongjiahau
date: 2026-11-12
branch: feat/sing-lyrics
cost: { minutes: 11, tokens: 25376 }
id: l-209dbcf4
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

## Watch out
- Writing a score into the local D1 copy with sqlite3's `readfile('file')` stores the content as a BLOB. The worker then panics while reading the file list (worker d1 `unwrap` on "invalid type", then "Worker's code had hung"), every `POST /files/list` returns 500, and the web app looks stuck with no files loading.
- Wrap it: `content = cast(readfile('file') as text)`. To repair a row already written: `update files set content = cast(content as text) where typeof(content) = 'blob';` The running worker picks it up without a restart.

In the code: local database at crates/live-share-worker/.wrangler/state/v3/d1/miniflare-D1DatabaseObject/<hash>.sqlite, table `files`; the panic shows up in dev.log under [worker].
