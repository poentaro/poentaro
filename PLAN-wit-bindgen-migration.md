# Plan: migrate `poentaro-wasm`'s JS boundary from `wasm-bindgen`/`tsify` to `wit-bindgen`/`cargo-component`/`jco`

## Status

**Phase 1 done**, on branch `wit-bindgen-migration` (not yet merged to
`master`). `crates/poentaro-wasm/Cargo.toml` drops `[lints] workspace = true`
for a local lint table (`unsafe_code`/`unreachable_pub`/
`unused_qualifications` relaxed, everything else copied verbatim) and gains
`[package.metadata.component]`/`wit/` per the original plan. `cargo
component build` now succeeds against the **real** crate (not a toy
example), gated via a new `wasm-bindgen-boundary` Cargo feature (on by
default, so `wasm-pack build`/tests/clippy are all unaffected — verified).
Two second-order blockers turned up beyond what the plan anticipated and
are now fixed, both load-bearing for any future phase that touches these
files:

1. `wasm-bindgen`/`serde-wasm-bindgen`/`tsify` had to become **optional**
   dependencies (`dep:` + the boundary feature), not just cfg-gated at the
   call site — merely linking wasm-bindgen's rlib retains its internal
   `JsValue`-family `#[wasm_bindgen]` definitions and leaves
   `__wbindgen_placeholder__` import stubs even with zero reachable
   `#[wasm_bindgen]` call sites of our own (confirmed: source-only gating
   still failed identically in both debug and `--release`).
