---
title: "Fake cloud failures in e2e as protocol method errors, and a taken destination folder moves the batch to the next numbered folder"
author: Wongjiahau
date: 2026-11-14
branch: feat/anyx-protocols
cost: { minutes: 30, tokens: 121102 }
id: l-cd686d95
scope: repo
status: active
src: agent
touches:
  - crates/poentaro-wasm/package.json
  - web/e2e/e2eAccounts.ts
  - web/e2e/e2eEmails.ts
  - web/e2e/features/steps/account-auth.steps.ts
  - web/e2e/features/steps/agent-protocol.steps.ts
  - web/e2e/features/steps/request-trace-id.steps.ts
  - web/e2e/features/steps/session-ended.steps.ts
  - web/e2e/features/steps/synced-share-multi-device.steps.ts
  - web/e2e/global-setup.ts
  - web/e2e/protocolClients.ts
---

## Decided
- E2e steps that make the worker refuse a cloud call answer the page's `POST /app` with a method error (e.g. `trashed`) encoded in Rust, not an HTTP status. A non-2xx status is now a transport failure the outbox retries, so faking "permanent" with a 400 no longer works.
- When `folders/move` answers that the destination folder exists, the whole batch moves into the next numbered folder (`New` -> `New 2`) instead of merging into it. A file someone made inside the destination meanwhile is renamed on its own.

## Watch out
- Loading the cloud with one `files/read` per file sent ~23k `/app` requests in one e2e run; with 5 workers, loads missed the 15s waits and many unrelated scenarios failed. Content has to load lazily (or in one request), not per file at load.
- The New file/folder dialog validates names in Rust while rendering; under load it can open before the main-thread wasm is ready and crash the page. It now waits on `useWasmReady`.
- `--repeat-each` on cloud scenarios fails on later repeats because the shared e2e account keeps the first run's folders; judge by the first run.

In the code: web/e2e/appCalls.ts (routeAppCall, answerCall, refuseCall, rewriteAnswers), crates/cloud-outbox/src/batch_adjust.rs, web/src/storage/outbox/sendMessage.ts (moveFolder), web/src/hooks/useWasmReady.ts, web/src/storage/cloudBackendListing.ts (listFiles).
