---
title: ARCHITECTURE.md explains how the system fits together and is not a code reference
author: Wongjiahau
date: 2026-11-06
branch: master
cost: { minutes: 10, tokens: 70600 }
id: l-d5e3a37f
scope: repo
status: active
src: agent
touches:
  - ARCHITECTURE.md
  - AGENTS.md
---

## Decided
The doc had grown too large to read, because the old rule required an update whenever a key type or entry signature changed, so it turned into a hand-kept copy of the code. It is now cut to how the layers fit together, data flow, security decisions, failure behaviour and a glossary of at most two sentences per term. Reasoning tied to one piece of code moves to a doc comment beside it.

## Tried and ruled out
Deleting the doc outright: the pipeline overview, the glossary and the cross-file security and migration reasoning can't be found from any one file.

## Watch out
Other files cite section and term names ("File tree", "Origin migration", "Rename to Poentaro", "Route list and generated client", "Rod and spring"), so renaming a section means updating those citations. The old text had also drifted from the code: it said the tuplet bracket row was always reserved, and it isn't.

In the code: ARCHITECTURE.md, the "Architecture documentation" rule in AGENTS.md, PRs #138 and the one stacked on it.
