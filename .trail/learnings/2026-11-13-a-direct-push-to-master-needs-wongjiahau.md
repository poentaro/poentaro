---
title: "A direct push to master needs the CI ruleset switched off briefly, and only when the user asks"
author: Wongjiahau
date: 2026-11-13
branch: master
cost: { minutes: 5, tokens: 5412 }
id: l-c6b1d26e
scope: repo
status: active
src: agent
touches:
  - ".github/workflows/ci.yml"
---

## Watch out
- The "master requires CI" ruleset requires its status checks (the jobs in `.github/workflows/ci.yml`) to pass, so a plain push to master is rejected, even for a markdown-only change like `.claude/skills/next-issue/SKILL.md`.

## Decided
- By default, changes go through a PR with auto-merge.
- When the user explicitly asks for a direct push to master, set the ruleset's enforcement to disabled, push, and set it back to active straight away in the same command, so it is never left off. Check that the push is a fast-forward of origin/master first.
- When the main checkout is mid-conflict or behind origin, make the commit in a temporary detached worktree on origin/master instead of touching the shared tree.

In the code: ruleset id 24567755. `gh api -X PUT repos/jianpu-generator/jianpu-generator/rulesets/24567755 -f enforcement=disabled`, push, then the same with `enforcement=active`. Rejected push text: "push declined due to repository rule violations ... 10 of 10 required status checks are expected".
