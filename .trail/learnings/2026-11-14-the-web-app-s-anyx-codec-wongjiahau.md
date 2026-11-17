---
title: "The web app's anyx codec is two exports of the wasm component, not anyx's generated wasm code"
author: Wongjiahau
date: 2026-11-14
branch: feat/anyx-protocols
id: l-cc631f00
scope: repo
status: active
src: agent
touches:
  - Cargo.lock
  - TODO-anyx-protocols.md
  - crates/poentaro-protocol/build.rs
  - crates/poentaro-wasm/Cargo.toml
  - crates/poentaro-wasm/src/component/guest_impl.rs
  - crates/poentaro-wasm/src/component/mod.rs
  - crates/poentaro-wasm/wit/world.wit
  - web/package.json
---

## Decided
The web app encodes requests and decodes responses for the app, public and test protocols through two exports of poentaro-wasm, `protocol-encode-request` and `protocol-decode-response`, which take the protocol as a WIT enum and call `anyx_protocol::json` with the schema generated in poentaro-protocol.

Why: `anyx-protocol-generate wasm` writes `#[wasm_bindgen]` functions, but poentaro-wasm is a WIT component built with `cargo component` and transpiled by jco, so those functions would never be exported. One pair of exports serves every schema, so adding a schema needs no new wasm glue.

## Watch out
poentaro-protocol's build.rs generates both the Rust modules and the TypeScript clients into OUT_DIR. The `poentaro-protocol-typescript` binary copies the TypeScript into web/src/generated/protocol, and `build:worker-types` runs it, so CI and lefthook needed no change. The generator's code starts with `#![allow(...)]`, which `include!` refuses, so build.rs wraps each schema's code in its `pub mod` itself.

In the code: crates/poentaro-protocol/build.rs, crates/poentaro-protocol/src/bin/poentaro-protocol-typescript.rs, crates/poentaro-wasm/src/component/guest_protocol.rs, crates/poentaro-wasm/wit/world.wit (protocol enum), web/package.json (build:protocol).
