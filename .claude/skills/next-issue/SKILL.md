---
name: next-issue
description: Work on the next open GitHub issue that no agent has taken yet — claim it with a comment, implement it in a worktree, and open a PR whose description has annotated before/after screenshots, then hand over a local server (prod data copied, signed in) already showing the change. Use when asked to "work on the next issue".
---

# Next issue

1. Pick the next open issue that isn't already being worked on by an agent
   (no "taken" comment, no open PR for it).
2. Leave a comment on the issue saying it's taken.
3. Implement it in a git worktree, then open a PR from that worktree.
4. The PR description must contain annotated before/after screenshots
   (the changed area marked, e.g. with a red box).
5. Start the local server from the worktree and hand over a URL that already
   shows the change, signed in as `hou32hou@gmail.com`. Follow "Set up the
   test fixture for the user" in `AGENTS.md`: the user should not have to
   write content, create data or sign in.
   - Run `just dev`, then `just copy-prod-db` once the worker is up, so the
     local database holds production's accounts and files. Each worktree has
     its own database, so a fresh worktree needs its own copy. Never write
     to production.
   - Sign in as `hou32hou@gmail.com` automatically. Look up its `user_id` in
     `password_credentials` (by `email`) in the local D1 SQLite file
     (`crates/live-share-worker/.wrangler/state/v3/d1/miniflare-D1DatabaseObject/<hash>.sqlite`).
     Make a random token and insert a `sessions` row: `token_hash` is the
     token's SHA-256 hex, and `created_at` / `expires_at` are in
     milliseconds. Then serve the web app with a scratch Vite config that
     wraps `web/vite.config.ts` and uses `transformIndexHtml` to set
     `poentaro:account-auth:v2` to `{ "token": "<token>", "email":
     "hou32hou@gmail.com" }` in `localStorage`.
   - Load content that demonstrates the change: a file that reproduces the
     bug, or for a syntax feature a `demo/NN-<feature>.poentaro` (public
     domain only, since demos ship with the app). Vite doesn't watch `demo/`,
     so restart `just dev` after adding one.
   - Deep-link to it with `?file=<name>` (e.g.
     `http://localhost:5173/?file=05-lyrics.poentaro`), plus any other URL
     state that gets closer to the change.
   - Keep the fixture scripts and the scratch config in the scratchpad, out
     of the commit.
   - Open the URL in a fresh browser and confirm that it shows the change
     and that you're signed in. Then give the user the URL and the fewest
     remaining steps to see it.
