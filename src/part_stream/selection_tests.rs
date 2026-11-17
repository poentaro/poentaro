use crate::error::Span;
use crate::part_stream::{
    apply_part_stream, part_stream_ranges_to_source, source_ranges_to_part_stream,
};

const MELODY_AND_BASS: &str = r#"parts ["Melody" [M]: notes, "Bass" [B]: notes],

score """
    [M]   1  2 3 4   // intro
    [B] 5 5 5 5

    [M] 5 6 7 1 // second
    [B] 3 3 3 3
    """,
"#;

/// Where the first `needle` after the first `after` sits in `text`.
fn span_of(text: &str, after: &str, needle: &str) -> Span {
    let from = text.find(after).unwrap();
    let start = from + text[from..].find(needle).unwrap();
    Span::new(start, start + needle.len())
}

fn texts<'a>(text: &'a str, spans: &[Span]) -> Vec<&'a str> {
    spans
        .iter()
        .map(|span| &text[span.start..span.end])
        .collect()
}

fn draft(source: &str, abbreviation: &str, stream: &str) -> String {
    apply_part_stream(source, abbreviation, stream)
        .source
        .unwrap()
}

#[test]
fn a_stream_range_maps_onto_the_parts_line_whatever_its_spacing() {
    let stream = "1 2 3 4\n5 6 7 1";
    let ranges =
        part_stream_ranges_to_source(MELODY_AND_BASS, "M", stream, &[span_of(stream, "", "2 3")]);
    assert_eq!(texts(MELODY_AND_BASS, &ranges), vec!["2 3"]);
    assert_eq!(ranges, vec![span_of(MELODY_AND_BASS, "[M]", "2 3")]);
}

#[test]
fn a_range_across_a_barline_becomes_one_range_per_measure() {
    let stream = "1 2 3 4\n5 6 7 1";
    let ranges = part_stream_ranges_to_source(
        MELODY_AND_BASS,
        "M",
        stream,
        &[Span::new(
            stream.find('4').unwrap(),
            stream.find('6').unwrap() + 1,
        )],
    );
    assert_eq!(texts(MELODY_AND_BASS, &ranges), vec!["4", "5 6"]);
}

#[test]
fn ranges_map_into_the_draft_once_the_stream_is_edited() {
    let stream = "0 1 2 3\n4 5 6 7\n1";
    let draft = draft(MELODY_AND_BASS, "M", stream);
    let ranges = part_stream_ranges_to_source(
        MELODY_AND_BASS,
        "M",
        stream,
        &[
            span_of(stream, "4", "5 6"),
            Span::new(stream.len(), stream.len()),
        ],
    );
    assert_eq!(texts(&draft, &ranges), vec!["5 6", ""]);
    assert_eq!(ranges[0], span_of(&draft, "[M] 4", "5 6"));
    let after_the_one = draft.find("[M] 1 0 0 0").unwrap() + "[M] 1".len();
    assert_eq!(ranges[1], Span::new(after_the_one, after_the_one));
}

#[test]
fn spelled_out_padding_has_nothing_to_select_in_the_source() {
    let source = r#"parts ["Melody" [M]: notes],

score """
    [M] 1 2
    """,
"#;
    let stream = "1 2 - -";
    let padding = part_stream_ranges_to_source(source, "M", stream, &[span_of(stream, "", "- -")]);
    assert_eq!(padding, Vec::<Span>::new());
    let note_and_padding =
        part_stream_ranges_to_source(source, "M", stream, &[span_of(stream, "", "2 -")]);
    assert_eq!(texts(source, &note_and_padding), vec!["2"]);
}

