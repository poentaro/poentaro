//! Cucumber harness for [`format_score`]'s part-order sorting (see
//! `tests/features/format_score_part_order.feature`): a measure group's
//! surviving `[Key]` lines are reordered to match `parts` declaration
//! order, and a positional (unprefixed) lyrics line must travel with its
//! nearest preceding `[Key]` line's block rather than being treated as
//! independent, unattached content.
//!
//! Clippy's `allow-*-in-tests` (clippy.toml) only recognizes `#[test]`-
//! attributed functions as test code; cucumber's `#[given]`/`#[when]`/
//! `#[then]` step functions don't qualify even though this whole file only
//! ever runs under `cargo test`. Mirrors `tests/cucumber.rs`'s
//! `#![allow(clippy::disallowed_macros)]` for the same reason.
#![allow(
    clippy::unwrap_used,
    clippy::panic,
    clippy::indexing_slicing,
    clippy::disallowed_macros,
    clippy::needless_pass_by_value
)]

use cucumber::gherkin::Step;
use cucumber::{given, then, when, World as _};
use poentaro::format_anyfix::format_anyfix as format_score;

#[derive(Debug, Default, cucumber::World)]
struct FormatWorld {
    source: String,
    formatted: String,
}

#[given(expr = "the score source:")]
fn given_score_source(world: &mut FormatWorld, step: &Step) {
    world.source = step.docstring().cloned().unwrap_or_default();
}

#[when(expr = "it is formatted")]
fn when_formatted(world: &mut FormatWorld) {
    world.formatted = format_score(&world.source);
}

#[then(expr = "the formatted source is:")]
fn then_formatted_source(world: &mut FormatWorld, step: &Step) {
    let expected = step.docstring().cloned().unwrap_or_default();
    assert_eq!(
        world.formatted.trim_end_matches('\n'),
        expected.trim_matches('\n'),
        "formatted source mismatch"
    );
}

#[tokio::main]
async fn main() {
    FormatWorld::run("tests/features/format_score_part_order.feature").await;
}
