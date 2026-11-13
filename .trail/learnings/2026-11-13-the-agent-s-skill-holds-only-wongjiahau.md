---
title: "The agent's skill holds only how to send a request; the schema comes from SCHEMA at session start"
author: Wongjiahau
date: 2026-11-13
branch: feat/agent-protocol
cost: { minutes: 28, tokens: 48860 }
id: l-f8fe916f
scope: repo
status: active
src: agent
touches:
  - ARCHITECTURE.md
  - Cargo.lock
  - Cargo.toml
  - crates/live-share-worker/src/agent.rs
  - crates/live-share-worker/src/app_log/agent.rs
  - crates/live-share-worker/src/handlers/agent.rs
  - crates/live-share-worker/src/handlers/agent/files.rs
  - crates/poentaro-grammar/Cargo.toml
  - crates/poentaro-protocol/Cargo.toml
  - crates/poentaro-protocol/schema/files.anyx
---

## Decided
- The AI agents prompt has the agent save the API key to a file only the owner can read and install a global `poentaro` skill. The skill never contains the schema or the score syntax. It says to send `SCHEMA` (anyx's built-in introspection call) at the start of each session, and to call `score/syntax` before writing a score, so the skill never goes stale when the schema changes.
- Everything the agent reads comes through `POST /agent` as protocol calls. The user asked for the old `GET /agent` (the guide) and `GET /agent/syntax` to move into the schema, since anyx is transport-agnostic and HTTP routes are Poentaro's concern.
- The skill is written to `~/.agents/skills/poentaro/SKILL.md`, which most agents (Codex, Gemini CLI, Copilot, OpenCode) read. The user wanted it to cover most agents, not only Claude.

## Watch out
- Claude Code does not read `~/.agents/skills`, only `~/.claude/skills`, so the prompt symlinks the skill folder there.
- The key must stay out of the skill text. The skill reads it with `$(cat ~/.config/poentaro/key)`, and a test asserts the skill part has no key.

In the code: crates/live-share-worker/src/agent_prompt.txt (template), api_keys::agent_prompt, crates/poentaro-protocol/schema/score.anyx, anyx commit 6ad957b (SCHEMA).
