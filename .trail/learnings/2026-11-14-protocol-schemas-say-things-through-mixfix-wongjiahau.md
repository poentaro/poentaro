---
title: "Protocol schemas say things through mixfix shapes and names, keeping doc strings only for rules a shape can't carry"
author: Wongjiahau
date: 2026-11-14
branch: feat/anyx-protocols
id: l-22b4617f
scope: repo
status: active
src: agent
touches:
  - crates/live-share-worker/src/handlers/protocols/app.rs
  - crates/live-share-worker/src/handlers/protocols/app/files.rs
  - crates/live-share-worker/src/handlers/protocols/public.rs
  - crates/live-share-worker/src/handlers/protocols/test.rs
  - crates/poentaro-protocol/schemas/app/account.anyx
  - crates/poentaro-protocol/schemas/app/files.anyx
  - crates/poentaro-protocol/schemas/app/folders.anyx
  - crates/poentaro-protocol/schemas/public/account.anyx
  - crates/poentaro-protocol/schemas/test/accounts.anyx
---

## Decided
- In the anyx schemas, write variants and errors as sentences with slots where their words go, e.g. `(path: TEXT) would be both file and folder`, `(email: TEXT) is not a test account`, `locked for (duration: INTEGER) seconds`, then drop the `#` doc that only restated it.
- A slot's name says what the value is; the trailing word gives the unit. The user corrected `(seconds: INTEGER) seconds` to `(duration: INTEGER) seconds`.
- Doc strings stay for rules no shape can carry (e.g. "Nothing moved.").

## Tried and ruled out
- Long renames that cram a whole doc into the name ("saved first by someone at revision ...", "list including trash"): too far.
- Dropping every doc string: not the intent either.

## Watch out
- Generated names come from the words only: `locked for _ seconds` is `LockedForSeconds { duration }`, `_ would be both file and folder` is `WouldBeBothFileAndFolder { path }`. Renaming a shape changes the wire text and the Rust/TS callers in the same commit.
- Fix guidance upstream: anyx `crates/anyx-protocol/PROTOCOL.md` "Writing shapes" (commit 3aac265) is where agents learn this.

In the code: `crates/poentaro-protocol/schemas/**.anyx`; callers in `crates/live-share-worker/src/handlers/protocols/`.
