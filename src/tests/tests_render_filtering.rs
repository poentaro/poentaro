use super::*;

#[test]
fn list_parts_from_source_returns_declarations() {
    let input = r#"metadata { title "t", author "a" },

parts [
    "main": chords,
    "Alto 1 & Tenor" [A1 & T]: notes,
    "Alto 1 & Tenor lyrics" [A1 & Tv1]: lyrics [A1 & T],
],

score """
    time=4/4 key=C4 bpm=120
    [main] 1m
    [A1&T] 1 2 3 4
    [A1&Tv1] a b c d
    """,
"#;
    let parts = list_parts_from_source(input, "test.poentaro", &[]).unwrap();
    assert_eq!(parts.len(), 3);
    assert_eq!(parts[0].abbreviation, "main");
    assert_eq!(parts[0].display_name, "main");
    assert_eq!(parts[1].abbreviation, "A1&T");
    assert_eq!(parts[1].display_name, "Alto 1 & Tenor");
    assert_eq!(parts[2].abbreviation, "A1&Tv1");
    assert_eq!(parts[2].display_name, "Alto 1 & Tenor lyrics");
}

#[test]
fn hidden_lyrics_do_not_reserve_lyric_row_space() {
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
    [Sopranov1] sop sop sop sop
    [Alto] 5 6 7 1
    [Altov1] alt alt alt alt
    """,
"#;
    let all = render_svgs_from_source(input, "test.poentaro", &[])
        .unwrap()
        .svgs;
    let alto_lyrics_hidden = render_svgs_from_source_filtered(
        input,
        "test.poentaro",
        Some(&["Soprano".into(), "Sopranov1".into(), "Alto".into()]),
        &[],
    )
    .unwrap()
    .svgs;
    assert_ne!(
        all[0].len(),
        alto_lyrics_hidden[0].len(),
        "hiding one part's lyrics should change rendered SVG size"
    );
}

#[test]
fn render_svgs_from_source_filtered_can_hide_lyrics_per_part() {
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
    [Sopranov1] sop sop sop sop
    [Alto] 5 6 7 1
    [Altov1] alt alt alt alt
    """,
"#;
    let all = render_svgs_from_source(input, "test.poentaro", &[])
        .unwrap()
        .svgs;
    let alto_lyrics_hidden = render_svgs_from_source_filtered(
        input,
        "test.poentaro",
        Some(&["Soprano".into(), "Sopranov1".into(), "Alto".into()]),
        &[],
    )
    .unwrap()
    .svgs;
    assert!(all[0].contains("sop"));
    assert!(all[0].contains("alt"));
    assert!(alto_lyrics_hidden[0].contains("sop"));
    assert!(!alto_lyrics_hidden[0].contains("alt"));
}

#[test]
fn render_svgs_from_source_filtered_can_hide_parts() {
    let input = r#"metadata { title "t", author "a" },

parts ["Soprano": notes, "Alto": notes],

score """
    time=4/4 key=C4 bpm=120
    [Soprano] 1 2 3 4
    [Alto] 5 6 7 1
    """,
"#;
    let all = render_svgs_from_source(input, "test.poentaro", &[])
        .unwrap()
        .svgs;
    let soprano_only =
        render_svgs_from_source_filtered(input, "test.poentaro", Some(&["Soprano".into()]), &[])
            .unwrap()
            .svgs;
    assert_ne!(all[0], soprano_only[0]);
}

#[test]
fn render_svgs_from_source_filtered_hides_legend_entry_for_filtered_out_parts() {
    let input = r#"metadata { title "t", author "a" },

parts ["Soprano" [S]: notes, "Alto" [A]: notes],

score """
    time=4/4 key=C4 bpm=120
    [S] 1 2 3 4
    [A] 5 6 7 1
    """,
"#;
    let all = render_svgs_from_source(input, "test.poentaro", &[])
        .unwrap()
        .svgs;
    assert!(all[0].contains(">S ") || all[0].contains("S \u{2014}"));
    assert!(all[0].contains(">A ") || all[0].contains("A \u{2014}"));

    let soprano_only =
        render_svgs_from_source_filtered(input, "test.poentaro", Some(&["S".into()]), &[])
            .unwrap()
            .svgs;
    assert!(soprano_only[0].contains("S \u{2014} Soprano"));
    assert!(!soprano_only[0].contains("A \u{2014} Alto"));
}

#[test]
fn render_documents_from_source_with_visibility_hides_legend_entry_for_filtered_out_parts() {
    let input = r#"metadata { title "t", author "a" },

parts ["Soprano" [S]: notes, "Alto" [A]: notes],

score """
    time=4/4 key=C4 bpm=120
    [S] 1 2 3 4
    [A] 5 6 7 1
    """,
"#;
    let soprano_only = render_documents_from_source_with_visibility(
        input,
        "test.poentaro",
        &ResolvedPartVisibility {
            rendered_tracks: Some(vec!["S".into()]),
            ..Default::default()
        },
        &[],
        NoteLabels::Degrees,
    )
    .unwrap()
    .documents;
    let svgs = serializer::serialize(&soprano_only, None);
    assert!(svgs[0].contains("S \u{2014} Soprano"));
    assert!(!svgs[0].contains("A \u{2014} Alto"));
}

