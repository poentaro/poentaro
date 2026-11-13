//! A score's SVG and MIDI exports for `POST /agent`.
#![allow(clippy::disallowed_macros)]

use live_share_worker::agent::{export_midi, export_svg};

const VALID: &str = "parts [\"Melody\" [M]: notes],\nscore \"\"\"\n    [M] 1 2 3 4\n    \"\"\",\n";

/// A score whose second line doesn't close its list. The renderer would
/// recover from it and draw a blank page.
const UNREADABLE: &str = "title \"Hymn\",\nparts [oops\n";

#[test]
fn a_score_exports_as_svg_pages() {
    let pages = export_svg(VALID).unwrap();
    assert_eq!(pages.len(), 1);
    assert!(pages[0].starts_with("<svg"), "{}", &pages[0][..80]);
}

#[test]
fn a_score_exports_as_a_standard_midi_file() {
    let midi = export_midi(VALID).unwrap();
    assert_eq!(&midi[..4], b"MThd");
}

#[test]
fn a_score_with_errors_is_not_exported_and_says_where() {
    let svg = export_svg(UNREADABLE).unwrap_err();
    let midi = export_midi(UNREADABLE).unwrap_err();
    assert_eq!(svg, midi);
    assert!(svg.iter().any(|error| error.line == 2), "{svg:?}");
}
