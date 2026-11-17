---
title: "Anyfix documents are decoded from the parsed tree, never lowered to the old section text"
author: Wongjiahau
date: 2026-11-05
branch: claude/epic-hopper-4zcpry
cost: { minutes: 52, tokens: 664129 }
id: l-8daf63c0
scope: repo
status: active
src: agent
touches:
  - demo/01-pitches.poentaro
  - demo/02-durations.poentaro
  - demo/03-slurs-and-ties.poentaro
  - demo/04-chords.poentaro
  - demo/05-lyrics.poentaro
  - demo/06-percussion.poentaro
  - demo/07-directives.poentaro
  - demo/09-sequence.poentaro
  - demo/10-multi-part.poentaro
  - demo/11-part-options.poentaro
---

## Decided
Consumers of anyfix-expr decode the parsed tree directly. The user rejected lowering an anyfix document to section-syntax text and re-parsing it, because anyfix-expr exists to spare the consumer from writing parsing code. It also needed a source map to translate spans back. The score stays one multiline string, read by the existing score parser.

## Tried and ruled out
Lowering plus a source map: it rendered correctly but kept two parsers and a span translation layer, so it was removed.

## Watch out
The saving is small so far: about 995 lines of section-syntax code can go once production files are migrated, against 952 lines of decoder, and the 3866-line score parser is untouched. The 995 is an estimate from tracing reachable functions. Most of the old parser's meaning checks stay, so don't count those files as deletable.
Section-syntax parsers can't be deleted until production files are migrated.
Running all 613 e2e tests locally overloaded the machine and produced 18 false failures, all of which passed when re-run one at a time. Run scoped tests locally and let CI run the full suite.
Run `cargo test --workspace`, not just the root crate: a wasm crate test failure was missed otherwise.

In the code: src/parser/anyfix/read/, src/parser/anyfix/tree.rs, src/source_edit/anyfix_edit.rs, src/source_edit/anyfix_part_edit.rs, crates/poentaro-wasm/src/tests_render_invalid_part.rs
