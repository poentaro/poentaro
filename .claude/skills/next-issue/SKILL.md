---
name: next-issue
description: Work on the next open GitHub issue that no agent has taken yet — claim it with a comment, implement it in a worktree, and open a PR whose description has annotated before/after screenshots, then hand over a local server with production data and a fixture that demonstrates the change. Use when asked to "work on the next issue" or when the user approves its PR for CI follow-through and merge.
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
live-share worker from the PR worktree. Confirm any instance you reuse serves
that worktree, and leave the verified instance running for the user. Follow “Running the app locally”
and “Set up the test fixture for the user” in that worktree’s `AGENTS.md`.

1. Once the worker is ready, run `just copy-prod-db` from the worktree on
   every startup. The recipe reuses an existing local copy. Read from
   production and write fixture changes only to the worktree’s local database.
2. Prepare the local state that demonstrates the PR. Reuse the screenshot or
   regression fixture when possible. Create or update the relevant local file,
   records, or settings as needed. Automatically sign
   in as `hou32hou@gmail.com` using that account's identity in the local copy
   of production, even when the fixture can be opened anonymously. Follow
   the local session setup in `AGENTS.md` with a fresh token bound to the
   local worker. Serve the signed-in preview on loopback and keep tokens and
   scratch configuration out of the PR.
3. Build the URL using the actual Vite port. If the fixture is a file, include
   its URL-encoded name in `?file=<name>` and add any other supported URL
   state that opens the relevant view. If the chat client drops a URL fragment,
   use a temporary local path that opens the fixture.
4. Open the exact URL in a fresh browser and confirm it reaches the prepared
   fixture already signed in as the requested account. Verify it without
   relying on state seeded by a test runner. Give the user that URL, a plain
   address they can paste if the chat client blocks the link, and any
   remaining steps to see the change.

## Once the user approves the PR

When the user says "approved", "lgtm", or "ship it" in chat, or leaves an
approving GitHub review, treat that as authorization to take the active PR
through CI and merge it. Continue in the same turn until GitHub confirms the
merge. Ask again only if resolving a failure would materially change what
the user approved or a blocker requires their input.

1. **Watch CI and fix failures.** Confirm the PR targets `master`.
   For a stacked PR, wait for its prerequisite PR to merge, then confirm the
   intended base. Use `gh pr checks <n> --repo poentaro/poentaro --watch`.
   On failure, read the logs with `gh run view <run-id> --log-failed`,
   fix the cause in the PR worktree, commit and push, and watch CI for the
   new head. Investigate e2e failures and rerun a job only for a confirmed
   transient failure. Resolve merge conflicts in the worktree and wait for
   CI on the updated head.
2. **Squash-merge after CI passes.** Confirm the expected PR CI workflow ran
   successfully on the current head and all required checks passed. Then use
   `gh pr merge <n> --repo poentaro/poentaro --squash --match-head-commit <sha>`
   with the verified SHA. If the head changes, watch the new head before
   merging. If GitHub queues the merge, stay on it until the PR is merged.
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
