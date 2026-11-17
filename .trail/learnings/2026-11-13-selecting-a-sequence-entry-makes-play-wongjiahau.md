---
title: "Selecting a sequence entry makes Play Selection a note selection, and local e2e may reuse another worktree's server"
author: Wongjiahau
date: 2026-11-13
branch: fix/sequence-lyric-occurrence
cost: { minutes: 29, tokens: 116493 }
id: l-039a1660
scope: repo
status: active
src: agent
touches:
  - web/e2e/features/play-selection-follows-sequence-entry.feature
  - web/e2e/features/steps/play-selection-follows-sequence-entry.steps.ts
  - web/src/hooks/useAppSelectionAndNavigation.ts
  - web/src/hooks/useMeasureAudioPlayback.ts
  - web/src/hooks/usePoentaroWorker.ts
  - web/src/hooks/usePoentaroWorkerActions.ts
  - web/src/hooks/useSequenceNavigation.ts
  - ".github/workflows/ci.yml"
  - ARCHITECTURE.md
  - Cargo.lock
---

## Decided
Play Selection plays a selected `sequence` entry as that occurrence (its WITH/WITHOUT/ONLY), while the selection still covers exactly the entry's measures. The sequence selection is never cleared when the user selects something else, so the measure check is what keeps a later, different selection playing as written.

## Watch out
- Clicking a Sequence toolbar entry also selects its lines in Monaco, so `notePlaybackSelectionActive` is true and the Play Selection button calls `playNoteSelection`, not `playSelectedMeasures`. A fix only in `playSelectedMeasures` does nothing for this case.
- The Rust singing path already sang the chosen line correctly for an entry range. The bug for issue #155 was only that the web never passed the entry.
- `playwright.config.ts` uses `reuseExistingServer: true` on port 5183. If another worktree's e2e Vite is still up on 5183, local e2e runs silently test that worktree's code. Check `lsof -iTCP:5183` first, or run with a scratch config on another port.

In the code: web/src/hooks/useMeasureAudioPlayback.ts (`playSelectedSequenceEntry`), web/src/hooks/useSequenceNavigation.ts (`coveredMeasures`), web/e2e/features/play-selection-follows-sequence-entry.feature
