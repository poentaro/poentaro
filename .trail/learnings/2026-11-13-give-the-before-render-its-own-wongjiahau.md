---
title: Give the before render its own cargo build directory when making PR screenshots
author: Wongjiahau
date: 2026-11-13
branch: feat/lyric-pinyin-override
cost: { minutes: 100, tokens: 212155 }
id: l-8a20bfb3
scope: repo
status: active
src: agent
touches:
  - demo/16-pinyin.poentaro
  - ARCHITECTURE.md
  - crates/poentaro-singer/src/lib.rs
  - crates/poentaro-singer/src/render/lyrics.rs
  - crates/poentaro-singer/src/render/tests.rs
  - crates/poentaro-wasm/src/component/highlight_conversion.rs
  - crates/poentaro-wasm/wit/world.wit
  - src/ast/parsed/events.rs
  - src/chord_lyric_placement_tests.rs
  - src/compiler/part_slice.rs
---

## Decided
For before/after PR screenshots, render "before" from a detached worktree of the base commit with its own build directory, or with a plain `cargo run` there. Never point it at the feature worktree's `target/`.

## Tried and ruled out
Reusing the feature worktree's target dir (CARGO_TARGET_DIR) for the before build, to save a full compile. Afterwards the feature worktree's `cargo run` considered itself fresh and ran the old binary, so before and after looked identical. Touching `src/lib.rs` wasn't enough, because workspace crates (the singer) were stale too. Once the before worktree was deleted, the build script's generated `include_bytes!` font paths pointed at it, and the build broke.

## Watch out
If the before and after renders look the same, suspect a stale binary before suspecting the feature.

In the code:
- recover with `touch build.rs` plus each `crates/*/src/lib.rs` in the feature worktree, then rebuild
- the font paths come from build.rs → `target/debug/build/poentaro-*/out/fonts_generated.rs`
