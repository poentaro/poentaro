---
title: "The generated grammar parser lives in its own crate with Clippy's default settings"
author: Wongjiahau
date: 2026-11-11
branch: refactor/anyfix-grammar
cost: { minutes: 28, tokens: 450752 }
id: l-41adf8ba
scope: repo
status: active
src: agent
touches:
  - src/parser/anyfix/decoded.rs
  - clippy.toml
---

## Decided
The parser generated from the anyfix grammar is kept in its own workspace crate, without the workspace lints and with its own clippy.toml. The generated code includes `#![allow(dead_code)]` and has long functions and complex types. The main crate forbids all of those (`forbid` can't be overridden with an `allow` inside the module).

## Watch out
Clippy finds clippy.toml by walking up directories. Without a crate-local clippy.toml, the generated crate picks up the repo root's strict settings (type-complexity-threshold = 200, too-many-lines-threshold = 80) and fails with type_complexity. This isn't a bug in the generator, so don't patch the generated output or upstream for it.

## Tried and ruled out
- Putting the generated module inside the main crate: blocked by `#![forbid(dead_code)]` and the forbidden clippy lints.
- Tweaking the upstream fixture to dodge type_complexity: reverted, because the root clippy.toml was the actual cause.

In the code:
- crates/poentaro-grammar/clippy.toml (contains only a comment, so Clippy uses its defaults)
- crates/poentaro-grammar/Cargo.toml (no `[lints] workspace = true`)
- `just generate-parser` regenerates src/generated.rs
