---
title: "The anyfix reader must read values from the tree, not rebuild strings to parse again"
author: Wongjiahau
date: 2026-11-06
branch: refactor/anyfix-typed-values
cost: { minutes: 35, tokens: 186251 }
id: l-f471c893
scope: repo
status: active
src: agent
touches:
  - web/e2e/features/section-jump-select.feature
  - web/e2e/features/sequence-jump-select.feature
  - web/e2e/features/steps/cloud-file-history.steps.ts
  - web/e2e/features/steps/edit-parts-modal-setting-limits.steps.ts
  - web/e2e/features/steps/edit-parts-modal.steps.ts
  - web/e2e/features/steps/measure-autoscroll.steps.ts
  - web/e2e/features/steps/part-label-copy-paste-across-systems.steps.ts
  - web/e2e/features/steps/part-stream-editor-sources.ts
  - web/e2e/features/steps/play-note-selection-audio.steps.ts
  - web/e2e/features/steps/section-jump-select.steps.ts
---

## Decided
The reader turns tree nodes straight into typed values. The user rejected the way the first version worked, where tree values were joined back into strings and passed to the old value parsers, because it kept the old parsing code alongside the new tree and is why moving to anyfix did not shorten the parsing code.

## Tried and ruled out
Passing `value_text` output to string parsers (`parse_positive_u32`, `parse_offset`, stripping `%` from a joined volume). It kept old and new behaviour identical while both syntaxes existed, but nothing justifies it once the old syntax is gone.

## Watch out
Some conversion is still needed: a number node holds text, and `+1` and `80%` are call nodes. The score body is different: it stays one string read by the score parser, by design.

In the code: `sound_program` in src/parser/anyfix/read/parts.rs is the pattern to copy. `value_text` is still fine for reading a name, like an instrument.
