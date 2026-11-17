---
title: "Keep manual dev servers off port 5183, because e2e quietly reuses whatever is listening there"
author: Wongjiahau
date: 2026-11-13
branch: feat/preview-max-view
cost: { minutes: 30, tokens: 112266 }
id: l-a07c9145
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
  - crates/live-share-worker/queries/delete_api_key.sql
  - crates/live-share-worker/queries/delete_api_keys_of_users.sql
  - crates/live-share-worker/queries/get_api_key.sql
  - crates/live-share-worker/queries/get_file_by_name.sql
---

## Watch out
Playwright's web server has `reuseExistingServer: true` on port 5183. A hand-started Vite there (e.g. one whose config auto-signs in and forces the cloud backend for a PR handover) gets reused by `pnpm test:e2e`. Then every scenario that seeds `poentaro:files:v1` in localStorage silently lands on the demo file and fails, even untouched ones like the harmonica view. Start handover servers on another port (5185 worked), or stop them before running e2e.

Preview e2e steps also need a desktop width: at 768px and below the app shows the mobile layout, and the preview reports no systems.

In the code: web/playwright.config.ts (`baseURL`, `webServer[0]`), the MOBILE_BREAKPOINT_QUERY in web/src/hooks/useMediaQuery.ts.
