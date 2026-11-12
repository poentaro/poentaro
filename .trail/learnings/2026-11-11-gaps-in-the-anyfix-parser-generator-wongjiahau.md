---
title: "Gaps in the anyfix parser generator are fixed upstream, not worked around in this repo"
author: Wongjiahau
date: 2026-11-11
branch: refactor/anyfix-grammar
cost: { minutes: 11, tokens: 108767 }
id: l-def838fe
scope: repo
status: active
src: agent
touches:
  - Cargo.toml
---

## Decided
The user owns both anyfix-expr and this repo. When the grammar can't express something cleanly, change the generator in anyfix-expr and bump the pinned rev here. Don't add workarounds to the grammar or the Rust readers. Fixes made this way so far: `EACH` (per-item error recovery), generated code that builds on edition 2021, and no overlap warning for a fallback choice placed after the cases it catches.

The `value` rule ends with a catch-all `written: ANYFIX CALL`, so things like `20k` or `(0, -)` still decode, and the Rust readers report them as the wrong value for their field. That matches what the old hand-written parser accepted.

## Tried and ruled out
- Narrowing the earlier choices to avoid the overlap warning: rejected. The fallback is intended, so the generator now checks whether the later choice covers the earlier one, choice by choice.

## Watch out
Bumping anyfix-expr also changes the formatter: `-4` is a negative number literal, so `octave: - 1` is now written `octave -1`, and `20k` lexes as `20` followed by `k`. The generator's error messages replace some of the old custom wording.

In the code:
- crates/poentaro-grammar/poentaro.anyfix
- the anyfix-expr rev in Cargo.toml and crates/poentaro-grammar/Cargo.toml must stay the same
- `just generate-parser`
