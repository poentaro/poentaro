---
title: "The old section syntax was removed outright, and text in it still opens with syntax errors"
author: Wongjiahau
date: 2026-11-06
branch: refactor/remove-section-syntax
cost: { minutes: 71, tokens: 891996 }
id: l-78f917de
scope: repo
status: active
src: agent
touches:
  - src/parser/anyfix/tree.rs
  - src/format_anyfix.rs
  - src/document_tests.rs
---

## Decided
After production was migrated, the section-syntax parsers and the converter were deleted with no fallback. The user chose this knowingly: "old text still opens, just that it will be filled with a tons of syntax error (that's fine, at least the user can fix)". Everything goes through the anyfix reader, so unmigrated text and old version history show errors in the editor rather than failing to open. Three production files were left unmigrated on the user's call.

The formatter returns a document unchanged when it has anyfix errors, because reformatting text it can't read would mangle what the user needs to fix by hand.

## Tried and ruled out
Keeping the converter for old text, or migrating version history. Neither was wanted, and either would keep two syntaxes alive.

## Watch out
The `could not convert "…"` handling stays in the reader: migrated production text can contain it, so it isn't a leftover to clean up. The migration script depends on the deleted converter and can't run from this code any more; rerunning a migration means going back to a commit before the removal.

In the code: `src/parser/anyfix/read/`, `src/parser/anyfix/tree.rs`, `src/format_anyfix.rs`, `src/document_tests.rs`.
