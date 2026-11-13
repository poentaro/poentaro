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

## Once the user approves the PR

Approval means the user says so in chat (e.g. "approved", "lgtm", "ship it")
or leaves an approving review on GitHub. Then, without pausing between steps:

1. **Set auto-merge.** First confirm the PR's base is `master`
   (`gh pr view <n> --repo poentaro/poentaro --json baseRefName`); if it is
   stacked on another PR's branch, say so and handle the bottom of the stack
   first. Confirm the `master` ruleset still requires the CI checks, since
   auto-merge without them has merged PRs before CI finished. Then
   `gh pr merge <n> --repo poentaro/poentaro --auto --squash`.
2. **Watch until merged, fixing CI failures.** Wait on the checks
   (`gh pr checks <n> --repo poentaro/poentaro --watch`, in the background or
   via Monitor, not a polling loop). When a check fails, read its log
   (`gh run view <run-id> --log-failed`), fix the cause in the worktree,
   commit and push, then keep watching. Investigate e2e failures rather than
   re-running them away; re-run a job only when the log shows a known flake.
   If the branch falls behind `master` with conflicts, rebase and push.
   Stop and ask only if a fix would change what the user approved.
3. **Notify on merge.** When `gh pr view <n> --json state` reports `MERGED`,
   send a push notification (PushNotification) and tell the user the PR is
   merged, then ask them to reply "ok" to start the cleanup. Do nothing more
   until they do.
4. **Clean up** after the "ok":
   - Stop every server started for this issue (dev servers from `just dev`,
     `cf dev` workers, Vite, Playwright runs), checking with `lsof -i` that
     their ports are free.
   - Remove the worktree (`git worktree remove <path>`, then
     `git worktree prune`) and delete the local branch; the remote branch is
     deleted on merge, otherwise delete it too.
   - Delete temporary repos, clones and scratch files made for the issue
     (e.g. the fixture scripts and scratch Vite config in the scratchpad, or
     anything in `/tmp`).
   - In the main checkout, `git fetch` and fast-forward `master` only if it
     has no local changes; other sessions may share that tree.
   - Report what was removed and anything left behind on purpose.
5. **Close this herdr pane** as the very last action, since it ends this
   session: `herdr pane close "$HERDR_PANE_ID"`. Skip it if `HERDR_PANE_ID`
   is unset (not running inside herdr).
