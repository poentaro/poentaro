---
title: "Max view lays out one point per CSS pixel and shrinks only its own header, so A4 output never changes"
author: Wongjiahau
date: 2026-11-13
branch: feat/preview-max-view
cost: { minutes: 30, tokens: 121102 }
id: l-8476aa35
scope: repo
status: active
src: agent
touches:
  - ARCHITECTURE.md
  - crates/poentaro-wasm/src/component/guest_impl.rs
  - crates/poentaro-wasm/src/component/guest_selection_and_render.rs
  - crates/poentaro-wasm/src/component/render_and_generate_conversion.rs
  - crates/poentaro-wasm/src/responses.rs
  - crates/poentaro-wasm/src/tests_render.rs
  - crates/poentaro-wasm/src/tests_render_invalid_part.rs
  - crates/poentaro-wasm/wit/world.wit
  - src/document_render.rs
  - src/grid_layout/expand.rs
---

## Decided
Max view (the continuous page) maps the preview's width 1 CSS px to 1 pt. That draws the score at three quarters of print size: more measures per line than true size, and still larger than an A4 page shrunk onto a phone.
The header (title, subtitle and author fonts, part legend columns) is shrunk to fit only on a continuous page. Fixed-size pages keep the score's sizes exactly, so print, export and A4 preview output are byte-identical to before.

## Tried and ruled out
0.75 pt per CSS px (true size): on a 390px phone only two measures fit per line, and an A4-sized title ran off the screen while the legend columns overlapped.

## Watch out
A narrow continuous page needs the header fit. Without it, any header element sized for A4's 595pt overflows. The sequence rows and the running title are not fitted yet.

In the code: `page_layout_from_wit` in crates/poentaro-wasm/src/component/render_and_generate_conversion.rs (px → pt), `PageLayout::fit_header` and src/grid_layout/header_fit.rs.
