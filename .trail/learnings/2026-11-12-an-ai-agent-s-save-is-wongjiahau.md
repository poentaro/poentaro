---
title: "An AI agent's save is refused for any score error, even ones the editor tolerates, and the comment migration skips old section-syntax files"
author: Wongjiahau
date: 2026-11-12
branch: feat/agent-protocol
id: l-b9162b74
scope: repo
status: active
src: agent
touches:
  - ".github/workflows/ci.yml"
  - ARCHITECTURE.md
  - Cargo.lock
  - Cargo.toml
  - crates/live-share-worker/Cargo.toml
  - crates/live-share-worker/cloudflare.config.ts
  - crates/live-share-worker/src/app_log.rs
  - crates/live-share-worker/src/crypto.rs
  - crates/live-share-worker/src/db/e2e_reset.rs
  - crates/live-share-worker/src/db/files.rs
---

## Decided
- `POST /agent` refuses to save a score with any error-level diagnostic: what doesn't fit the document's shape, and the compiler's recoverable errors too (e.g. "measure has no data lines", "unknown directive"). The web editor still saves such text, since a person is mid-edit; an agent should only leave valid scores behind. Warnings never block.
- Converting stored `#` comments to `##` holds back any file that still doesn't read as an anyx document after converting, and lists it by name.

## Watch out
- The e2e suite's `DEFAULT_CLOUD_FILE_CONTENT` is not a valid score (a bare note line outside any part, and the old `(bpm=… )` directive form), so the agent rejects it. Agent e2e steps write their own valid score (`[M] …` lines).
- Old section-syntax files use `# metadata`, `# parts`, `# score` as headers. `anyx::migrate_comments` would turn those into `##`, which means nothing there. As of 2026-11-13, production's only `#` files are 3 such files (Rasa Sayang, untitled 7, 快樂天堂), so the conversion currently changes nothing.
- e2e fixtures that used `# ...` as comments now produce a line-string error; use `##`.

In the code: crates/live-share-worker/src/agent.rs (`score_errors`), tools/migrate-comments, scripts/migrate-comments-d1.py, web/e2e/features/steps/agent-protocol.steps.ts
