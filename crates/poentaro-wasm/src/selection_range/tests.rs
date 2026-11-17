//! Split by the same topic boundaries as the production code under
//! `selection_range/` (see that directory's `mod.rs`) — each file here
//! covers the matching topic module's tests. `test_helpers` holds the
//! fixtures/builder functions shared across them.

mod test_helpers;

mod label_fill_tests;
mod lyric_label_tests;
mod lyric_lyric_tests;
mod measure_tests;
mod mixed_tests;
mod note_lyric_tests;
mod note_note_tests;
mod part_label_tests;
mod selection_contiguity_property_tests;
mod system_tests;
