//! `wit-bindgen` component boundary — the crate's one and only JS/TS
//! boundary mechanism (`PLAN-wit-bindgen-migration.md`; the old
//! `wasm-bindgen`/`tsify` boundary this module coexisted with through the
//! migration's Phase 1-6 was deleted in Phase 6 stage 4). Every function's
//! per-comment "Phase N" history below records how it was ported, kept as
//! useful context rather than scrubbed after the fact.
//!
//! `wit-bindgen`'s generated glue itself calls `mem::forget` on a `Drop`
//! type (`Box<[u8]>`) internally, tripping the workspace's `mem_forget`
//! clippy lint (a deliberate hard rule elsewhere in the crate). The lint
//! fires on code attributed to the macro's expansion, not the invocation
//! site itself, so it needs a module-level (not attribute-on-invocation)
//! allow — scoped to just this module rather than relaxed crate-wide.
//!
//! Phase 3, group 3: `rename_symbol`'s WIT-level ABI lowering (each
//! `string` param lowers to a `ptr, len` pair; `source`/`old_name`/
//! `new_name`/`raw_instruments` are `string`-shaped, `kind` is a plain
//! discriminant) produces a 9-argument generated export shim, tripping this
//! crate's `too_many_arguments` threshold (6) on code attributed to the
//! `wit_bindgen::generate!` macro's expansion, not any function this file
//! actually wrote — same macro-attribution situation as `mem_forget` above.
//!
//! Phase 3, group 7: `resolve_selection_range`'s generated export shim
//! decodes two `clickable-element-id` variant arguments inline, each
//! producing a `match arg { 0 => .., 1 => .., .., n => { debug_assert_eq!(n,
//! 4, ..); .. } }` for the final case — `std::assert_eq`-family macros are
//! banned crate-wide (`clippy::disallowed_macros`), but this one is
//! `wit_bindgen::generate!`'s own generated code, attributed back to the
//! macro invocation itself rather than any function this file wrote, same
//! macro-attribution situation as `mem_forget`/`too_many_arguments` below.
//!
//! This module is split into submodules purely to keep every file under the
//! crate's 400-line cap; `wit_bindgen::generate!` above puts every WIT type
//! into this one `component` module's namespace, so each submodule needs
//! `use super::*;` to reach them — explicit named imports of the dozens of
//! generated types each file uses would be far less readable than the
//! `wildcard_imports` lint (workspace-`deny`) exists to prevent, so it's
//! relaxed here, same module-level-not-per-site reasoning as the lints
//! above. Rust also forbids splitting one trait impl across files (E0119),
//! so `impl Guest for Component` stays a single block in `guest_impl.rs`;
//! each method's real body lives in a same-named free function in the
//! relevant `guest_*` submodule instead (see those files' own
//! `needless_pass_by_value` allow: they're kept at the exact by-value
//! parameter types the `Guest` trait requires, `guest_impl.rs` just forwards
//! to them, but clippy doesn't know that once they're plain free functions).
#![allow(
    clippy::mem_forget,
    clippy::too_many_arguments,
    clippy::disallowed_macros,
    clippy::wildcard_imports
)]

wit_bindgen::generate!({
    world: "poentaro-wasm",
});

struct Component;

mod guest_file_selection;
mod guest_file_store;
mod guest_folder_create;
mod guest_folder_share;
mod guest_generate_and_timings;
mod guest_impl;
mod guest_legacy_file_store;
mod guest_metadata_and_misc;
mod guest_outbox;
mod guest_part_stream_selection;
mod guest_presets;
mod guest_range_edits;
mod guest_selection_and_render;
mod guest_selection_range;
mod guest_staff_export;
mod guest_staff_measures;

mod diagnostics_conversion;
mod highlight_conversion;
mod history_conversion;
mod metadata_and_selection_conversion;
mod metadata_fields_conversion;
mod outbox_conversion;
mod part_visibility_conversion;
mod parts_symbols_conversion;
mod render_and_generate_conversion;
mod selection_span_conversion;
mod svg_conversion;
mod svg_flatten;

use diagnostics_conversion::*;
use guest_file_selection::*;
use guest_file_store::*;
use guest_folder_create::*;
use guest_folder_share::*;
use guest_generate_and_timings::*;
use guest_legacy_file_store::*;
use guest_metadata_and_misc::*;
use guest_outbox::*;
use guest_part_stream_selection::*;
use guest_presets::*;
use guest_range_edits::*;
use guest_selection_and_render::*;
use guest_selection_range::*;
use guest_staff_export::*;
use guest_staff_measures::*;
use highlight_conversion::*;
use metadata_and_selection_conversion::*;
use metadata_fields_conversion::*;
use part_visibility_conversion::*;
use parts_symbols_conversion::*;
use render_and_generate_conversion::*;
use selection_span_conversion::*;
use svg_conversion::*;
use svg_flatten::*;

export!(Component);

#[cfg(test)]
mod tests_outbox;
#[cfg(test)]
mod tests_outbox_history;
#[cfg(test)]
mod tests_part_stream;