2. Tsify's `#[tsify(into_wasm_abi)]` derive macro emits its own internal
   `#[wasm_bindgen]` struct on every derived type, not just on exported
   functions (confirmed in `tsify-macros` source) — so every Tsify-derived
   type in `types.rs`/`metadata_types.rs`/`svg_types.rs`/
   `lyric_selection_types.rs`/`note_selection_types.rs`/
   `selection_range/types.rs` needed its derive/`#[tsify(...)]` attributes
   converted to `#[cfg_attr(feature = "wasm-bindgen-boundary", ...)]`
   (mechanical, ~50 sites; `types_export.rs` was already whole-module-gated
   so needed no per-site change). All 41 `#[wasm_bindgen] fn` exports
   (`lib.rs` → new `wasm_boundary.rs`, `lib_wav.rs`, `lib_mp3.rs`,
   `lib_pdf.rs`, `lib_midi.rs`, `lib_import.rs`, `share_payload.rs`,
   `selection_range/mod.rs`'s `resolve_selection_range`) and their
   feature-specific `responses_*`/`types_export` plumbing in
   `responses.rs`/`types.rs` are now gated the same way. `wasm-pack build`,
   `cargo test -p poentaro-wasm` (82 passed), and `cargo clippy` (both
   boundary-on and boundary-off) all still pass clean.

`crates/poentaro-wasm/src/component.rs` is the `wit-bindgen` skeleton itself:
a trivial `greet(name: string) -> string` world (`wit/world.wit`), proving
the pipeline works end-to-end on the retrofitted crate. Real types/functions
still need Phase 2/3. `src/bindings.rs` (cargo-component's scratch output)
is gitignored per the plan.

**Phase 2 done.** Every type in `crates/poentaro-wasm/src/{types,
note_selection_types, lyric_selection_types, metadata_types, svg_types,
types_export, responses*, selection_range/types}.rs` is now ported into
`crates/poentaro-wasm/wit/world.wit` as flat world-level `record`/`variant`/
`enum` declarations (the flat-vs-`interface` decision: **flat**, per the
plan's recommendation — the file stayed easy to navigate at full size, no
need for `interface` grouping). No `#[wasm_bindgen] fn`s were touched;
`crates/poentaro-wasm/src/component.rs`'s `Guest` impl still only implements
`greet`. `cargo component build` (debug and `--release`), `cargo build`/
`cargo test -p poentaro-wasm` (82 passed), and `cargo clippy` all still pass
clean on both the `wasm-bindgen-boundary`-on (default) and -off
(`--no-default-features --features wav,mp3,pdf,midi --target
wasm32-unknown-unknown`) builds — matching Phase 1's verification bar
exactly (clippy without `--all-targets`; `--all-targets` fails identically
on `master` today, pre-existing and unrelated, because the boundary-gated
`tests_*.rs` modules aren't excluded from the target set when the feature
that gates their imports is off).

**Phase 3, group 1 done** (`group_note_selection`/`group_lyric_selection`).
`wit/world.wit` gained two `export`s (placed right after their response
`variant`s, not grouped separately) and `component.rs`'s `Guest` impl now
implements both alongside `greet`, converting WIT-generated
records/variants to/from the existing `types.rs`/`note_selection_types.rs`/
`lyric_selection_types.rs` shapes and calling the same
`crate::responses::group_note_selection_response`/
`group_lyric_selection_response` the old `#[wasm_bindgen] fn`s call — that
pair of functions is genuinely untouched, both mechanisms coexist. No
surprises versus the spike: the WIT-generated Rust type names
(`NoteSpan`, `NoteCellIn`, `GroupNoteSelectionResponse`,
`GroupNoteSelectionResponseOk`, `NoteSelectionRun`, and their lyric
counterparts) collide by name (not by path) with the crate's own
same-named types, but since `component.rs` never `use`s either set
unqualified — WIT-generated names are bare (macro-injected into this
module's scope), the crate's own types are always `crate::`-qualified —
there's no ambiguity. `usize`↔`u32` conversion is a plain `as` cast at
each field, per Phase 2's mapping note. `cargo component build`
(debug and `--release`, default `wasm32-wasip1` target — matching how
Phase 1/2 were verified; Phase 5 is what pins `wasm32-unknown-unknown` for
the real asset build), `cargo build --target wasm32-unknown-unknown`,
`cargo test -p poentaro-wasm --features wav,mp3,pdf,midi` (82 passed), and
`cargo clippy` on both the boundary-on (default) and boundary-off
(`--no-default-features --features wav,mp3,pdf,midi --target
wasm32-unknown-unknown`) builds all pass clean.

**Phase 3, group 2 done** (`list_note_spans`/`list_lyric_spans`/
`list_measure_spans`). `wit/world.wit` gained three `export`s, each placed
right after its own response `variant` (they're scattered across the
`types.rs` section of the file rather than adjacent to each other, since
that's where their response types were already declared — group 1's "next
to the response variant" convention still holds per-export, it just doesn't
imply the three are contiguous with each other here). `component.rs`'s
`Guest` impl now implements all three alongside the group 1 pair and
`greet`, calling the same `crate::responses::list_note_spans_response`/
`list_lyric_spans_response`/`list_measure_spans_response` the old
`#[wasm_bindgen] fn`s call — untouched, both mechanisms coexist. No
surprises versus group 1's established pattern: `MeasureSpanOut`'s fields
matched WIT's `measure-span` record exactly (checked before converting, per
the task's instruction — no shape drift since Phase 2). `enabled_tracks:
Option<Vec<String>>` round-trips cleanly through WIT's
`option<list<string>>` with a plain `enabled_tracks.as_deref()` at the call
site — confirmed `None` and `Some(vec![])` stay distinct through
`wit-bindgen`'s generated `Option<Vec<String>>` Rust binding (no flattening
to an empty-vec-means-None convention). One new, mechanical finding:
`NoteSpanOut`/`LyricSpanOut` are all-`Copy`-field structs, so their
`*_to_wit` conversion functions tripped
`clippy::needless_pass_by_value` (this crate's clippy config, denied) where
group 1's `*_from_wit` functions (WIT type -> crate type, consumed by
value, not flagged) didn't — fixed by taking `&NoteSpanOut`/`&LyricSpanOut`
and mapping over `.iter()` instead of `.into_iter()`; `MeasureSpanOut`/
`SectionRangeOut`/`SequenceEntryOut` weren't flagged since they contain
`String`/`Option<String>` fields, which are consumed, not just copied.
`cargo component build` (debug and `--release`), `cargo build --target
wasm32-unknown-unknown`, `cargo test -p poentaro-wasm --features
wav,mp3,pdf,midi` (82 passed), and `cargo clippy` on both the
boundary-on (default) and boundary-off (`--no-default-features --features
wav,mp3,pdf,midi --target wasm32-unknown-unknown`) builds all pass clean —
same verification bar as group 1.

**Phase 3, group 3 done** (`list_parts`/`list_symbols`/`rename_symbol`/
`get_measure_index_at_offset` — named to match `wasm_boundary.rs`'s actual
fn name, not the plan prose's `measure_at_offset` shorthand). `wit/world.wit`
gained four `export`s, each placed right after its own response `variant`
(same per-export placement convention as groups 1/2), plus a new
`instrument-info` record (`value`/`category`/`source`/`role`/
`articulation`, all `string`, no `Option` fields) placed just before
`list-parts-response`'s export since `list_parts` is first in this group's
order — the first WIT input shape ported that wasn't already a
`serde_wasm_bindgen`-decoded `Vec<T>` matching an existing internal type
1:1; all four functions take `list<instrument-info>` in place of today's
`raw_instruments: JsValue` decoded via
`serde_wasm_bindgen::from_value::<Vec<InstrumentInfo>>(...).unwrap_or_default()`.
`component.rs`'s `Guest` impl now implements all four alongside groups 1/2
and `greet`, calling the same
`crate::part_declarations::list_parts_response`/
`crate::symbols::list_symbols_response`/`rename_symbol_response`/
`crate::responses::get_measure_at_offset_response` the old
`#[wasm_bindgen] fn`s call — untouched, both mechanisms coexist.
`list_part_declarations` was deliberately left unported (shares the same
`InstrumentInfo`-input shape, but isn't in this group's list). Two new
findings beyond groups 1/2:

1. **`cargo component build` needs `--no-default-features` on this crate**
   (plus the real feature set), confirmed necessary starting this group
   when building from inside `crates/poentaro-wasm/` directly (`cargo
   component build --features wav,mp3,pdf,midi` alone still links the
   `wasm-bindgen-boundary` feature, which is on by default, and fails with
   the same `__wbindgen_placeholder__` error Phase 1 first hit) — not a new
   regression, just the first time this session ran the raw command instead
   of copying a prior verified invocation verbatim; `Cargo.toml`'s own
   feature-flag doc comment already said as much.
2. **A 9-argument generated export shim** — `rename_symbol`'s WIT-level ABI
   lowering (`source`/`old-name`/`new-name`/`raw-instruments` are each
   `string`- or `list`-shaped, lowering to a `ptr, len` pair; `kind` lowers
   to one plain discriminant: 4×2 + 1 = 9) tripped this crate's
   `too_many_arguments` threshold (6) on code attributed to the
   `wit_bindgen::generate!` macro's expansion rather than any function this
   phase actually wrote — same macro-attribution situation Phase 3's
   `mem_forget` allow already covered, fixed the same way: added
   `clippy::too_many_arguments` to `component.rs`'s existing module-level
   `#![allow(...)]`.
3. `MeasureAtOffsetResponse`'s `*_to_wit` conversion function tripped
   `clippy::needless_pass_by_value` even though its only data-carrying case
   (`Ok { measure_index: usize }`) is a single `Copy` field — same group 2
   finding, same fix (`&crate::types::MeasureAtOffsetResponse` param,
   `*measure_index as u32` at the match arm).

`cargo component build` (debug and `--release`, default `wasm32-wasip1`
target), `cargo build --target wasm32-unknown-unknown`, `cargo test -p
poentaro-wasm --features wav,mp3,pdf,midi` (82 passed), and `cargo clippy` on
both the boundary-on (default) and boundary-off (`--no-default-features
--features wav,mp3,pdf,midi --target wasm32-unknown-unknown`) builds all
pass clean — same verification bar as groups 1/2.

**Phase 3, group 4 done** (`render`/`render_with_highlight_range`). `wit/world.wit`
gained two `export`s, placed right after the `instrument-info` record (the
earliest point where `render-response`, `instrument-info`, and
`measure-range-in` — all three needed by these exports' signatures — are
already declared). `component.rs`'s `Guest` impl now implements both
alongside groups 1-3 and `greet`, calling the same
`crate::responses::render_response`/`render_with_highlight_range_response`
the old `#[wasm_bindgen] fn`s call — untouched, both mechanisms coexist.
This group also implements the arena-flattening conversion for
`SvgDocumentOut`'s recursive tree that Phase 2 flagged but left unbuilt
(`flatten_svg_element`, pre-order, `Group.child-indices` as arena indices).

One new, real (not stylistic) blocker turned up, unlike anything groups 1-3
hit: **a single-word WIT export name literally collides with the
`wasm-bindgen` export of the same name on `wasm32-unknown-unknown`.**
`render`'s kebab-case WIT form and its snake_case Rust form are the exact
same string (no hyphens to keep them apart, unlike every multi-word export
ported so far — `group-note-selection`, `list-note-spans`,
`rename-symbol`, etc. — where the hyphen vs. underscore difference kept
`wit-bindgen`'s and `wasm-bindgen`'s literal export names distinct by
accident). Confirmed via the generated `src/bindings.rs`:
`wit-bindgen`'s raw ABI export lowers to a plain
`#[export_name = "render"]`, identical to `#[wasm_bindgen] pub fn
render`'s own export symbol — both `cargo build --target
wasm32-unknown-unknown --features wav,mp3,pdf,midi` (boundary-on) and the
**real** `wasm-pack build --features wav,mp3,pdf,midi` failed identically
with `error: symbol `render` is already defined`, meaning this would have
broken the actual production build, not just an isolated check. Fixed by
renaming just the WIT export/Guest method — `render` -> `render-svg`,
`render-with-highlight-range` -> `render-svg-with-highlight-range` — a
deliberate, documented deviation from group 3's "WIT export name matches
`wasm_boundary.rs`'s Rust fn name exactly" convention for this pair only;
the old `#[wasm_bindgen] fn`s were not touched. Any future single-word
function name (none remain in the plan's Phase 3 order 5 list) should be
checked against this same risk before assuming the exact-name-match
convention is safe.

`cargo component build` (debug and `--release`,
`--no-default-features --features wav,mp3,pdf,midi`), `cargo build
--target wasm32-unknown-unknown --features wav,mp3,pdf,midi` (now passes,
confirming the rename fixed the collision), `cargo test -p poentaro-wasm
--features wav,mp3,pdf,midi`, and `cargo clippy` on both the boundary-on
(default) and boundary-off (`--no-default-features --features
wav,mp3,pdf,midi --target wasm32-unknown-unknown`) builds all pass clean —
same verification bar as groups 1-3.

**Phase 3, group 5 done** (`generate_midi`/`generate_wav`/`generate_pdf`/
`generate_mp3` and their `split` variants — exactly the 8 functions the
plan's Phase 3 ordering names: `generate_midi`, `generate_split_midis`,
`generate_wav`, `generate_split_wavs`, `generate_pdf`,
`generate_split_pdfs`, `generate_mp3`, `generate_split_mp3s`).
`wit/world.wit` gained eight `export`s, each placed right after its own
response `variant` (all of which Phase 2 already ported as bare types,
unused until now — same per-export placement convention as groups 1-4).
`component.rs`'s `Guest` impl now implements all eight alongside groups 1-4
and `greet`, calling the same `crate::responses::generate_wav_response`/
`generate_split_wavs_response`/`generate_mp3_response`/
`generate_split_mp3s_response`/`generate_pdf_response`/
`generate_split_pdfs_response`/`generate_midi_response`/
`generate_split_midis_response` the old `#[wasm_bindgen] fn`s call —
untouched, both mechanisms coexist.

One new, real blocker turned up, unlike anything groups 1-4 hit: **the
functions this group needed to call didn't exist outside the
`wasm-bindgen-boundary` feature at all**, not just their `#[wasm_bindgen]
fn` wrappers. Every prior group's underlying `crate::responses::*`/
`crate::part_declarations::*`/`crate::symbols::*` function was already
compiled unconditionally (only the `#[wasm_bindgen] fn` wrapper and each
type's `Tsify` derive were feature-gated per-site, per Phase 1's
established conversion). For this group, `types_export.rs` (holding
`GenerateWavResponse`/`GenerateMp3Response`/`GeneratePdfResponse`/
`GenerateMidiResponse`/their split-variant siblings, all with an
unconditional `#[derive(Tsify)]`, not per-field `cfg_attr`'d) and
`responses.rs`'s `responses_wav`/`responses_mp3`/`responses_pdf`/
`responses_midi` submodules (holding `generate_wav_response`,
`generate_split_wavs_response`, etc.) were gated
`#[cfg(all(feature = "wasm-bindgen-boundary", feature = "wav"))]` (etc.) as
*whole modules* in `lib.rs`/`responses.rs` — Phase 1 explicitly skipped
converting `types_export.rs` to the per-site `cfg_attr` pattern at the
time because *"`types_export.rs` was already whole-module-gated so needed
no per-site change"*, since nothing outside the boundary needed to call
into it yet. Fixed by extending Phase 1's already-established mechanical
conversion (whole-module `#[cfg]` → per-site
`#[cfg_attr(feature = "wasm-bindgen-boundary", derive(Tsify))]`/
`#[cfg_attr(..., tsify(into_wasm_abi))]`/`#[cfg_attr(..., tsify(type =
"Uint8Array"))]`, with the module-level `#[cfg]` narrowed down to just its
real feature, e.g. `#[cfg(feature = "wav")]`) to `types_export.rs`,
`responses_wav.rs`, `responses_mp3.rs`, `responses_pdf.rs`,
`responses_midi.rs`, and the corresponding `mod`/`use` declarations in
`lib.rs`/`responses.rs`/`types.rs`. The `#[wasm_bindgen] fn`s themselves in
`lib_wav.rs`/`lib_mp3.rs`/`lib_pdf.rs`/`lib_midi.rs` stayed gated exactly as
before (feature + boundary) — only the underlying response-function/type
modules needed narrowing. Confirmed via `cargo check` on both
`--features wav,mp3,pdf,midi` and
`--no-default-features --features wav,mp3,pdf,midi --target
wasm32-unknown-unknown` before writing any `component.rs`/`wit/world.wit`
changes.

**Scope note, deliberately not folded into this group**:
`generate_wav_for_measure_range`, `generate_mp3_for_measure_range`,
`list_note_timings`, `list_note_timings_for_range`,
`generate_instrument_preview_wav`, and `generate_percussion_preview_wav`
live in the same `lib_wav.rs`/`lib_mp3.rs` files as this group's functions
but are not named anywhere in Phase 3's ordering list (which says exactly
"`generate_midi` / `generate_wav` / `generate_pdf` / `generate_mp3` and
their `split` variants"). They remain unported. Phase 6 assumes every
function is ported before cutover, so a future group (or an explicit scope
decision) is needed to cover these six before Phase 6 can proceed — flagged
here so they aren't silently dropped from the plan's radar.

`cargo component build` (debug and `--release`,
`--no-default-features --features wav,mp3,pdf,midi`), `cargo build --target
wasm32-unknown-unknown --features wav,mp3,pdf,midi`, `cargo test -p
poentaro-wasm --features wav,mp3,pdf,midi` (82 passed), and `cargo clippy` on
both the boundary-on (default, `--features wav,mp3,pdf,midi`) and
boundary-off (`--no-default-features --features wav,mp3,pdf,midi --target
wasm32-unknown-unknown`) builds all pass clean — same verification bar as
groups 1-4.

Two more mapping decisions/findings turned up, both encoded directly as
comments in `wit/world.wit` rather than only here:

1. **`usize` -> WIT `u32`, not `u64`**: not a narrowing compromise — this
   crate only ever targets `wasm32-unknown-unknown` (32-bit), where `usize`
   already *is* 32 bits. Also avoids `u64` mapping to a JS `bigint` in the
   generated bindings, which would silently change every offset/index's JS
   type from a plain `number` today.
2. **`SvgElementOut` is a recursive tree** (`SvgKindOut::Group { children:
   Vec<SvgElementOut>, .. }`) — confirmed the component model cannot
   express this directly (`cargo component build` rejects a
   directly-self-referential WIT record/variant outright: "type
   `svg-element` depends on itself"). Not shape-for-shape portable as-is;
   resolved by flattening the tree into an arena: `svg-document.elements`
   is a pre-order-flattened `list<svg-element>` holding every element at
   every depth exactly once, `svg-document.root-element-indices` holds the
   indices of the document's direct children (what `SvgDocumentOut.elements`
   held directly before), and `svg-group-kind.child-indices` is
   `list<u32>` indices into that same arena instead of nested elements.
   Phase 3's `svg_document_to_out`-equivalent conversion code must produce
   this flattened shape; nothing consumes it yet.

**Phase 3, group 6 done** (`generate_wav_for_measure_range`/
`generate_mp3_for_measure_range`/`list_note_timings`/
`list_note_timings_for_range`/`generate_instrument_preview_wav`/
`generate_percussion_preview_wav` — the six functions group 5's Status entry
flagged as living in the same `lib_wav.rs`/`lib_mp3.rs` files but not named
anywhere in Phase 3's explicit ordering list). `wit/world.wit` gained six
`export`s: `generate-wav-for-measure-range`/`generate-instrument-preview-wav`/
`generate-percussion-preview-wav` right after `generate-wav`'s own export
(all three return `generate-wav-response`), `list-note-timings`/
`list-note-timings-for-range` right after the `note-timings-response`
variant (declared but unused since Phase 2), and
`generate-mp3-for-measure-range` right after `generate-mp3`'s own export —
same per-export placement convention as prior groups. All six are
multi-word names (checked proactively per group 4's finding) so none
collide with `wasm_boundary.rs`'s snake_case export symbols on
`wasm32-unknown-unknown`. `component.rs`'s `Guest` impl now implements all
six alongside groups 1-5 and `greet`, calling the same
`crate::responses::generate_wav_for_measure_range_response`/
`generate_mp3_for_measure_range_response`/`list_note_timings_response`/
`list_note_timings_for_range_response`/
`generate_instrument_preview_wav_response`/
`generate_percussion_preview_wav_response` the old `#[wasm_bindgen] fn`s
call — untouched, both mechanisms coexist. This closes the scope gap group
5 flagged: **Phase 3's function porting is now fully complete.**

**Correction (found while scoping Phase 6, fixed as group 7 below): this
claim was premature.** It was only checked against the plan's own Phase 3
ordering list (`group_note_selection`/`list_note_spans`/.../the group 5/6
six), which was never itself exhaustive against the crate's real
`#[wasm_bindgen] fn` inventory — it silently omitted a whole second tier of
functions living in `wasm_boundary.rs`/`lib_import.rs`/`share_payload.rs`/
`selection_range/mod.rs` that never appeared in Phase 3's suggested-order
bullet list at all (not even as a "not yet in this group" flag the way group
5 flagged group 6's six). A direct grep of every `#[wasm_bindgen] fn` across
the whole crate, cross-checked one by one against `component.rs`'s `Guest`
impl, found 17 more real, live functions with no counterpart — see group 7
immediately below. **The lesson, stated plainly for whichever group's Status
entry is written last**: "fully complete" must mean *"grepped every
`#[wasm_bindgen] fn` in the crate and confirmed each has a `Guest`
counterpart,"* not *"implemented everything the plan's own ordering list
named."* Group 7's own Status entry below re-runs that exhaustive grep as
its final step and names it explicitly as the real completion gate.

**Phase 3, group 7 done** (17 functions found by the exhaustive
`#[wasm_bindgen] fn` inventory cross-check above, none of them named in
Phase 3's original ordering list: `set_layout_fonts`,
`list_part_declarations`, `update_part_declaration`, `shift_part_octave`,
`format_score`, `get_metadata_defaults`, `get_default_lyrics_font_size`,
`get_default_title_font_size`, `get_default_subtitle_font_size`,
`get_default_author_font_size`, `get_default_part_legend_font_size`,
`get_default_page_number_font_size` — all in `wasm_boundary.rs` —
`extract_source_from_svg`/`extract_source_from_pdf` (`lib_import.rs`),
`compress_share_payload`/`decompress_share_payload` (`share_payload.rs`),
and `resolve_selection_range` (`selection_range/mod.rs`)). `wit/world.wit`
gained 17 new `export`s across four new/extended sections — the six
font-size getters and `get-metadata-defaults`/`set-layout-fonts`/
`shift-part-octave`/`format-score` placed right after the pre-existing
`metadata-defaults` record (Phase 2 had already ported that record but never
exported anything against it); `list-part-declarations`/
`update-part-declaration` placed right after the pre-existing
`list-part-declarations-response` variant (same story — ported in Phase 2,
unused until now); a new `// ==== lib_import.rs ====`/`// ==== share_payload.rs
====` pair of sections for `extract-source-from-svg`/`extract-source-from-pdf`/
`compress-share-payload`/`decompress-share-payload`, placed right after the
`generate-split-mp3s` export (the natural end of the wav/pdf/midi/mp3 run);
and `resolve-selection-range` at the very end of the file, right after the
pre-existing `resolve-selection-range-response` variant — `clickable-element-id`/
`note-cell-out`/`lyric-cell-out`/`resolve-selection-range-response` were
*also* already fully ported in Phase 2 and sitting unused, the same
pre-ported-but-unexported situation as `metadata-defaults`/
`list-part-declarations-response`. All 17 names are multi-word (checked
proactively per group 4/6's finding) so none collide with
`wasm_boundary.rs`'s snake_case export symbols on `wasm32-unknown-unknown`.

Per-function delegation, checked individually as the task asked (some
already had a separate `crate::` response function to call, several didn't):

- `get_metadata_defaults`/the six `get_default_*_font_size` getters: call
  the exact same `crate::metadata_types::MetadataDefaultsOut::default()`/
  `poentaro::ast::grouped::default_*_font_size` the old
  `#[wasm_bindgen] fn`s call directly (their own bodies were already a
  one-line direct call, not a separate response function) — duplicated
  directly in the `Guest` method body rather than extracted, since
  duplicating a one-line call is not meaningfully riskier than extracting a
  function around it.
- `list_part_declarations`/`update_part_declaration`: call the same
  `crate::part_declarations::list_part_declarations_response`/
  `update_part_declaration_source` the old `#[wasm_bindgen] fn`s call —
  already existed as separate `crate::` functions, untouched, both
  mechanisms coexist. Same treatment as group 3's `list_parts`/`list_symbols`/etc.
- `set_layout_fonts`/`shift_part_octave`/`format_score`: same
  duplicate-the-one-line-call treatment as the font-size getters — their
  bodies were already direct one/three-line calls into
  `poentaro::{set_directive_line_font_bytes,set_lyric_font_bytes,
  set_monospace_font_bytes}`/`source_edit::shift_part_octave`/
  `format_source::format_score`, no separate response function existed to
  reuse.
- `extract_source_from_svg`/`extract_source_from_pdf`: same
  duplicate-the-call treatment — bodies were already a direct one/two-line
  call into `poentaro::source_embed::{extract_embedded_source,
  extract_embedded_source_from_pdf}`, both already unconditional (not
  gated on any Cargo feature at all in the core crate), so no
  prerequisite-ungating fix was needed either.
- `compress_share_payload`/`decompress_share_payload`: **extracted**, unlike
  the two bullets above — their bodies were ~10-15 real lines each (brotli
  param setup, an early-return on compress failure, a UTF-8 validation step
  on decompress), long enough that duplicating verbatim into `component.rs`
  would create real drift risk if either body ever changed. Pulled out into
  two new `pub(crate)` functions in `lib.rs` itself
  (`compress_share_payload_bytes`/`decompress_share_payload_bytes`,
  unconditional — `brotli` is already an unconditional crate dependency, no
  feature gate needed), mirroring group 5/6's `types_export.rs`/
  `trim_window` extraction pattern exactly. `share_payload.rs`'s old
  `#[wasm_bindgen] fn`s now call these two new functions instead of
  inlining the logic themselves — same external behavior, confirmed by the
  unchanged `cargo test` count.
- `resolve_selection_range`: calls the same
  `crate::selection_range::resolve_selection_range_response` the old
  `#[wasm_bindgen] fn` already calls — that function was already
  unconditional (not gated on `wasm-bindgen-boundary` at all, unlike every
  other function in this group and unlike group 6's `sequence_entry_range`/
  `trim_window` finding), so the only real blocker here was **visibility**,
  not feature-gating: `ClickableElementId`/`ResolveSelectionRangeResponse`
  (and, only in the `wasm-bindgen-boundary`-off build, `NoteCellOut`/
  `LyricCellOut`) live in `selection_range/types.rs`, reached via a private
  `mod types;` in `selection_range/mod.rs` — invisible outside the
  `selection_range` module tree, so `component.rs` (a sibling of
  `selection_range`, not a descendant) couldn't name them at all before this
  group. Fixed with a `pub(crate) use types::{ClickableElementId,
  ResolveSelectionRangeResponse};` (unconditional) plus a second
  `#[cfg(not(feature = "wasm-bindgen-boundary"))] pub(crate) use
  types::{LyricCellOut, NoteCellOut};` (gated, since only `component.rs`
  needs those two and the boundary-on build denies `unused_imports` at full
  strictness) in `selection_range/mod.rs` — a visibility widening only, the
  module's own logic and its existing `#[wasm_bindgen] fn` are untouched.

Two new findings, neither anticipated by groups 1-6:

1. **A macro-attributed `clippy::disallowed_macros` hit, not just
   `mem_forget`/`too_many_arguments`**: `resolve_selection_range`'s
   generated export shim decodes two `clickable-element-id` variant
   arguments inline (each a 5-case discriminant match), and `wit-bindgen`'s
   own generated code for the last case of each match uses
   `debug_assert_eq!(n, 4, "invalid enum discriminant")` — caught by this
   crate's blanket `clippy::disallowed_macros` ban on the `assert_eq!`
   family (a hard rule elsewhere in the crate, meant to catch *hand-written*
   release-mode-elided assertions, not generated ABI-decoding sanity
   checks). Same macro-attribution situation as `mem_forget`/
   `too_many_arguments` — fixed the same way, added
   `clippy::disallowed_macros` to `component.rs`'s existing module-level
   `#![allow(...)]`.
2. **`MetadataDefaultsOut` is all-`Copy`-field despite the struct itself
   only deriving `Clone`, not `Copy`** (every field, including the nested
   `TextStyleDefaultsOut`, is individually `Copy`) — tripped the same
   `clippy::needless_pass_by_value` finding groups 2/3/6 already established
   for all-`Copy`-field `*_to_wit` functions, even though the struct-level
   derive itself doesn't say `Copy`. Fixed the same way: `&T` param on
   `metadata_defaults_to_wit`/`text_style_defaults_to_wit`, called as
   `metadata_defaults_to_wit(&crate::metadata_types::MetadataDefaultsOut::default())`
   at the one call site.

`cargo component build` (debug and `--release`,
`--no-default-features --features wav,mp3,pdf,midi`), `cargo build --target
wasm32-unknown-unknown --features wav,mp3,pdf,midi`, `cargo test -p
poentaro-wasm --features wav,mp3,pdf,midi` (81 passed), and `cargo clippy` on
both the boundary-on (default, `--features wav,mp3,pdf,midi`) and
boundary-off (`--no-default-features --features wav,mp3,pdf,midi --target
wasm32-unknown-unknown`) builds all pass clean — same verification bar as
groups 1-6.

**Final exhaustive cross-check (the real completion gate for "Phase 3
complete," not group 6's premature claim)**: grepped every `pub fn` directly
preceded by `#[wasm_bindgen]` across the entire crate (`wasm_boundary.rs`,
`lib_wav.rs`, `lib_mp3.rs`, `lib_pdf.rs`, `lib_midi.rs`, `lib_import.rs`,
`share_payload.rs`, `selection_range/mod.rs` — every file that has ever had
one) and diffed that list against every `fn` name inside `component.rs`'s
`Guest` impl. Result: **42 `#[wasm_bindgen] fn`s total, all 42 accounted
for** — 40 have an exact-name `Guest` method, and the remaining 2
(`render`/`render_with_highlight_range`) are the group 4-documented
deliberate renames to `render_svg`/`render_svg_with_highlight_range` (the
single-word-collision fix), confirmed present under their renamed form, not
actually missing. No stray `#[wasm_bindgen] fn` turned up anywhere outside
the 8 files already named above. **Phase 3 is now genuinely, exhaustively
complete** — any future phase that adds a new `#[wasm_bindgen] fn` should
re-run this same grep-and-diff before claiming completeness again, rather
than trusting a prose ordering list.

One real blocker turned up, confirming the task's proactive-check
instruction was warranted: **`crate::sequence_entry_range` and
`lib_wav.rs`'s `trim_window` helper — needed by
`generate_wav_for_measure_range`/`generate_mp3_for_measure_range`/
`list_note_timings_for_range` to recombine an `Option<usize>` pair back into
a `RangeInclusive`/`TrimWindow` — were reachable only from the
`wasm-bindgen` boundary**, not just their `#[wasm_bindgen] fn` wrappers,
unlike this group's underlying `crate::responses::*` functions (already
unconditionally compiled, confirmed per the task's instruction before
writing any WIT). `sequence_entry_range` in `lib.rs` was gated
`#[cfg(all(feature = "wasm-bindgen-boundary", any(feature = "wav", feature =
"midi")))]`, and `trim_window` was a `pub(crate) fn` defined directly inside
`lib_wav.rs`, whose whole module is gated
`#[cfg(all(feature = "wasm-bindgen-boundary", feature = "wav"))]` in
`lib.rs`. Fixed by dropping the `wasm-bindgen-boundary` conjunct from
`sequence_entry_range`'s `#[cfg]` (kept `any(wav, midi)`), and by moving
`trim_window` out of `lib_wav.rs` into `lib.rs` itself as a second shared,
`#[cfg(feature = "wav")]`-gated crate-level helper (mirroring
`sequence_entry_range`'s existing placement) — `lib_wav.rs`/`lib_mp3.rs` now
`use crate::trim_window;` instead of defining/importing it locally. This is
a helper *relocation*, not a change to either helper's logic, and the old
`#[wasm_bindgen] fn`s in `lib_wav.rs`/`lib_mp3.rs` were not otherwise
touched — confirmed via `cargo check` on both `--features wav,mp3,pdf,midi`
and `--no-default-features --features wav,mp3,pdf,midi --target
wasm32-unknown-unknown` before writing any `component.rs` changes, same
verification order group 5 used for its own prerequisite fix.

`NoteTimingOut` is an all-`Copy`-field struct, so its `*_to_wit` conversion
function (`note_timing_to_wit`) took `&NoteTimingOut` and mapped over
`.iter()` from the start — same `clippy::needless_pass_by_value` finding
groups 2/3 already established, applied proactively rather than
rediscovered. No new `too_many_arguments`/`too_many_lines` issues turned up
despite `generate_wav_for_measure_range`/`generate_mp3_for_measure_range`
being 12-argument WIT exports and `list_note_timings_for_range` a
9-argument one — `component.rs`'s existing module-level
`#![allow(clippy::too_many_arguments)]` (added in group 3) already covers
the macro-attributed generated export shims, and none of this group's own
hand-written functions individually crossed either threshold.

`cargo component build` (debug and `--release`,
`--no-default-features --features wav,mp3,pdf,midi`), `cargo build --target
wasm32-unknown-unknown --features wav,mp3,pdf,midi`, `cargo test -p
poentaro-wasm --features wav,mp3,pdf,midi` (82 passed), and `cargo clippy` on
both the boundary-on (default, `--features wav,mp3,pdf,midi`) and
boundary-off (`--no-default-features --features wav,mp3,pdf,midi --target
wasm32-unknown-unknown`) builds all pass clean — same verification bar as
groups 1-5.

**Phase 4 done** (`web/src/wasmInit.ts`'s instantiation model). Added, but did
NOT wire into any real call site: `ensureWasmComponentCoreModule` (fetches/
compiles the jco-transpiled component's core `.wasm` exactly once, mirroring
`ensureWasmModule`'s shared-promise/progress-reporting behavior byte-for-byte),
`instantiateWasmComponentFromModule(module)` (calls the generated
`instantiate((_) => module, {})` — no fetch, empty import object, per the
"Resolved risks" section's confirmed-sufficient approach for
`wasm32-unknown-unknown`), and `ensureWasmComponentInit` (the main-thread
`ensureWasmComponentCoreModule` -> `instantiateWasmComponentFromModule` chain,
sharing one in-flight promise the same way `ensureWasmInit` does). The old
`ensureWasmModule`/`ensureWasmInit` and every real call site across `web/`
(render/list_parts/generate_*/etc., all still importing from `'poentaro-wasm'`,
the wasm-bindgen package) are completely untouched — confirmed deliberate, not
an oversight:

- **Coexistence window resolved from the plan text itself, no need to guess**:
  Phase 3's every group explicitly landed with "both mechanisms coexist,"
  and Phase 6 ("Cutover and cleanup") is explicitly where call sites switch
  and the old `#[wasm_bindgen]` exports get deleted. Since Phase 4's own task
  says not to touch call sites elsewhere, and every call site depends on
  `ensureWasmInit()`'s `init()` populating the `'poentaro-wasm'` module those
  call sites import from, rewriting `wasmInit.ts` in place (replacing rather
  than adding to its exports) would have broken every one of those call
  sites immediately — not a real option. So this phase adds the new
  instantiation mechanism as new, additional, side-by-side exports (proven
  correct by a dedicated test, see below), ready for Phase 6 to switch call
  sites onto once every function is cut over.
- **Manual build artifact used for testing** (not wired into
  `web/package.json` — that's Phase 5), produced with, run from
  `crates/poentaro-wasm/`:
  ```sh
  cargo component build --release --no-default-features \
    --features wav,mp3,pdf,midi --target wasm32-unknown-unknown
  ```
  then, run from `web/` (jco installed ad hoc via `npx`, nothing added to
  `package.json`):
  ```sh
  npx @bytecodealliance/jco transpile \
    ../target/wasm32-unknown-unknown/release/poentaro_wasm.wasm \
    --instantiation async -o ../crates/poentaro-wasm/pkg-component
  ```
  Output landed at `crates/poentaro-wasm/pkg-component/` (`poentaro_wasm.js`,
  `poentaro_wasm.d.ts`, `poentaro_wasm.core.wasm`), gitignored — mirrors the
  existing gitignored `crates/poentaro-wasm/pkg/` convention `wasm-pack`
  already uses, and `web/src/wasmInit.ts` imports directly from this path
  (an `?url` import for the core wasm, an ordinary import for the
  `instantiate` glue + its `Root` export-map type), the same way it already
  imports `pkg/poentaro_wasm_bg.wasm?url` directly rather than through an
  alias. Confirmed via `jco --version` (1.32.1) that no separate install
  step was needed beyond `npx`'s own on-demand fetch. **Real, not toy,
  confirmation this actually runs**: a throwaway Node script instantiated
  the generated component straight from these two files (no mocking) and
  called `greet('world')`/`listParts(...)` successfully, both returning
  correct real output — the generated bindings work end-to-end, not just
  type-check.
- **`Root`'s shape confirmed flat, matching Phase 2/3's "no `interface`
  grouping" decision**: the generated `.d.ts`'s `Root` interface lists all
  25 ported functions (`greet` plus every Phase 3 group 1-6 export) as
  plain methods returning the exact response-`variant` shapes ported to
  `.wit` — no extra nesting from jco's transpile step.
- New test: `web/src/wasmComponentInit.test.ts`, mirroring
  `mainThreadWasmInit.race.test.ts`'s style (mocks the generated
  `poentaro_wasm.js` module, stubs `fetch`/`WebAssembly.compileStreaming`)
  — proves fetch-and-compile-once sharing across concurrent callers, proves
  `instantiateWasmComponentFromModule` takes no fetch path at all (the
  worker's future entry point once a module arrives via `postMessage`), and
  proves the empty-import-object assumption end-to-end against the mocked
  `instantiate` call's actual arguments.

**One real, pre-existing blocker found and fixed, not anticipated by any
prior phase's Status entry**: `crates/poentaro-wasm/src/lib.rs`'s `mod
component;` has been unconditional (not feature-gated) since Phase 3 group 1
(`ea49b26`), unlike every other wit-bindgen-vs-wasm-bindgen-exclusive file in
this crate. Once `component.rs` started implementing real `Guest` methods
(not just `greet`), this meant `wit_bindgen::generate!`/`export!`'s raw ABI
exports (kebab-case `#[export_name = "render-svg"]`-style symbols) were
compiled into the *same* `.wasm` binary as every `#[wasm_bindgen] fn`, even
in the default (`wasm-bindgen-boundary`-on) build `wasm-pack` uses.
`wasm-bindgen-cli`'s `--target web` postprocessing lists every exported
symbol from that binary — hyphenated ones included — as unquoted properties
of the generated `InitOutput` interface in `pkg/poentaro_wasm.d.ts`, which is
not valid TypeScript syntax (`readonly cabi_post_generate-split-midis: ...`
parses as subtraction, not an identifier) and made `tsc` fail with 60-200+
syntax errors depending on how many functions were ported by that point.
**Confirmed pre-existing, not something this phase's `wasmInit.ts` changes
caused**: reproduced identically on a clean worktree checked out at
`ea49b26` (Phase 3 group 1's own commit), before this phase touched
anything. Also confirmed why no prior phase caught it: every phase's own
verification bar was cargo-level only (`cargo build`/`test`/`clippy` on both
boundary-on and boundary-off) — none of them ran `wasm-pack build` +
`tsc -b` together, and this branch's own lefthook `pre-commit` hook only
runs on `ref: master` (this work has stayed on `wit-bindgen-migration`
throughout), so the hook's `web-typecheck` job (which does run
`build:wasm` + `tsc -b` together) never fired on any of Phase 1-3's commits
either. Fixed with a two-line change matching the crate's own established
pattern exactly: `#[cfg(not(feature = "wasm-bindgen-boundary"))]` on `mod
component;` in `lib.rs`, the same mutual-exclusion convention every other
wit-bindgen-only file already uses. Verified: `cargo component build`
(boundary off) unaffected, `wasm-pack build --features wav,mp3,pdf,midi`
(boundary on, the real build) now produces a clean `pkg/poentaro_wasm.d.ts`
with no hyphenated properties, `tsc -b` passes with zero errors, `cargo
test -p poentaro-wasm --features wav,mp3,pdf,midi` (81 passed) and `cargo
clippy` on both boundary-on and boundary-off builds stay clean. This narrows
clippy's coverage of `component.rs` itself to the boundary-off build only
(it's simply not compiled at all in the boundary-on build now) — a coverage
change, not a behavior change, and consistent with every other
wit-bindgen-exclusive file in the crate.

Verification run this phase: `cargo test -p poentaro-wasm --features
wav,mp3,pdf,midi` (81 passed), `cargo clippy` on both boundary-on and
boundary-off builds (clean), `cargo component build --release
--no-default-features --features wav,mp3,pdf,midi --target
wasm32-unknown-unknown` (succeeds), `pnpm run build:wasm && pnpm exec tsc -b`
in `web/` (clean, confirming the `lib.rs` fix), `pnpm run test:unit` in
`web/` (106 passed, up from 104 — the two new component-init tests), `pnpm
run lint:ast-grep` (clean). `pnpm exec biome check .` could not be run to
completion in this session's sandbox (`[warn] Linter process terminated
abnormally (possibly out of memory)`, reproduced identically on a single
untouched file with 24GB of system RAM available — an environment issue,
not a regression from this phase's changes). Also manually ran `pnpm run
dev` and confirmed via `curl` that Vite's dev server resolves and
transforms `wasmInit.ts`'s new imports (`pkg-component/poentaro_wasm.js` and
`?url` core-wasm import) into working `/@fs/...` module URLs with no
resolution errors; a full interactive browser check wasn't available in
this sandbox (no real TTY for `terminal-browser`), so this build-level
check plus the direct Node smoke test of the compiled component stood in
for it.

**Phase 5 done** (build pipeline, scoped strictly to adding the new pipeline
as separate scripts — `predev`/`prebuild`/`build:wasm` deliberately left
running `wasm-pack` unchanged). This diverges from Phase 5's own bullet-list
wording ("Replace `wasm-pack build ...` in `predev`/`prebuild`/`build:wasm`")
after a mid-task check-in: the "Rollback" section is more specific and more
authoritative than that bullet's phrasing — *"Every phase through Phase 5
can run with the old `wasm-bindgen` build fully intact and unmodified ...
nothing forces committing to the new mechanism until Phase 6 deletes the old
code"* — and Phase 4 already confirmed every real call site in `web/`
resolves exclusively through wasm-pack's `pkg/` output via the `poentaro-wasm`
alias in `vite.config.ts`/`tsconfig.app.json`, with the component/jco path
still unused by any call site. Taking the bullet literally (repointing
`predev`/`prebuild`/`build:wasm` at the component pipeline) would have broken
`pnpm run dev`/`build` today. **Flagging this tension explicitly for Phase
6**: when Phase 6 cuts call sites over, the *naming* still needs to land on
Phase 5's original intent (`build:wasm` etc. eventually meaning the
component/jco build, with the wasm-pack scripts renamed/removed at that
point) — this phase only adds the new capability, it doesn't rename anything
yet.

Concretely, added to `web/package.json`:

- New script `build:wasm-component`: `cargo component build --manifest-path
  ../crates/poentaro-wasm/Cargo.toml --profile release-wasm
  --no-default-features --features wav,mp3,pdf,midi --target
  wasm32-unknown-unknown && jco transpile
  ../target/wasm32-unknown-unknown/release-wasm/poentaro_wasm.wasm
  --instantiation async -o ../crates/poentaro-wasm/pkg-component` — the exact
  commands Phase 4 ran by hand, now permanent and scripted, not wired into
  `predev`/`prebuild`/`build` (that's Phase 6's job once call sites cut
  over).
- `@bytecodealliance/jco` added as a real `devDependency` (`^1.32.1`, the
  version Phase 4 used ad hoc via `npx`) instead of relying on `npx`'s
  on-demand fetch, per Phase 1's original plan text. `pnpm install` resolved
  and installed it cleanly (one transient registry retry, unrelated).
- `build:wasm:audio` **dropped, not ported**: confirmed genuinely dead by a
  full-repo grep (`grep -rn build:wasm:audio` across everything except
  `.claude/worktrees/` spike copies and this plan file itself) — no
  `package.json` script, CI workflow, or `lefthook.yml` job references it.
  Not carried forward into the component pipeline in any form.

**Output path/import convention: kept Phase 4's `pkg-component/` as final**,
not changed. `jco transpile`'s output filenames
(`poentaro_wasm.js`/`poentaro_wasm.d.ts`/`poentaro_wasm.core.wasm`) are identical
between Phase 4's manual scratch run and this phase's scripted one (jco
names them after the input `.wasm`'s stem regardless of source path), so
`web/src/wasmInit.ts`'s existing imports needed zero changes. `pkg-component/`
was already gitignored (`.gitignore` line 8, added in Phase 4) alongside
`pkg` — no gitignore change needed either.

**Release-profile decision: a dedicated `release-wasm` Cargo profile**,
added to the root `Cargo.toml`:

```toml
[profile.release-wasm]
inherits = "release"
debug = false
strip = true
```

`cargo component build --release` (i.e. plain `release`) inherits the
workspace's `[profile.release] debug = true` (kept for native CLI
debugging) and produced a **142.3 MiB** `.wasm` — over 20x wasm-pack's
current **6.9 MiB** `pkg/poentaro_wasm_bg.wasm`, and 8 MiB over
`scripts/check-wasm-size.py`'s size gate (not run against this artifact
today, but would fail it). Building the same crate/feature set against the
new `release-wasm` profile instead (`debug = false`, `strip = true`,
everything else inherited from `release` unchanged) produced a **5.5 MiB**
core `.wasm` — smaller than wasm-pack's current output, despite wasm-pack's
build not running `wasm-opt` either (`--no-opt`). Scoping the override to a
separate profile rather than changing `[profile.release]` itself keeps the
native `poentaro` CLI's existing debug-symbol release build (and wasm-pack's
own build, which still runs under plain `release`) completely unaffected —
`build:wasm-component`'s cargo invocation is the only thing that requests
`release-wasm`. No further stripping/optimization pass (`wasm-opt`) was
added on top; 5.5 MiB already beats the existing baseline, so no follow-up
is flagged.

**Verification performed**: `pnpm run build:wasm-component` (new script)
runs clean and produces the three expected files at
`crates/poentaro-wasm/pkg-component/` with matching filenames; `pnpm run
build:wasm` (unchanged wasm-pack script) still runs clean, confirming
`predev`/`prebuild`/`build:wasm` are untouched in behavior; `pnpm exec tsc
-b` (clean); `pnpm run test:unit` (106 passed, same count as Phase 4 — no
test changes needed since no call site or `wasmInit.ts` code changed);
`pnpm run build` (`tsc -b && vite build`, clean); `pnpm run preview` serving
and responding 200 on `/`; `pnpm run lint:ast-grep` (clean); `cargo test -p
poentaro-wasm --features wav,mp3,pdf,midi` (81 passed); `cargo clippy` on both
the boundary-on (default, `--features wav,mp3,pdf,midi`) and boundary-off
(`--no-default-features --features wav,mp3,pdf,midi --target
wasm32-unknown-unknown`) builds (both clean) — confirming the profile/script
changes have no effect on the Rust-level checks, as expected.

Four spikes done across three isolated worktrees, none merged, before this
session:

1. `.claude/worktrees/agent-a4c370b59e23802b0` (`experiments/wit-bindgen-spike/`)
   — core pipeline proof: structs/arrays/tagged-unions type-check end to
   end, and a malformed TS call site is rejected at compile time (something
   `tsify`'s current `any`-typed inputs cannot do).
2. Same worktree, follow-up — Web Worker module sharing: a
   `WebAssembly.Module` compiled once and `postMessage`'d into a worker
   instantiates correctly via jco's `instantiate(getCoreModule, ...)`, the
   exact pattern `web/src/wasmInit.ts` needs.
3. `.claude/worktrees/agent-aaad06814c61708e3` (`experiments/wit-bindgen-uint8-spike/`,
   `experiments/wit-bindgen-vite-spike/`) — binary payload (`Vec<u8>` →
   `Uint8Array`) fidelity, and real Vite dev/build/preview integration.
4. `.claude/worktrees/agent-aff1c327b381d7ca7` (`experiments/component-retrofit-spike/`)
   — retrofitting `cargo-component` onto an *existing* workspace member with
   a local path dependency and Cargo feature flags (mirroring the real
   crate), instead of a from-scratch `cargo component new` crate.

All four of the plan's original open risks are now resolved with direct
evidence (see "Resolved risks" below). One new, concrete blocker-if-ignored
was found (the workspace's `unsafe_code = "forbid"` lint) and is folded
into Phase 1 below.

**Phase 6, stages 1-3 done** (cutover — stage 3's checkpoint gate reached;
stages 4-6, the deletions, deliberately not started yet per the task's
explicit "stop and check in" instruction).

**Phase 5-vs-Rollback tension, resolved for real (as Phase 5's own Status
entry flagged this stage would have to)**: `predev`/`prebuild`/`build:wasm`
now run `build:wasm-component` (renamed conceptually to be "the" wasm
build — the script name `build:wasm-component` itself was kept as-is
rather than renamed to `build:wasm`, since nothing in the plan requires the
*script name* to change, only which pipeline `predev`/`prebuild` invoke).
`wasm-pack`-driven build scripts no longer exist anywhere in
`web/package.json`. This is a real, deliberate commitment past the
Rollback section's "every phase through Phase 5" boundary — from this
commit onward, `pnpm run dev`/`build` no longer produce or depend on
`pkg/`'s wasm-bindgen output at all, matching Phase 6's own bullet list
("Once every function is ported... remove the old... `pkg/` output
directory's old shape" — the *build script* stopped producing it in stage
1; stage 4 below still needs to delete the old `#[wasm_bindgen]`
Rust-side exports and gitignore entry, per the Rollback tension's
resolution: stage 1 committing to the new pipeline is what stage 4's
Rust-side deletion is now free to follow, not a second independent
decision point).

**Stage 1** (`5bd7d66`): `web/package.json`'s `predev`/`prebuild` now run
`pnpm run build:wasm-component`; the wasm-pack-driven `build:wasm` script
is gone (its old body — `wasm-pack build ... --features wav,mp3,pdf,midi`
— deleted, not kept as a dead alias). `web/vite.config.ts`'s dev-mode
auto-rebuild-on-Rust-file-change plugin (`wasmDevPlugin`) now shells out to
`cargo component build` + `jco transpile` (via `node_modules/.bin/jco`)
instead of `wasm-pack`, invalidating the new `pkg-component/poentaro_wasm.js`
module in Vite's module graph on rebuild instead of the old
`pkg/poentaro_wasm.js`. The `poentaro-wasm` module alias in
`vite.config.ts`'s `resolve.alias` and the `"poentaro-wasm"` path mapping in
`tsconfig.app.json` are both deleted outright (not just left unused) —
safe only because stage 2 (landed as a paired commit immediately after,
not truly a standalone gate) already moved every real import off the bare
`poentaro-wasm` specifier. `playwright.config.ts`'s stale wasm-pack-specific
comment updated to describe the new pipeline. **`wasm-pack` itself stays a
listed `devDependency` for now** — confirmed genuinely unused by `knip`
(`--include dependencies`) as of this stage, but left in place until stage
4's broader old-mechanism deletion so dependency removal and mechanism
removal land together, not split across two unrelated-looking diffs.

**Stage 2** (`5aec21d`): every real call site in `web/` that imported from
the wasm-bindgen `'poentaro-wasm'` package (function calls *and* type-only
imports — `NoteTimingOut`, `SvgDocumentOut`, `PartOut`, `SymbolOut`,
`GenerateWavResponse` and siblings, etc.) now imports from a new
`web/src/poentaroWasm.ts` instead. Cross-checked exhaustively against
`crates/poentaro-wasm`'s 42-function inventory (the same grep-and-diff
discipline Phase 3 group 7's "genuinely, exhaustively complete" cross-check
established) — every function with a real `web/` call site is cut over;
`get_measure_index_at_offset` is the one ported function with *no* real
`web/` call site today (confirmed via grep), included in `poentaroWasm.ts`
anyway for parity rather than silently dropped.

**Design decision made explicitly at this stage, not dictated verbatim by
the plan text**: `poentaroWasm.ts` is a compatibility bridge, not a
transliteration. It exposes the *exact same* function names, signatures,
and output shapes the old `pkg/poentaro_wasm.d.ts` did (snake_case fields
where tsify emitted snake_case, a flat `{ status: 'ok' | 'err', ... }`
response envelope, `SvgElementOut`'s recursive tree shape, etc.), doing the
real WIT-typed `Root` call internally and converting the result back. The
alternative the plan's Phase 6 prose could be read as implying — propagate
the WIT/jco output shape (camelCase fields, `{ tag, val }` variants, the
flattened `SvgDocument` arena) all the way out to every consumer — was
deliberately rejected after tracing the real blast radius: `web/`'s ~50
files reference these output shapes by field name far beyond the ~30 files
that `import ... from 'poentaro-wasm'` directly (component props, hook
return types, test fixtures, all flowing from `types.ts`'s
`PartInfo = PartOut`-style re-exports). Rereading the plan's own "Why"
section settled this: the type-safety gap this migration exists to close
is specifically *inputs* (`JsValue`/`any`, previously undetectable by
`tsc`) — outputs were already fully generated and type-safe via `tsify`
before this migration ever started, so reshaping them to a new naming
convention is not what "don't leave lingering manual any-shaped types"
was asking for. Every function below genuinely passes a real WIT-typed
argument to `Root` (the actual fix); only the return value's field names/
variant encoding are translated back, confined entirely to
`poentaroWasm.ts` itself. Two non-trivial conversions this bridge does for
real, not just renaming: un-flattening `SvgDocument`'s pre-order arena
(Phase 3 group 4's flattening, reversed) back into the nested
`SvgElementOut` tree `PreviewSvgRenderer.tsx` already recurses over, and
converting `ClickableElementId` between the app's hand-written
`{ kind, ... }` shape (`components/clickableElementId.ts`) and the WIT
`{ tag, val }` variant (kebab-case tags) for `resolve_selection_range`.

One real, jco-codegen-specific blocker turned up, not anticipated by any
prior phase: **jco's generated `.d.ts` declares each variant case's
payload record under the *same name* as its discriminated-union member**
(e.g. `ClickableElementIdNote` is declared once as the flat
`{ sourcePartIndex, noteId }` record and again merged with
`{ tag: 'note', val: ClickableElementIdNote }`), and TypeScript's
declaration-merging combines these into one, wider, self-referential type
that a plain `{ tag, val }` object literal — the only shape jco's runtime
actually reads — doesn't structurally satisfy. Fixed in
`convertClickableElementIdToWit` by building the value as `unknown` and
asserting it into the WIT type once at the end, rather than fighting the
merge on every variant case; documented inline as a jco quirk in case a
future function's conversion trips the same thing.

`wasmInit.ts`'s `ensureWasmModule`/`ensureWasmInit` now do what were
separately-named `ensureWasmComponentCoreModule`/`ensureWasmComponentInit`
during the Phase 3-5 coexistence window (fetch/compile/instantiate the
component, then `setWasmRoot()` into `poentaroWasm.ts`) — the old
wasm-bindgen-specific `ensureWasmModule`/`ensureWasmInit` bodies are gone,
not kept alongside. `poentaro.worker.ts` mirrors this on its own thread:
`ensureInit()` now calls `instantiateWasmComponentFromModule` +
`poentaroWasm.setWasmRoot` instead of the wasm-bindgen `init()`, and reports
every export as available unconditionally (`audioAvailable`/
`pdfAvailable`/`midiAvailable`/`mp3Available` all hardcoded `true` in the
worker's `ready` message) since the component build always ships every
feature — matching reality, not a regression, since `web/package.json`
only ever built with the full feature set anyway (see the plan's
"Complicating factor" note). `worker/optionalWasmExports.ts` is deleted
outright: its whole reason to exist (defensively checking `'x' in module`
in case a stale cached wasm-bindgen build lacked a newer export) no longer
applies — every function is unconditionally present in the component
`Root`.

Three wasm-init tests updated to mock the jco-generated component glue
module (`../../crates/poentaro-wasm/pkg-component/poentaro_wasm.js`, matched
by resolved path, not textual specifier — confirmed each test's relative
`vi.mock` path resolves to the same file `wasmInit.ts`/`poentaro.worker.ts`
import) instead of the old bare `'poentaro-wasm'` specifier:
`mainThreadWasmInit.race.test.ts`, `worker/poentaro.worker.raceInit.test.ts`,
and `wasmComponentInit.test.ts` (the Phase 4 test exercising
`ensureWasmComponentCoreModule`/`ensureWasmComponentInit` directly, now
renamed in-place to exercise the folded-in `ensureWasmModule`/
`instantiateWasmComponentFromModule` instead of being deleted, since its
own coverage — fetch-and-compile-once sharing, instantiate-from-an-
already-compiled-module-with-no-fetch — is still real coverage of
`wasmInit.ts`'s current code, not dead).

**Stage 3 — full app verified working end-to-end against the component/jco
path only, before any deletion (the plan's explicit gate)**:

- `pnpm exec tsc -b`: clean, zero errors.
- `pnpm run test:unit`: 106/106 passed (same count as Phase 4/5 — no test
  was added or removed net, only the three wasm-init tests' mock targets
  changed).
- `pnpm run build` (`tsc -b && vite build`): clean; `dist/` contains
  `poentaro_wasm.core-*.wasm` (6.6 MiB) and no `poentaro_wasm_bg.wasm` — the
  wasm-bindgen artifact is gone from the shipped build, not just unused
  source.
- `pnpm run check` (knip): flags `wasm-pack` as an unused devDependency
  (expected — see stage 1's note on deferring its removal to stage 4) and
  two pre-existing, unrelated findings (`src/utils/partSource.ts` unused,
  `@github/hotkey` unused dependency) not touched by this phase.
- **Full Playwright e2e suite** (`pnpm run test:e2e`, real browser,
  against `vite` serving the freshly-built `pkg-component/` output — no
  wasm-bindgen build present on disk during this run at all): **260
  scenarios, 240 passed clean, 20 passed only after Playwright's own
  configured retry (2 retries/test), 0 failed** after retries exhausted.
  One scenario (`sequence-jump-preview-reveal-target.feature.spec.js`,
  a `toBeInViewport()` scroll-position assertion) initially looked like a
  real, non-flaky failure (failed all 3 attempts in the full-suite run) —
  investigated directly per the task's "stop if a shape mismatch turns up"
  instruction before treating it as acceptable flake: rerun in isolation
  3 more times, passed twice and flaked once (passed on its own retry),
  confirming genuine test-level flakiness (scroll/viewport timing in this
  sandbox, matching `playwright.config.ts`'s own documented
  cache-write-failure sandbox caveats) rather than a wasm-shape regression
  — nothing about scroll timing touches any type this migration changed.
  Every other retry-passed scenario was an already-known-flaky class
  (export-progress-toast timing, GitHub-storage-mock timing) unrelated to
  this migration.
- **Manual visual verification** (the `run` skill found no project-level
  run skill, so it fell back to the browser-driven pattern): launched
  `vite` directly (bypassing Playwright's own webServer wiring) and drove
  a headless Chromium against it with a throwaway script (`chromium-cli`
  unavailable in this sandbox), screenshotting the app after it loaded a
  real `.poentaro` file. The screenshot shows a correctly rendered "Pitches"
  score (3 measures, section labels, editor/preview in sync) — real,
  visual proof that `render_svg` (and every function the initial file-load
  path depends on) works end-to-end through the component path with the
  screenshot taken from a build where the wasm-bindgen `pkg/` mechanism
  was never invoked this session. The soundfont/PDF-fonts progress banner
  showed its two large-asset fetches as errored in this particular
  headless run — reproduced the exact `net::ERR_CACHE_WRITE_FAILURE`
  `playwright.config.ts` already documents as a known sandbox limitation
  worked around by Playwright's own browser launch flags (which this
  throwaway script didn't pass, unlike the real e2e suite, where the same
  soundfont/font-dependent scenarios — instrument preview, PDF/audio
  export — passed cleanly) — not a wasm-component regression.

**Phase 6, stage 4 done** (`a620f3c`) — the wasm-bindgen/tsify boundary is
deleted outright: `wasm_boundary.rs`/`lib_wav.rs`/`lib_mp3.rs`/
`lib_pdf.rs`/`lib_midi.rs`/`lib_import.rs`/`share_payload.rs` (every
`#[wasm_bindgen] fn` in the crate) are gone, along with the
`wasm-bindgen-boundary` Cargo feature and the optional `wasm-bindgen`/
`serde-wasm-bindgen`/`tsify` dependencies — this crate now has exactly one
build configuration and one boundary mechanism (`component.rs`). `mod
component;` in `lib.rs` and `selection_range/mod.rs`'s WIT-facing
re-exports are unconditional now (the Phase 4 `#[cfg(not(...))]`
coexistence gates are gone). Deliberate deviation from the original
"make Tsify derives permanent/unconditional" instruction: the underlying
Rust struct/enum types are kept exactly as they were, but their
`#[cfg_attr(feature = "wasm-bindgen-boundary", derive(Tsify))]`
attributes were removed rather than made unconditional, since making them
unconditional would silently reintroduce Phase 1's own confirmed blocker
(linking wasm-bindgen's rlib, which `into_wasm_abi` requires, leaves
`__wbindgen_placeholder__` stubs that only `wasm-bindgen-cli`'s `--target
web` postprocessing strips — a step `cargo component build` never runs).
Verified: `cargo component build` still succeeds with the derives
stripped, and nothing in Tsify's JS-codegen role is missed since
`wit-bindgen`/`jco` now generates every function's real TS types from
`wit/world.wit`. One genuine finding: `GroupNoteSelectionResponse::Err`/
`GroupLyricSelectionResponse::Err` are flagged dead code for the first
time (never actually constructed — grouping over already-typed spans/cells
can't fail) — kept (`#[allow(dead_code)]`, documented inline) rather than
reshaping the WIT-facing response shape, since `component.rs`/
`wit/world.wit`/`poentaroWasm.ts` already treat them as a real API shape for
parity with every other spans-based response.

**Phase 6, stage 5 confirmed a no-op**: none of `experiments/
wit-bindgen-spike/`, `experiments/wit-bindgen-uint8-spike/`,
`experiments/wit-bindgen-vite-spike/`, `experiments/
component-retrofit-spike/`, or the three spike worktree branches
(`worktree-agent-a4c370b59e23802b0`/`aaad06814c61708e3`/`aff1c327b381d7ca7`)
exist anywhere in this repo — confirmed via `find`/`git branch --list
'*spike*'`/`git log --all --oneline | grep spike`, all empty. They were
done in throwaway worktrees before this session (see the four numbered
spikes above) and never merged into any branch this repo still holds —
nothing to delete.

**Phase 6, stage 6 done** (`ae5d80c`) — `ARCHITECTURE.md` gained a "WASM
boundary mechanism" section (ahead of the existing "WASM exports" section)
documenting `component.rs`/`wit/world.wit`, the `cargo-component`/`jco`
build, and `poentaroWasm.ts`'s compatibility-bridge role, per `CLAUDE.md`'s
rule. The exported functions' own signatures/shapes (in "WASM exports")
are unchanged from the reader's perspective — that's the bridge's whole
point — so that section's content stays accurate; only the mechanism
producing it changed. Also fixed two stale file-path references left over
from an earlier, unrelated `selection_range` directory split
(`selection_range_types.rs` → `selection_range/types.rs`,
`selection_range.rs` → `selection_range/mod.rs`). `syntax.md` needed no
change, confirming the plan's own prediction — no user-facing `.poentaro`
syntax was touched anywhere in this migration.

**Post-deletion re-verification** (this session, after stages 4-6 landed):
`cargo test -p poentaro-wasm --features wav,mp3,pdf,midi` — 82 passed;
`cargo clippy -p poentaro-wasm --features wav,mp3,pdf,midi --all-targets` —
clean (this now runs `--all-targets` successfully, unlike every earlier
phase's boundary-off variant, since there's no cfg-off build configuration
left to trip the previously-documented pre-existing `--all-targets`
failure); `pnpm exec tsc -b` — clean; `pnpm run test:unit` — 106/106
passed. The dedicated boundary-on/boundary-off double verification every
prior phase ran is retired as of this stage — there is now only one
configuration to verify, matching the plan's Goal/Non-goals ("delete
[wasm-bindgen/tsify] rather than keep two boundary mechanisms alive").

**Phase 6, and the whole migration, is complete.** Every function in
`crates/poentaro-wasm` is exposed exclusively through `wit-bindgen`/
`component.rs`/`wit/world.wit`; `web/` builds and ships exclusively
through `cargo-component`/`jco` (`pnpm run build`'s `dist/` contains only
the jco-generated core `.wasm`, no wasm-bindgen artifact); the full
Playwright e2e suite passed (260 scenarios, 240 clean + 20 known-flake
retries, 0 real failures, recorded under stage 3 above) against this
component-only build before stage 4 deleted the old mechanism. The
`web/src/poentaroWasm.ts` compatibility bridge (reshaping the WIT-typed
`Root`'s outputs back to the old snake_case/flat-`{status,...}` convention
real call sites already expect) is a deliberate, permanent architectural
piece, not a migration leftover — it stays because the plan's actual
type-safety goal was untyped *inputs* (the `JsValue`/`any` problem
"Why" describes), not renaming every output field; nothing in stage 4's
cleanup changed that.

## Goal

Replace `wasm-bindgen` + `tsify` as the Rust↔TS boundary mechanism for the
whole `poentaro-wasm` crate with `wit-bindgen` (guest) + `cargo-component`
(build) + `jco` (JS/TS glue), so every function's full signature — inputs
*and* outputs — is generated from one `.wit` schema instead of only outputs
being generated and inputs being hand-tallied `JsValue`/`any`.

## Non-goals

- No backward compatibility to preserve — single user, no external
  consumers of `poentaro-wasm`'s JS API, no need for a long parallel-support
  period.
- Not attempting to also adopt Component Model features irrelevant to a
  browser target (resources, async, WASI) — `wasm32-unknown-unknown` stays
  the build target throughout, confirmed to avoid the WASI shim entirely
  (single small core module, empty import object) versus the default
  `wasm32-wasip1` target's multi-module, WASI-shimmed output.
- Not preserving `wasm-bindgen`/`tsify` afterward as a fallback — once
  cutover is complete and tests pass, delete them rather than keep two
  boundary mechanisms alive.

## Why

`tsify`'s `into_wasm_abi` makes Rust→TS output types fully generated and
type-safe already, but `Vec<CustomStruct>` isn't supported as a real
`wasm-bindgen` parameter type (only `JsCast` types are), so every `*In`
type (`NoteCellIn`, `LyricCellIn`, `MeasureRangeIn`, the instrument-info
structs) is passed as `JsValue`, decoded with
`serde_wasm_bindgen::from_value(...).unwrap_or_default()`, and surfaces as
`any` in the `.d.ts` — a rename or shape change on the Rust side is not
caught by the TS compiler, only discovered at runtime (or silently
defaulted to an empty result). `wit-bindgen`/`jco` closes this because a
single `.wit` file is the schema for both sides of every function, inputs
included — confirmed with a real, deliberately-malformed TS call site
producing genuine `tsc` errors (`TS2353`, `TS2741`, `TS2322`) in spike (1).

## Resolved risks (evidence, not inference)

### Binary payloads: `Vec<u8>` → `Uint8Array` — confirmed clean

`list<u8>` in `.wit` maps directly to a real `Uint8Array` in the generated
`.d.ts` (not `number[]`, no manual wrap needed). Verified: byte values
`[0,1,2,253,254,255]` round-tripped exactly (unsigned, no i8
sign-corruption), coexists fine as one arm of a `variant` alongside a
plain-record-list arm (mirroring `GenerateWavResponse`'s `Ok{wav}`/
`Err{diagnostics}` shape), and a ~1MB buffer round-tripped correctly in
0.66ms — no copy-heavy bottleneck to worry about for PDF/MIDI/WAV/MP3
export sizes.

### Vite integration — confirmed working, dev and production build

Both jco's default relative-fetch (`new URL('./core.wasm', import.meta.url)`)
and an explicit `?url` import + `getCoreModule` override work under `vite
dev` **and** a real `vite build` + `vite preview` (production static
assets, content-hashed, correctly rewritten by Vite's build-time analysis)
— not just the dev server. No `optimizeDeps.exclude`/`assetsInclude`
config needed; Vite already treats `.wasm` as a static asset and recognizes
jco's `new URL(..., import.meta.url)` pattern natively. One harmless
build-time warning (`node:fs/promises` externalized — dead code in jco's
generated Node-fallback branch, not a real issue).

**Recommended approach for Phase 4/5**: the explicit `?url` + `getCoreModule`
override, not the default relative fetch — it requires the smallest change
to `wasmInit.ts`, since `wasmInit.ts` already does its own manual
`fetchWithProgress`/`WebAssembly.compileStreaming`/worker-sharing and gains
nothing from jco's default fetch path. Concretely: transpile with `jco
transpile --instantiation async`, keep the existing `?url` import (pointed
at the new generated core `.wasm` file instead of `poentaro_wasm_bg.wasm`),
and on the worker side call `instantiate((_) => receivedModule, {})` with
no fetch at all, since the module already arrived via `postMessage`.

### Web Worker module sharing — confirmed working

A `WebAssembly.Module` compiled once and `postMessage`'d into a
`worker_threads` worker (a faithful stand-in for a browser `Worker` — same
structured-clone algorithm, including the `WebAssembly.Module` transfer
clause) instantiates correctly via `instantiate((_) => receivedModule, {})`
with zero re-fetch/re-compile, for both the `wasm32-unknown-unknown` build
(empty import object, trivial) and the `wasm32-wasip1` build (WASI import
object must be constructed locally on whichever side calls `instantiate`,
since it wraps live JS closures that can't cross `postMessage` — moot once
`wasm32-unknown-unknown` is the target, per the non-goals above).

### `cargo-component` retrofit onto an existing workspace member — confirmed viable, with one real prerequisite fix

`cargo component new`'s scaffolding is **not required** — an existing
workspace member becomes `cargo-component`-buildable by hand-adding
`[package.metadata.component]` (WIT package name) and
`[package.metadata.component.target] path = "wit"` to its `Cargo.toml`,
plus a `wit/` directory, plus `wit-bindgen` as a normal dependency. This
worked on the first `cargo component build` invocation once two
prerequisite issues (found via direct testing, not inference) were
resolved — see Phase 1 below, since one of them is a real, required code
change to `crates/poentaro-wasm/Cargo.toml` before any porting can start:

1. **Blocking**: the root workspace sets
   `[workspace.lints.rust] unsafe_code = "forbid"`, inherited by
   `crates/poentaro-wasm/Cargo.toml`'s `[lints] workspace = true`.
   `wit-bindgen`'s `export!`/`generate!` macros expand to
   `unsafe extern "C"` functions, which `forbid(unsafe_code)` rejects as a
   hard compile error (not downgradable once inherited via `forbid`).
   Cargo also refuses to mix `workspace = true` with any local `[lints.*]`
   table in the same manifest, so the fix is all-or-nothing: drop
   `[lints] workspace = true` from `crates/poentaro-wasm/Cargo.toml`, copy the
   workspace's clippy lints into it verbatim, and locally relax
   `unsafe_code`, `unreachable_pub`, and `unused_qualifications` (the three
   that conflict) while keeping the rest as-is.
2. **Non-blocking but adjacent**: `poentaro` itself has a
   pre-existing, unrelated latent bug — `src/audio_source.rs` unconditionally
   imports symbols that are only used inside `#[cfg(feature = "wav")]`/
   `#[cfg(feature = "midi")]` code, so building it (or anything depending on
   it) with less than the full feature set trips `-D unused-imports`.
   Confirmed against the real, unmodified crate:
   `cargo check -p poentaro-wasm --no-default-features` fails identically
   today, independent of this migration. It never surfaces in practice
   because `web/package.json` always builds with
   `--features wav,mp3,pdf,midi`. Not something this plan needs to fix, but
   Phase 1's first sanity build must use that full feature set — a
   bare/no-feature sanity build will fail for a reason that has nothing to
   do with `wit-bindgen`.

**Root workspace side effects**: confirmed via `git diff` before/after —
`crates/poentaro-wasm/` untouched; root `Cargo.toml` gains exactly one line
(the new member's path, nothing else — no resolver/profile changes); root
`Cargo.lock` gains the new member's own dependencies plus one real but
harmless side effect worth knowing about: `wit-bindgen`'s transitive
dependencies (`wit-component`/`wit-parser`, needing `indexmap` with its
`serde` feature) widen the *single, workspace-shared* resolved `indexmap`
entry's feature set, which is also used by `zip` (already a dependency for
PDF/MIDI/MP3 export) — `zip`'s own behavior is unaffected, but it's a real
example of how adding `wit-bindgen`'s dependency tree to any workspace
member can silently touch a shared dependency's feature set in the
lockfile without touching that dependency's own manifest entry.

### Cargo feature flags vs. a `wit-bindgen` `Guest` trait — confirmed: implementations can be feature-gated, trait methods cannot

- `cargo component build --features <name>` is exactly plain
  `cargo build --features <name>` — no separate mechanism.
- A single `.wit` world needs no per-feature variants; Rust-side
  `#[cfg(feature = "...")]` can freely change what a function's *body*
  does.
- **Directly tested and confirmed foreclosed**: `#[cfg]`-gating a `Guest`
  trait *method itself* and building without that feature fails with
  `error[E0046]: not all trait items implemented` — the generated trait
  unconditionally requires every WIT-declared export to be implemented,
  regardless of Cargo features. This isn't a new problem for this plan
  (Phase 2 already commits to declaring every function unconditionally in
  the `.wit` world, matching the fact that the real app always ships every
  feature on today), but it's now a confirmed hard constraint, not a
  currently-irrelevant theoretical one — worth stating plainly in case the
  build matrix ever needs to change in the future.

### `Guest` trait ergonomics at small-but-real scale — confirmed clean, with one deliberate design choice to make in Phase 2

A 4-function `Guest` trait (plain function, `list<record>` param, a
hand-rolled `variant` return, one feature-gated implementation) generated a
flat, ordinary-looking Rust trait with no nesting or verbosity friction —
kebab-case WIT names became `snake_case` Rust automatically, and
`String`/`Vec<T>`/records/variants all mapped unambiguously. This flatness
is specifically because the spike declared every function as a direct
`export` of the WIT `world`, not grouped under a named `interface` — Phase
2 should decide deliberately whether `poentaro-wasm`'s real `.wit` groups its
10+ functions under `interface` blocks (more conventional for a
larger API, but was not spike-tested and may reintroduce a
`bindings::exports::<pkg>::<world>::...` nesting depth) or keeps everything
as flat world-level exports as the spike did. Recommendation: start flat;
only introduce `interface` grouping if the flat `.wit` world file itself
becomes hard to navigate at full size.

## Complicating factor found while scoping this plan

`web/package.json` always builds `poentaro-wasm` with every optional feature
on (`--features wav,mp3,pdf,midi` in `predev`/`prebuild`/`build:wasm`,
`wav,mp3` only in the unused `build:wasm:audio` script) — so in practice
there is exactly one feature combination that ships, and (per the
confirmed E0046 finding above) a `wit-bindgen` `Guest` trait couldn't
support per-feature-combo exports even if that changed. The `.wit` world
can simply declare every function unconditionally. `build:wasm:audio`
being unused should be confirmed dead (via `knip` or a repo search for
callers) and either ported or dropped as part of Phase 5.

## Design / phased plan

### Phase 1 — Fix prerequisites, then bootstrap tooling and crate skeleton

- **New, required first step**: edit `crates/poentaro-wasm/Cargo.toml` to
  drop `[lints] workspace = true`, replace it with the workspace's clippy
  lints copied verbatim plus `unsafe_code`, `unreachable_pub`, and
  `unused_qualifications` relaxed locally (see "Resolved risks" above for
  why — this is a hard compile-blocker for any `wit-bindgen` code in this
  crate, confirmed, not hypothetical). This is a small, self-contained
  change that can land on its own before anything else in this plan.
- Add `cargo-component` to the dev toolchain (documented install step, not
  a `Cargo.toml` dependency — it's a `cargo` subcommand).
- Add `@bytecodealliance/jco` as a local devDependency in `web/`
  (`package.json`), not global.
- Add `[package.metadata.component]` (WIT package name, e.g.
  `poentaro:wasm`) and `[package.metadata.component.target] path = "wit"` to
  `crates/poentaro-wasm/Cargo.toml`, and a `wit/` directory — confirmed this
  is all that's needed on an existing workspace member; no
  `cargo component new`-only scaffolding is required.
- Sanity-check with a single trivial exported function
  (`greet(name: string) -> string` or similar) before porting any real
  logic — **build it with the full feature set the real app always uses**
  (`--features wav,mp3,pdf,midi`), not a bare build, since
  `poentaro` itself fails a bare/partial-feature build for reasons
  unrelated to this migration (see "Resolved risks" above).
- Gitignore `crates/poentaro-wasm/src/bindings.rs` — `cargo component build`
  writes this scratch file to disk on every build (IDE/rust-analyzer
  visibility only; the macro still expands in-place at compile time, this
  file isn't itself referenced by `lib.rs`).

### Phase 2 — Port the type schema to `.wit`

Translate every type in `crates/poentaro-wasm/src/{types,note_selection_types,
lyric_selection_types,metadata_types,selection_range_types,svg_types,
types_export,responses*}.rs` into `.wit` records/variants/enums. Every
shape actually in use maps cleanly, confirmed by spike:

- Plain structs → `record`.
- `Option<T>` fields (e.g. `NoteSpanOut.start`/`.end`,
  `PartDeclarationOut.follow_target`) → `option<T>`.
- The `#[serde(tag = "status")] enum { Ok {...}, Err {...} }` pattern
  (`RenderResponse`, `ListPartsResponse`, `GroupNoteSelectionResponse`, …)
  → `variant` with `ok`/`err` cases.
- `Vec<u8>` binary payloads (`GenerateWavResponse.wav`,
  `GeneratePdfResponse.pdf`, MIDI/MP3/zip bytes) → `list<u8>` — **confirmed**
  maps directly to `Uint8Array`, byte-exact, no performance concern at the
  ~1MB scale tested.
- Enums with no data (`DiagnosticSeverity`, `SymbolKindOut`,
  `PartDeclarationModeOut`, `OccurrenceRoleOut`) → WIT `enum`.

Keep field naming in `kebab-case` in `.wit` — confirmed `jco` auto-converts
to `camelCase` in the generated `.d.ts`, matching every existing
`#[serde(rename_all = "camelCase")]` struct with no extra configuration.

**Decide up front**: declare all 10+ functions as flat `export`s of the
`.wit` world (spike-confirmed clean, no nesting friction) rather than
grouping under `interface` blocks, unless the flat world file itself proves
unwieldy once every real function is added.

Land this phase as Rust structs generated by `wit-bindgen`'s guest macro,
without yet rewriting the `#[wasm_bindgen]` functions — get the types
compiling and matching shape-for-shape against the current `*In`/`*Out`
types before touching call sites.

### Phase 3 — Port functions one at a time, low-risk first

Suggested order, each landed as its own commit with the old
`#[wasm_bindgen]` function and new `wit-bindgen` implementation coexisting
(two separate compiled artifacts loaded side by side in `web/`, cut over
per call site) until the whole crate is ported. Every WIT-declared function
must be implemented unconditionally regardless of Cargo feature (confirmed
E0046 otherwise) — not an issue given every feature always ships together
today, but note it as a closed door, not a currently-irrelevant one:

1. `group_note_selection` / `group_lyric_selection` — already prototyped
   almost exactly as they exist in `lib.rs`; lowest risk, proves the real
   crate compiles under `wit-bindgen` before anything else depends on it.
2. `list_note_spans` / `list_lyric_spans` / `list_measure_spans` — no
   `JsValue` input, `Option<Vec<String>>` param (`enabled_tracks`) is the
   only new shape to validate against WIT's `option<list<string>>`.
3. `list_parts` / `list_symbols` / `rename_symbol` / `measure_at_offset` —
   similar shape family, still no binary payloads.
4. `render` / `render_with_highlight_range` — the most heavily used
   function and the most complex input (raw source string, instrument
   info list, highlight range), migrate once the simpler functions have
   proven the pattern end-to-end in the real app, not just the spike.
5. `generate_midi` / `generate_wav` / `generate_pdf` / `generate_mp3` and
   their `split` variants — now de-risked (binary payload mapping
   confirmed clean), but still last in line simply because they're the
   lowest call frequency and least likely to block the rest of the app if
   something unexpected turns up.

### Phase 4 — Rewrite `wasmInit.ts`'s instantiation model

Confirmed viable by spike, with a specific recommended approach: replace
the `wasm-bindgen`-specific `init()`/module-sharing calls with jco's
`instantiate(getCoreModule, imports, instantiateCore)` (transpiled with
`jco transpile --instantiation async`), using the **explicit `?url` +
override** approach rather than jco's default relative fetch (see
"Resolved risks" above for why — it's the smaller diff):

- Main thread: keep the existing `fetchWithProgress` download-progress
  plumbing and `WebAssembly.compileStreaming`/`compile` exactly as today,
  just against the new generated core `.wasm`'s `?url` import instead of
  `poentaro_wasm_bg.wasm`. `postMessage` the compiled `WebAssembly.Module` to
  the render worker as today.
- Worker: call `instantiate((_) => receivedModule, {})` — no fetch, no
  `?url` import needed on the worker side at all, since the module already
  arrived via `postMessage`.
- This phase should be behavior-invisible to the rest of the app if done
  right — same download-progress UX, same single-compile-shared-everywhere
  behavior.

### Phase 5 — Build pipeline

- Replace `wasm-pack build ...` in `web/package.json`'s
  `build:wasm`/`predev`/`prebuild` scripts with `cargo component build
  --release --target wasm32-unknown-unknown` + `jco transpile
  --instantiation async`.
- Confirm `build:wasm:audio` is genuinely dead (unused script) before
  deciding whether to port it or drop it.
- Confirmed by spike: Vite's `?url` import pattern and static-asset
  handling work against jco's output with zero config changes needed
  (no `optimizeDeps.exclude`, no `assetsInclude`), verified under both
  `vite dev` and a real `vite build` + `vite preview`. Update the specific
  import path/filename to match whatever `jco transpile` names the core
  `.wasm` output.
- Note: `cargo component build --release` inherits the workspace's
  `debug = true` release-profile setting, producing much larger output
  than a stripped release build would — worth a deliberate profile
  decision in this phase (not new to this migration, but worth comparing
  against current `wasm-pack --no-opt` output size deliberately rather
  than being surprised by it).
- Update `crates/poentaro-wasm/Cargo.toml`: drop `tsify`,
  `serde-wasm-bindgen`, `wasm-bindgen`; add `wit-bindgen` (version pinned
  to what the spikes used, 0.44.0, or whatever's current at implementation
  time).

### Phase 6 — Cutover and cleanup

- Once every function is ported and the app works end-to-end against the
  new module (manually verified via the `run` skill, plus the existing
  Playwright e2e suite), remove the old `#[wasm_bindgen]` functions and the
  `pkg/` output directory's old shape.
- Delete `experiments/wit-bindgen-spike/`, `experiments/wit-bindgen-uint8-spike/`,
  `experiments/wit-bindgen-vite-spike/`, `experiments/component-retrofit-spike/`,
  and the three spike worktrees/branches
  (`worktree-agent-a4c370b59e23802b0`, `worktree-agent-aaad06814c61708e3`,
  `worktree-agent-aff1c327b381d7ca7`).
- Update `ARCHITECTURE.md` per `CLAUDE.md`'s rule (entry-function
  signatures and module paths are changing) and `syntax.md` only if any
  user-facing `.poentaro` syntax is incidentally touched (unlikely — this is
  a boundary-mechanism change, not a syntax change).

## Remaining open items (genuinely unresolved, smaller in scope than before)

- **Full-scale bundle size / build time** versus the current `wasm-pack`
  pipeline has only been sanity-checked on trivial/toy crates (tens of
  bytes to a few KB), never on a crate with `poentaro-wasm`'s real size and
  dependency graph (fonts, `brotli`, the full `poentaro` surface).
  Worth measuring once Phase 3 has a few real functions ported, not
  assumed from the spikes.
- **Flat world-exports vs. `interface`-grouped WIT** at full 10+ function
  scale is a real design decision Phase 2 should make deliberately (see
  above) — the spike only validated the flat approach at 4 functions.
- ~~**`web/package.json`'s `build:wasm:audio` script status** (used/dead)
  should be confirmed before Phase 5 rather than assumed.~~ Resolved in
  Phase 5: confirmed dead, dropped.

## Rollback

Every phase through Phase 5 can run with the old `wasm-bindgen` build
fully intact and unmodified (new component built as a separate artifact,
cut over per call site) — nothing forces committing to the new mechanism
until Phase 6 deletes the old code. The one exception is Phase 1's lint
fix to `crates/poentaro-wasm/Cargo.toml` (dropping `[lints] workspace =
true`), which is required before any `wit-bindgen` code compiles in this
crate at all — but that change alone is inert (no behavior change) until
`wit-bindgen`-based code actually exists to trip the relaxed lints, so it
carries no real rollback risk on its own. If a later phase turns out to be
a blocker, the app keeps working on `wasm-bindgen` throughout and this plan
can be abandoned or paused with zero cleanup debt beyond the
`experiments/` folders, the spike worktrees, and any partially-ported
functions living alongside the originals.