#[test]
fn a_caret_in_a_measure_the_part_does_not_write_lands_on_its_measure_group() {
    let source = r#"parts ["Melody" [M]: notes, "Bass" [B]: notes],

score """
    [M] 1 2 3 4
    [B] 5 5 5 5

    [B] 3 3 3 3
    """,
"#;
    let stream = "1 2 3 4\n0 0 0 0";
    let caret = stream.len() - 1;
    let ranges = part_stream_ranges_to_source(source, "M", stream, &[Span::new(caret, caret)]);
    let group_start = source.find("[B] 3").unwrap();
    assert_eq!(ranges, vec![Span::new(group_start, group_start)]);
}

#[test]
fn a_stream_with_errors_maps_nothing() {
    let stream = "1 2 3 4 (";
    let ranges = part_stream_ranges_to_source(MELODY_AND_BASS, "M", stream, &[Span::new(0, 3)]);
    assert_eq!(ranges, Vec::<Span>::new());
    let ranges = source_ranges_to_part_stream(
        MELODY_AND_BASS,
        "M",
        stream,
        &[span_of(MELODY_AND_BASS, "[M]", "1")],
    );
    assert_eq!(ranges, Vec::<Span>::new());
}

#[test]
fn a_source_range_on_the_parts_line_maps_onto_its_chunk() {
    let stream = "1 2 3 4\n5 6 7 1";
    let ranges = source_ranges_to_part_stream(
        MELODY_AND_BASS,
        "M",
        stream,
        &[
            span_of(MELODY_AND_BASS, "[M]", "2 3"),
            span_of(MELODY_AND_BASS, "[M] 5", "7 1"),
        ],
    );
    assert_eq!(texts(stream, &ranges), vec!["2 3", "7 1"]);
}

#[test]
fn another_parts_note_jumps_to_its_measures_chunk() {
    let stream = "1 2 3 4\n5 6 7 1";
    let ranges = source_ranges_to_part_stream(
        MELODY_AND_BASS,
        "M",
        stream,
        &[span_of(MELODY_AND_BASS, "[B] 3", "3")],
    );
    let second_measure = stream.find('5').unwrap();
    assert_eq!(ranges, vec![Span::new(second_measure, second_measure)]);
}

#[test]
fn another_parts_note_adds_no_caret_where_the_part_is_selected() {
    let stream = "1 2 3 4\n5 6 7 1";
    let ranges = source_ranges_to_part_stream(
        MELODY_AND_BASS,
        "M",
        stream,
        &[
            span_of(MELODY_AND_BASS, "[M]", "1  2 3 4"),
            span_of(MELODY_AND_BASS, "[B]", "5 5 5 5"),
            span_of(MELODY_AND_BASS, "[B] 3", "3 3"),
        ],
    );
    let second_measure = stream.find('5').unwrap();
    assert_eq!(
        ranges,
        vec![Span::new(0, 7), Span::new(second_measure, second_measure)]
    );
}

#[test]
fn lyric_syllables_map_both_ways() {
    let source = r#"parts ["Melody" [M]: notes, "Words" [W]: lyrics [M]],

score """
    [M] 1 2 3 4
    [W] twin  kle twin kle

    [M] 5 5 6 6
    [W] lit tle star how
    """,
"#;
    let stream = "twin kle twin kle\nlit tle star how";
    let to_source =
        part_stream_ranges_to_source(source, "W", stream, &[span_of(stream, "lit", "tle star")]);
    assert_eq!(texts(source, &to_source), vec!["tle star"]);
    let to_stream =
        source_ranges_to_part_stream(source, "W", stream, &[span_of(source, "[W]", "kle twin")]);
    assert_eq!(texts(stream, &to_stream), vec!["kle twin"]);
}

#[test]
fn offsets_past_the_stream_or_inside_a_character_are_clamped() {
    let source = r#"parts ["Melody" [M]: notes, "Words" [W]: lyrics [M]],

score """
    [M] 1 2 3 4
    [W] 你 好 世 界
    """,
"#;
    let stream = "你 好 世 界";
    let ranges = part_stream_ranges_to_source(source, "W", stream, &[Span::new(1, 99)]);
    assert_eq!(texts(source, &ranges), vec!["你 好 世 界"]);
}
