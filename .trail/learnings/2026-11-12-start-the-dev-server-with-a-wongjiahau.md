---
title: Start the dev server with a demo score that shows the change
author: Wongjiahau
date: 2026-11-12
branch: feat/sing-lyrics
cost: { minutes: 11, tokens: 25376 }
id: l-332d322f
scope: repo
status: active
src: agent
touches:
  - demo/15-singing.poentaro
  - ARCHITECTURE.md
  - Cargo.lock
  - Cargo.toml
  - crates/poentaro-singer-wasm/Cargo.toml
  - crates/poentaro-singer-wasm/src/bindings.rs
  - crates/poentaro-singer-wasm/src/lib.rs
  - crates/poentaro-singer-wasm/wit/world.wit
  - crates/poentaro-singer/Cargo.toml
  - crates/poentaro-singer/src/lib.rs
---

## Decided
When the dev server is launched to show a PR, it must open with content that demonstrates the change, because otherwise the reviewer has to write an example before they can see anything. For a syntax feature, add a per-feature demo score (public-domain content only, because demos ship with the app), which then appears under the app's Demo folder. Tell the user which file to open and what to click.

## Tried and ruled out
Handing over the dev URL with only the default content. It was ruled out because the user then had to ask why no fixture showing the change was loaded.

## Watch out
Vite does not watch the demo folder because it sits outside web/. Restart the dev server after adding a demo, otherwise the new file isn't listed.

In the code:
- Demos: demo/NN-<feature>.poentaro, loaded by an eager import.meta.glob in web/src/defaultSource.ts
- Restart: stop `just dev`, then run it again