#[test]
fn split_track_names_falls_back_to_part_declarations() {
    let input = r#"metadata { title "t", author "a" },

parts ["Melody": notes, "Melody lyrics" [Melodyv1]: lyrics [Melody]],

score """
    time=4/4 key=C4 bpm=120
    [Melody] 1 2 3 4
    [Melodyv1] a b c d
    """,
"#;
    let score = compile(input, "test.poentaro", &[]).unwrap();
    let names =
        split_track_names(input, "test.poentaro", &score, SplitSelection::default()).unwrap();
    assert_eq!(names, vec!["Melody"]);
}

#[test]
fn split_pdf_filename_sanitizes_track_name() {
    assert_eq!(
        split_pdf_filename("song", "Alto 1 & Tenor"),
        "song - Alto 1 & Tenor.pdf"
    );
    assert_eq!(
        split_pdf_filename("song", "bad/name"),
        "song - bad-name.pdf"
    );
}

#[test]
fn apply_track_filter_keeps_only_enabled_lyric_parts() {
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
    [Altov1] alt alt alt alt
    """,
"#;
    let mut score = compile(input, "test.poentaro", &[]).unwrap();
    apply_track_filter(
        &mut score,
        Some(&["Soprano".into(), "Alto".into(), "Altov1".into()]),
    );
    let names: Vec<&str> = score.measures[0]
        .parts
        .iter()
        .filter_map(|part| part.name().map(String::as_str))
        .collect();
    assert_eq!(
        names,
        ["Soprano", "Alto", "Altov1"],
        "a lyric part that is not enabled should be dropped, like any other part"
    );
    assert!(score.measures[0].parts[2].slice().lyrics.is_some());
}

#[test]
fn apply_track_filter_keeps_a_lyric_part_whose_target_is_hidden() {
    let input = r#"parts ["Melody" [M]: notes, "Verse 1" [v1]: lyrics [M]],

score """
    [M] 1 2 3 4
    [v1] do re mi fa
    """,
"#;
    let mut score = compile(input, "test.poentaro", &[]).unwrap();
    apply_track_filter(&mut score, Some(&["v1".into()]));
    assert_eq!(score.measures[0].parts.len(), 1);
    let lyrics = score.measures[0].parts[0].slice().lyrics.as_ref().unwrap();
    assert_eq!(lyrics.target_name, "M");
    assert_eq!(lyrics.syllables.len(), 4);
    assert_eq!(
        lyrics.target_events.len(),
        4,
        "the lyric part still carries its target's timing"
    );
}

#[test]
fn key_prefix_only_b_omits_rest_filled_a() {
    let input = r#"metadata { title "Untitled", author "author" },

parts ["A": notes, "B": notes],

score """
    [B] 1 2 3 4
    """,
"#;
    let svgs = render_svgs_from_source(input, "test.poentaro", &[])
        .unwrap()
        .svgs;
    let svg = &svgs[0];

    assert!(
        !svg.contains(">A<"),
        "part A (rest-filled) should be omitted"
    );
    assert!(svg.contains(">B<"), "part B row label should appear");
    assert!(svg.contains(">1<"), "part B note 1 should appear");
    assert!(svg.contains(">2<"), "part B note 2 should appear");
    assert!(svg.contains(">3<"), "part B note 3 should appear");
    assert!(svg.contains(">4<"), "part B note 4 should appear");
    assert!(!svg.contains(">0<"), "part A rests should not appear");
}

#[test]
fn rendering_under_a_preset_draws_its_header_text() {
    let input = r#"metadata { title "Song", subtitle "Base subtitle" },

parts ["Soprano" [S]: notes],

score """
    time=4/4 key=C4 bpm=120
    [S] 1 2 3 4
    """,

preset "Choir" { subtitle "Choir subtitle" },
"#;
    let render = |preset_name: Option<&str>| {
        let documents = render_documents_from_source_with_visibility(
            input,
            "test.poentaro",
            &ResolvedPartVisibility {
                preset_name: preset_name.map(str::to_string),
                ..Default::default()
            },
            &[],
            NoteLabels::Degrees,
        )
        .unwrap()
        .documents;
        serializer::serialize(&documents, None).concat()
    };
    assert!(render(Some("Choir")).contains("Choir subtitle"));
    assert!(render(None).contains("Base subtitle"));
    assert!(render(Some("Gone")).contains("Base subtitle"));
}

#[test]
fn chosen_preset_from_source_reads_the_named_preset() {
    let input = r#"parts ["Soprano" [S]: notes, "Alto" [A]: notes],

score """
    [S] 1
    [A] 2
    """,

preset "Choir" { visible parts [A], soloed parts [S] },
"#;
    assert_eq!(
        chosen_preset_from_source(input, "test.poentaro", "Choir").unwrap(),
        Some(ChosenPreset {
            name: "Choir".to_string(),
            visible_parts: Some(vec!["A".to_string()]),
            soloed_parts: vec!["S".to_string()],
        })
    );
    assert_eq!(
        chosen_preset_from_source(input, "test.poentaro", "Gone").unwrap(),
        None
    );
}
