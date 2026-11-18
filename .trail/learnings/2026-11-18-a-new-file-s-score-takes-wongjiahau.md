---
title: "A new file's score takes the file's name as its title unless the content already has one"
author: Wongjiahau
date: 2026-11-18
branch: feat/174-file-name-from-title
cost: { minutes: 11, tokens: 87556 }
id: l-2a315f27
scope: repo
status: active
src: agent
touches:
  - src/file_tree/file_ops.rs
---

## Decided
Issue #174 ("New file name should be title of score by default unless specified") had no body. Read as: when a file is created, a missing or blank `title` in its metadata becomes the file's name (no extension, no folder). A non-blank title already in the content is kept — that is the "unless specified".
The rule lives in Rust's file-tree create, not in the web, so every caller gets it.

## Watch out
The new-file template no longer carries `title "Untitled"`; putting a title back in the template would silently disable the default.

In the code:
- `src/file_tree/file_ops.rs` `create_file` → `titled_by_default` (uses `source_edit::update_metadata_field`)
- `new_file_template.poentaro`
