---
title: Rewriting file contents directly in the database must also write a version-history row
author: Wongjiahau
date: 2026-11-06
branch: chore/migrate-anyfix-script
cost: { minutes: 29, tokens: 146339 }
id: l-4db49065
scope: repo
status: active
src: agent
touches:
  - ARCHITECTURE.md
  - Cargo.lock
  - Cargo.toml
  - anyfix-syntax.md
  - crates/poentaro-wasm/src/component/guest_metadata_and_misc.rs
  - crates/poentaro-wasm/src/component/guest_outbox.rs
  - crates/poentaro-wasm/src/component/guest_presets.rs
  - crates/poentaro-wasm/src/component/guest_range_edits.rs
  - crates/poentaro-wasm/src/component/guest_selection_and_render.rs
  - crates/poentaro-wasm/src/component/guest_selection_range.rs
---

## Decided
A migration that rewrites `files.content` also inserts a `file_versions` row holding the reverse patch back to the old text, and bumps `revision` by one. Both statements only apply if `revision` is unchanged, so a save made during the migration wins and a re-run picks that file up.

## Tried and ruled out
Copying the lyrics migration's plain `UPDATE ... revision = revision + 1`: the new revision has no version row, which leaves a gap, and `reachable_versions` stops at a gap, so every older version of the file disappears from History. That script also reads the `docs` table, which has since been dropped, so it no longer runs.

## Watch out
Compare SVGs and error messages. The 2026-11-06 anyfix run held 4 files only because their messages changed, with no SVG difference. Files with `Lyrics [l] = lyrics`, `# groups` or `notes+lyrics` stay in section syntax until fixed by hand, which blocks removing the section parsers.
Removing those parsers also has to account for old `file_versions` rows, browser-saved files and previously exported SVG/PDF files, which can still hold section syntax.

In the code: scripts/migrate-anyfix-d1.py, tools/migrate-anyfix/src/migration.rs, crates/live-share-worker/src/file_history.rs
