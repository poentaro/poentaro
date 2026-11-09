use super::*;

/// Recursively collects the visual leading-glyph x of every `Text` element
/// carrying `variant`, in document order. The leading-glyph x is where the
/// first character actually starts painting, accounting for `anchor`
/// (`Middle`/`End` shift the paint origin away from the box x), so it's the
/// coordinate readers actually compare when eyeballing column alignment.
fn collect_leading_glyph_x(
    elements: &[renderer::new_types::SvgElement],
    variant: renderer::new_types::SvgVariant,
    out: &mut Vec<(f32, String)>,
) {
    use compositor::types::TextAnchor;
    use renderer::new_types::SvgKind;

    for elem in elements {
        if elem.variant == Some(variant) {
            if let SvgKind::Text {
                content,
                font_size,
                anchor,
                ..
            } = &elem.kind
            {
                let text_width = font_metrics::monospace_text_width(content, *font_size);
                let leading_x = match anchor {
                    TextAnchor::Start => elem.x,
                    TextAnchor::Middle => elem.x - text_width / 2.0,
                    TextAnchor::End => elem.x - text_width,
                };
                out.push((leading_x, content.clone()));
            }
        }
        if let SvgKind::Group { children, .. } = &elem.kind {
            collect_leading_glyph_x(children, variant, out);
        }
    }
}

/// Font size of the first `Text` element carrying `variant`, searched
/// recursively in document order.
fn first_text_font_size(
    elements: &[renderer::new_types::SvgElement],
    variant: renderer::new_types::SvgVariant,
) -> Option<f32> {
    use renderer::new_types::SvgKind;

    elements.iter().find_map(|elem| match &elem.kind {
        SvgKind::Text { font_size, .. } if elem.variant == Some(variant) => Some(*font_size),
        SvgKind::Group { children, .. } => first_text_font_size(children, variant),
        _ => None,
    })
}

/// A chord symbol wider than a single digit (e.g. `2m`) must still have its
/// root digit line up with the note it annotates — a reader visually
/// compares `[c] 1 2m 3m 4` against `[n] 1 2 3 4` column by column, not box
/// center against box center.
#[test]
fn wide_chord_symbols_align_their_root_digit_with_the_note() {
    let input = r#"parts ["c": chords, "n": notes],

score """
    [c] 1 2m 3m 4
    [n] 1 2 3 4
    """,
"#;

    let output = render_documents_from_source_with_visibility(
        input,
        "test.poentaro",
        &ResolvedPartVisibility::default(),
        &[],
        NoteLabels::Degrees,
    )
    .unwrap();

    let mut chords = Vec::new();
    let mut notes = Vec::new();
    collect_leading_glyph_x(
        &output.documents[0].elements,
        renderer::new_types::SvgVariant::ChordSymbol,
        &mut chords,
    );
    collect_leading_glyph_x(
        &output.documents[0].elements,
        renderer::new_types::SvgVariant::NoteHead,
        &mut notes,
    );

    assert_eq!(chords.len(), 4, "expected 4 chord symbols, got {chords:?}");
    assert_eq!(notes.len(), 4, "expected 4 note heads, got {notes:?}");

    for ((chord_x, chord_text), (note_x, note_text)) in chords.iter().zip(notes.iter()) {
        assert!(
            (chord_x - note_x).abs() < 0.01,
            "chord {chord_text:?} (leading x={chord_x}) should align with note \
             {note_text:?} (leading x={note_x})",
        );
    }
}

/// A chord's root accidental draws *left* of its degree (`♯1m`, like a note
/// head's `♯1`), hanging into the column's accidental lead, so the degree
/// itself — not the accidental — lines up with the note on the same beat.
#[test]
fn chord_root_accidentals_lead_the_degree_which_aligns_with_the_note() {
    let input = r#"parts ["c": chords, "n": notes],

score """
    [c] 1#m 2 3b7 4/5b
    [n] 1 2 3 4
    """,
"#;

    let output = render_documents_from_source_with_visibility(
        input,
        "test.poentaro",
        &ResolvedPartVisibility::default(),
        &[],
        NoteLabels::Degrees,
    )
    .unwrap();

    let mut chords = Vec::new();
    let mut notes = Vec::new();
    collect_leading_glyph_x(
        &output.documents[0].elements,
        renderer::new_types::SvgVariant::ChordSymbol,
        &mut chords,
    );
    collect_leading_glyph_x(
        &output.documents[0].elements,
        renderer::new_types::SvgVariant::NoteHead,
        &mut notes,
    );

    let chord_texts: Vec<&str> = chords.iter().map(|(_, text)| text.as_str()).collect();
    assert_eq!(
        chord_texts,
        vec!["\u{266F}1m", "2", "\u{266D}3\u{2077}", "4/\u{266D}5"]
    );
    assert_eq!(notes.len(), 4, "expected 4 note heads, got {notes:?}");

    let chord_font_size = first_text_font_size(
        &output.documents[0].elements,
        renderer::new_types::SvgVariant::ChordSymbol,
    )
    .expect("expected a chord symbol");
    for ((chord_x, chord_text), (note_x, note_text)) in chords.iter().zip(notes.iter()) {
        let degree_x = chord_x
            + font_metrics::monospace_text_width(
                font_metrics::chord_leading_accidental(chord_text),
                chord_font_size,
            );
        assert!(
            (degree_x - note_x).abs() < 0.01,
            "chord {chord_text:?} (degree x={degree_x}) should align with note \
             {note_text:?} (leading x={note_x})",
        );
    }
}

#[test]
fn notes_and_chords_parts_align_on_the_same_beat() {
    let input = r#"parts ["a": notes, "b": chords],

score """
    [a] 1
    [b] 1
    """,
"#;

    let output = render_documents_from_source_with_visibility(
        input,
        "test.poentaro",
        &ResolvedPartVisibility::default(),
        &[],
        NoteLabels::Degrees,
    )
    .unwrap();

    let mut chords = Vec::new();
    let mut notes = Vec::new();
    collect_leading_glyph_x(
        &output.documents[0].elements,
        renderer::new_types::SvgVariant::ChordSymbol,
        &mut chords,
    );
    collect_leading_glyph_x(
        &output.documents[0].elements,
        renderer::new_types::SvgVariant::NoteHead,
        &mut notes,
    );

    let (chord_x, _) = chords.first().expect("expected a chord symbol");
    let (note_x, _) = notes.first().expect("expected a note head");

    assert!(
        (chord_x - note_x).abs() < 0.01,
        "the note head and the chord symbol occupying the same beat should share \
         the same visual x position (note at x={note_x}, chord at x={chord_x})"
    );
}
