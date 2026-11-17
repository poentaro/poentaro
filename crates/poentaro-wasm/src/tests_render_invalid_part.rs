use crate::responses::render_response;
use crate::types::RenderResponse;
use poentaro::{NoteLabels, ResolvedPartVisibility};

#[test]
fn a_document_with_an_invalid_part_still_renders_and_reports_only_that_error() {
    let source = "parts [\"Melody\" [M]: nots],\n\nscore \"\"\"\n    [M] 1\n    \"\"\",\n";
    let response = render_response(
        source,
        &ResolvedPartVisibility::default(),
        &[],
        NoteLabels::Degrees,
        poentaro::grid_layout::PageLayout::A4,
    );
    let RenderResponse::Ok {
        documents,
        diagnostics,
        ..
    } = response
    else {
        panic!("expected the document to render");
    };
    assert!(!documents.is_empty());
    assert_eq!(diagnostics.len(), 1, "{diagnostics:?}");
    assert_eq!(
        &source[diagnostics[0].span.start..diagnostics[0].span.end],
        "nots"
    );
}
