use super::tests::parse_parts;
use crate::ast::parsed::Soundfont;

#[test]
fn source_level_follow_explicit_volume_and_octave() {
    use super::SourcePartMode;

    let content = r#"parts ["Melody" [M]: notes, "Chords" [C]: follow [M] { volume: 60 %, octave: + 1 }]
"#;
    let mut errors = Vec::new();
    let decls = crate::parser::anyfix::source_part_declarations(content, &mut errors, &[]);
    assert!(errors.is_empty());
    assert_eq!(decls[1].mode, SourcePartMode::Follow);
    assert_eq!(decls[1].volume, Some(60));
    assert_eq!(decls[1].octave_offset, Some(1));
}

#[test]
fn source_level_line_numbers_in_multi_section_file() {
    use crate::list_part_declarations_from_source;

    let source = r#"metadata { title "t" },

parts [
    "Melody" [M] : notes,
    "Chords" [C] : follow [M],
],

score "",
"#;

    let declarations = list_part_declarations_from_source(source, "test.poentaro", &[]).unwrap();
    assert_eq!(declarations.len(), 2);
    assert_eq!(declarations[0].line_number, 4);
    assert_eq!(declarations[1].line_number, 5);
}

#[test]
fn instrument_sets_the_sound_when_no_soundfont_is_quoted() {
    let content = r#"parts ["Violin" [V]: notes { instrument: violin }, "Zheng" [Z]: notes { instrument: guzheng }]
"#;
    let (decls, errors) = parse_parts(content, &[]);
    assert!(errors.is_empty());
    assert_eq!(decls[0].soundfont, Soundfont(40));
    assert_eq!(decls[1].soundfont, Soundfont(107));
}

#[test]
fn quoted_soundfont_overrides_the_instruments_sound() {
    let content = r#"parts ["Violin" [V]: notes { instrument: violin, sound 0 }]
"#;
    let (decls, errors) = parse_parts(content, &[]);
    assert!(errors.is_empty());
    assert_eq!(decls[0].soundfont, Soundfont(0));
}

#[test]
fn follow_part_naming_its_own_instrument_plays_that_instruments_sound() {
    let content = r#"parts [
    "Violin" [V]: notes { instrument: violin },
    "Cello" [C]: follow [V] { instrument: cello, octave: - 1 },
]
"#;
    let (decls, errors) = parse_parts(content, &[]);
    assert!(errors.is_empty());
    assert_eq!(decls[1].soundfont, Soundfont(42));
}

#[test]
fn follow_part_naming_no_instrument_plays_its_targets_sound() {
    let content = r#"parts ["Violin" [V]: notes { instrument: violin }, "Violin 2" [V2]: follow [V] { octave: - 1 }]
"#;
    let (decls, errors) = parse_parts(content, &[]);
    assert!(errors.is_empty());
    assert_eq!(decls[1].soundfont, Soundfont(40));
}

#[test]
fn source_level_instrument_labels_its_sound_without_writing_a_program() {
    use crate::list_part_declarations_from_source;

    let violin = super::InstrumentInfo {
        name: "Violin".to_owned(),
        program: 40,
        category: String::new(),
        source: String::new(),
        role: String::new(),
        articulation: String::new(),
    };
    let source = r#"parts ["Violin" [V]: notes { instrument: violin }],

score """
    1 - - -
    """,
"#;
    let declarations =
        list_part_declarations_from_source(source, "test.poentaro", &[violin]).unwrap();
    assert_eq!(declarations[0].settings.program, None);
    assert_eq!(declarations[0].sound_label.as_deref(), Some("40: Violin"));
}
