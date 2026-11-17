use super::*;
use crate::ast::parsed::PartKind;

#[test]
fn explicit_lyrics_keep_lyric_row() {
    let input = r#"metadata { title "t", author "a" },

parts [
    "Soprano": notes,
    "Soprano lyrics" [Sopranov1]: lyrics [Soprano],
    "Alto": notes,
    "Alto lyrics" [Altov1]: lyrics [Alto],
],

score """
    time=4/4 key=C4 bpm=120
    [Soprano] 1 2 3 4
    [Sopranov1] do re mi fa
    [Alto] 5 6 7 1
    [Altov1] la la la la
    """,
"#;
    let score = compile(input, "test.poentaro", &[]).unwrap();
    let parts = &score.measures[0].parts;
    assert_eq!(parts.len(), 4, "each lyric part is a part of its own");
    let lyric_targets: Vec<&str> = parts
        .iter()
        .filter_map(|part| part.slice().lyrics.as_ref())
        .map(|lyrics| lyrics.target_name.as_str())
        .collect();
    assert_eq!(lyric_targets, ["Soprano", "Alto"]);
    for part in parts {
        let slice = part.slice();
        assert_eq!(
            slice.lyrics.is_some(),
            matches!(slice.kind, PartKind::Lyrics { .. })
        );
    }
}

/// Each lyric part of one target is its own part, in declaration order.
#[test]
fn multiple_lyric_parts_of_one_target_are_separate_parts() {
    let input = r#"metadata { title "t", author "a" },

parts [
    "Melody": notes,
    "Melody verse 1" [Melodyv1]: lyrics [Melody],
    "Melody verse 2" [Melodyv2]: lyrics [Melody],
],

score """
    time=4/4 key=C4 bpm=120
    [Melody] 1 2 3 4
    [Melodyv1] do re mi fa
    [Melodyv2] one two three four
    """,
"#;
    let score = compile(input, "test.poentaro", &[]).unwrap();
    let parts = &score.measures[0].parts;
    assert_eq!(parts.len(), 3, "notes part plus two lyric parts");
    let verse_texts = |part: usize| -> Vec<String> {
        parts[part]
            .slice()
            .lyrics
            .as_ref()
            .unwrap()
            .syllables
            .iter()
            .map(|s| s.text.clone())
            .collect()
    };
    assert_eq!(parts[1].name().map(String::as_str), Some("Melodyv1"));
    assert_eq!(parts[2].name().map(String::as_str), Some("Melodyv2"));
    assert_eq!(verse_texts(1), vec!["do", "re", "mi", "fa"]);
    assert_eq!(verse_texts(2), vec!["one", "two", "three", "four"]);
}

/// A lyric part's syllables reach its own `PartSlice`, next to a target slice
/// that carries no lyrics.
#[test]
fn lyric_part_syllables_reach_its_own_part_slice() {
    let input = r#"metadata { title "t", author "a" },

parts ["Melody": notes, "Melody lyrics" [Melodyv1]: lyrics [Melody]],

score """
    time=4/4 key=C4 bpm=120
    [Melody] 1 2 3 4
    [Melodyv1] la la la la
    """,
"#;
    let score = compile(input, "test.poentaro", &[]).unwrap();
    assert!(matches!(
        score.measures[0].parts[0].slice().kind,
        PartKind::Notes
    ));
    assert!(score.measures[0].parts[0].slice().lyrics.is_none());
    let slice = score.measures[0].parts[1].slice();
    assert!(matches!(slice.kind, PartKind::Lyrics { .. }));
    let verse_texts: Vec<String> = slice
        .lyrics
        .as_ref()
        .unwrap()
        .syllables
        .iter()
        .map(|s| s.text.clone())
        .collect();
    assert_eq!(verse_texts, vec!["la", "la", "la", "la"]);
}

/// A part whose verse count changes between two consecutive measures no
/// longer forces a new system: systems pack purely by count, and a system's
/// rows become the union of every verse used across its measures (see the
/// "union-of-parts system packing" feature — union_row_order/pad_chunk_to_union
/// in `grid_layout::layout_systems`). The measure missing a verse gets it
/// padded in as a blank verse row rather than triggering an early break.
#[test]
fn verse_count_change_does_not_force_new_system() {
    let input = r#"metadata { title "t", author "a" },

parts [
    "Melody": notes,
    "Melody verse 1" [Melodyv1]: lyrics [Melody],
    "Melody verse 2" [Melodyv2]: lyrics [Melody],
],

score """
    time=4/4 key=C4 bpm=120
    [Melody] 1 2 3 4
    [Melodyv1] do re mi fa

    [Melody] 5 6 7 1
    [Melodyv1] one two three four
    [Melodyv2] uno dos tres cuatro
    """,
"#;
    let score = compile(input, "test.poentaro", &[]).unwrap();
    let compile_result = compiler::compile(&score);
    let compile_result = consolidator::consolidate(compile_result);
    let config = render_config::RenderConfig::from_metadata(&score.metadata);
    let systems =
        grid_layout::layout::pack_into_systems(&compile_result.blocks, &config, f32::INFINITY);
    assert_eq!(
        systems.len(),
        1,
        "a verse-count change alone should not force a new system; both measures fit in one"
    );
}

#[test]
fn lyric_parts_are_not_listed_as_editable_part_declarations() {
    let input = r#"parts ["Melody" [M]: notes, "Verse 1" [v1]: lyrics [M]],

score """
    [M] 1 2 3 4
    [v1] do re mi fa
    """,
"#;
    let declarations = list_part_declarations_from_source(input, "test.poentaro", &[]).unwrap();
    assert_eq!(declarations.len(), 1);
    assert_eq!(declarations[0].abbreviation, "M");
    let parts = list_parts_from_source(input, "test.poentaro", &[]).unwrap();
    assert_eq!(parts.len(), 2);
    assert_eq!(parts[1].abbreviation, "v1");
}
