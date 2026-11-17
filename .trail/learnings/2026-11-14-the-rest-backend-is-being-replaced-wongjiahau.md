---
title: "The REST backend is being replaced by separate anyx protocols for the public, the web app, agents and tests"
author: Wongjiahau
date: 2026-11-14
branch: master
cost: { minutes: 1, tokens: 2582 }
id: l-556618f4
scope: repo
status: active
src: agent
---

## Decided
- Four schemas, one per audience: `poentaro public` (no credential: sign up, sign in, open a link, origin handoff), `poentaro app` (session token: everything the web app does), the existing agent schema `poentaro` (API key, unchanged), and `poentaro test` (e2e reset, served only when enabled). Separate so a leaked API key can only edit scores, and so agent `write` can refuse unparsable scores while the app's autosave accepts them.
- Credentials travel in `Authorization: Bearer` and are checked before `handle`; a bad one is HTTP 401 for the whole request, not a per-method error.
- The app addresses files by id (the browser still makes new ids), agents by path: the offline outbox queues edits by id, and a file moved elsewhere while a tab is offline must not break its queued writes.
- The app loads file content lazily; offline, a file never opened on this device shows as unavailable.
- Old tabs: clients send a schema hash header, a mismatch gets 409 and a reload prompt; schema changes need not stay backward compatible.
- Generated code (all four schemas, agent included) is built at build time, not checked in.
- anyx gets `RETURNS NOTHING` instead of methods returning filler. anyx changes go straight to its upstream master.
- One PR for the migration; REST is deleted in a follow-up once app_log shows no REST calls, because tabs on the old bundle send no schema hash.

## Tried and ruled out
- One schema for app and agents: wider powers for API keys, and the two `write` behaviors conflict.
- Paths for the app: offline queued writes break when a file is moved by another device or an agent.
- Server-made file ids: the outbox would need to remap queued ops after each create.

## Watch out
- The generated TS client sends one call per request; there is no batching.
- Nothing checks the protocol VERSION at runtime.

In the code: plan in TODO-anyx-protocols.md; agent schema in crates/poentaro-protocol/schema/.
