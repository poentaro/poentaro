---
name: next-issue
description: Work on the next open GitHub issue that no agent has taken yet — claim it with a comment, implement it in a worktree, and open a PR whose description has annotated before/after screenshots, then hand over a local server with production data and a fixture that demonstrates the change. Use when asked to "work on the next issue".
---

# Next issue

1. Pick the next open issue that isn't already being worked on by an agent
   (no "taken" comment, no open PR for it).
2. Leave a comment on the issue saying it's taken.
3. Implement it in a git worktree, then open a PR from that worktree.
4. The PR description must contain annotated before/after screenshots
   (the changed area marked, e.g. with a red box).
5. Hand over the local test setup using the steps below.

## Local test handoff

Do this for the initial PR handoff and whenever the user asks to start or
restart the dev server to try the PR. Start both the web app and the local
live-share worker from the PR worktree. Follow “Running the app locally”
and “Set up the test fixture for the user” in that worktree’s `AGENTS.md`.

1. Once the worker is ready, run `just copy-prod-db` from the worktree on
   every startup. The recipe reuses an existing local copy. Read from
   production and write fixture changes only to the worktree’s local database.
2. Prepare the local state that demonstrates the PR. Create or update the
   relevant local file, records, or settings as needed. Automatically sign
   in as `hou32hou@gmail.com` using that account's identity in the local copy
   of production, even when the fixture can be opened anonymously. Follow
   the local session setup in `AGENTS.md` with a fresh token bound to the
   local worker. Serve the signed-in preview on loopback and keep tokens and
   scratch configuration out of the PR.
3. Build the URL using the actual Vite port. If the fixture is a file, include
   its URL-encoded name in `?file=<name>` and add any other supported URL
   state that opens the relevant view.
4. Open the exact URL in a fresh browser and confirm it reaches the prepared
   fixture already signed in as the requested account. Give the user that URL
   and any remaining steps to see the change.

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
