use itertools::Itertools;

use super::anyfix_errors;
use crate::ast::parsed::{MetadataTextKey, Notation};
use crate::part_stream::{apply_part_stream, extract_part_stream};
use crate::source_edit::{
    self, MetadataEdit, NewPreset, PartMode, PartSettings, PartShown, PresetEdit,
};
use crate::symbols::{rename_edits_in_text, SymbolKind};

const DOCUMENT: &str = r##"# A song
metadata { title : "Hello" },

# The voices
parts [
  "Soprano" [S] : notes,
  "Alto" [A] : notes,
],

# The tune
score """
  label="Verse"
  [S] 1 2 3 4 // first line
  [A] 5 6 7 1'

  [S] 2 2 2 2
  [A] 3 3 3 3
  """,
"##;

const WITH_A_PRESET: &str = r##"parts [ "Soprano" [S] : notes, "Alto" [A] : notes ],

# Just the top
preset "Choir" { visible parts : [S], notation : cello },

# The tune
score """
  [S] 1 2 3 4
  [A] 5 6 7 1
  """,
"##;

fn assert_reads(text: &str) {
    let errors = anyfix_errors(text, &[]);
    assert!(errors.is_empty(), "{errors:?}\n{text}");
}

#[test]
fn text_in_the_old_section_syntax_opens_with_syntax_errors() {
    let text =
        "# metadata\ntitle = \"Song\"\n\n# parts\nMelody = notes\n\n# score\n[Melody] 1 2 3 4\n";
    assert!(!anyfix_errors(text, &[]).is_empty());
    let score = crate::compile(text, "test.poentaro", &[]).unwrap();
    assert!(!score.document_diagnostics.is_empty());
    assert_eq!(crate::format_anyfix::format_anyfix(text), text);
}

#[test]
fn blank_text_opens() {
    assert!(crate::compile(" \n", "test.poentaro", &[]).is_ok());
}

#[test]
fn measures_appended_to_an_empty_one_line_string_make_it_a_raw_string() {
    let text = "parts [ \"S\" [S] : notes ],\nscore \"\",\n";
    let applied = apply_part_stream(text, "S", "1 2 3 4").source.unwrap();
    assert_reads(&applied);
    assert_eq!(
        applied,
        "parts [ \"S\" [S] : notes ],\nscore \"\"\"\n    [S] 1 2 3 4\n    \"\"\",\n"
    );
}

#[test]
fn every_demo_reads() {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("demo");
    for entry in std::fs::read_dir(root).unwrap() {
        let path = entry.unwrap().path();
        let source = std::fs::read_to_string(&path).unwrap();
        let errors = anyfix_errors(&source, &[]);
        assert!(errors.is_empty(), "{path:?}: {errors:?}");
    }
}

#[test]
fn spans_of_measures_are_spans_in_the_text() {
    let spans = crate::measure_spans::list_measure_spans_from_source(DOCUMENT, "test.poentaro")
        .unwrap()
        .spans;
    let second = spans.get(1).unwrap();
    assert!(DOCUMENT[second.start..second.end].contains("2 2 2 2"));
    assert_eq!(second.start_line, 16);
}

// ── metadata edits ───────────────────────────────────────────────────────

#[test]
fn a_new_metadata_field_changes_only_its_section() {
    let edited = source_edit::update_metadata_field(
        DOCUMENT,
        None,
        MetadataEdit::Text {
            key: MetadataTextKey::Author,
            value: Some("Me".to_owned()),
        },
    );
    assert_reads(&edited);
    assert_eq!(
        source_edit::parse_metadata_fields(&edited, None)
            .metadata
            .author
            .as_deref(),
        Some("Me")
    );
    for kept in ["# A song", "# The voices", "# The tune", "// first line"] {
        assert!(edited.contains(kept), "lost {kept:?}:\n{edited}");
    }
    let parts_onward = |text: &str| text[text.find("# The voices").unwrap()..].to_owned();
    assert_eq!(parts_onward(&edited), parts_onward(DOCUMENT));
}

#[test]
fn a_metadata_edit_keeps_what_could_not_be_converted() {
    let text =
        "metadata { title : \"Song\", could not convert \"garbage\" },\nparts [],\nscore \"\",\n";
    let edited = source_edit::update_metadata_field(
        text,
        None,
        MetadataEdit::Text {
            key: MetadataTextKey::Title,
            value: Some("New".to_owned()),
        },
    );
    assert!(edited.contains("could not convert \"garbage\""), "{edited}");
    assert!(edited.contains("\"New\""), "{edited}");
}

// ── preset edits ─────────────────────────────────────────────────────────

#[test]
fn presets_are_listed_from_the_tree() {
    let presets = source_edit::list_presets(WITH_A_PRESET);
    assert_eq!(presets.len(), 1);
    assert_eq!(presets[0].name, "Choir");
    assert_eq!(presets[0].notation, Some(Notation::Cello));
}

#[test]
fn a_new_preset_goes_after_the_last_preset() {
    let edited = source_edit::add_preset_section(
        WITH_A_PRESET,
        NewPreset {
            name: "Band".to_owned(),
            visible_parts: vec!["A".to_owned()],
            soloed_parts: Vec::new(),
            notation: Notation::Numbered,
        },
    );
    assert_reads(&edited);
    let names = source_edit::list_presets(&edited)
        .into_iter()
        .map(|preset| preset.name)
        .collect::<Vec<_>>();
    assert_eq!(names, ["Choir", "Band"]);
    assert!(
        edited.contains("# Just the top\npreset \"Choir\""),
        "{edited}"
    );
    assert!(edited.contains("# The tune\nscore"), "{edited}");
}

#[test]
fn a_first_preset_goes_above_the_comment_on_the_score() {
    let edited = source_edit::add_preset_section(
        DOCUMENT,
        NewPreset {
            name: "Choir".to_owned(),
            visible_parts: vec!["S".to_owned()],
            soloed_parts: vec!["S".to_owned()],
            notation: Notation::Numbered,
        },
    );
    assert_reads(&edited);
    assert!(edited.contains("# The tune\nscore"), "{edited}");
    assert!(edited.find("preset").unwrap() < edited.find("# The tune").unwrap());
    assert_eq!(source_edit::list_presets(&edited).len(), 1);
}

#[test]
fn a_preset_field_is_replaced_in_place() {
    let edited = source_edit::update_preset_field(
        WITH_A_PRESET,
        "Choir",
        &PresetEdit::Notation(Notation::Violin),
    );
    assert_reads(&edited);
    assert_eq!(
        source_edit::list_presets(&edited)[0].notation,
        Some(Notation::Violin)
    );
    assert_eq!(edited.matches("notation").count(), 1, "{edited}");
    assert!(edited.contains("# Just the top"), "{edited}");
}

#[test]
fn an_empty_part_list_removes_its_field() {
    let soloed = source_edit::update_preset_field(
        WITH_A_PRESET,
        "Choir",
        &PresetEdit::SoloedParts(vec!["S".to_owned()]),
    );
    assert!(soloed.contains("soloed parts"), "{soloed}");
    let cleared =
        source_edit::update_preset_field(&soloed, "Choir", &PresetEdit::SoloedParts(Vec::new()));
    assert_reads(&cleared);
    assert!(!cleared.contains("soloed parts"), "{cleared}");
}

#[test]
fn hiding_a_part_lists_the_parts_left() {
    let all_parts = ["S".to_owned(), "A".to_owned()];
    let shown = source_edit::set_preset_part_shown(
        WITH_A_PRESET,
        "Choir",
        &all_parts,
        &PartShown {
            abbreviation: "A".to_owned(),
            shown: true,
        },
    );
    assert_reads(&shown);
    assert!(shown.contains("[S, A]"), "{shown}");
}

// ── part edits ───────────────────────────────────────────────────────────

#[test]
fn a_part_edit_rewrites_its_kind_and_keeps_the_score() {
    let settings = PartSettings {
        mode: PartMode::Notes,
        range_instrument: None,
        program: None,
        volume: 80,
        octave_offset: -1,
    };
    let edited = source_edit::update_part_declaration(DOCUMENT, "A", &settings).unwrap();
    assert_reads(&edited);
    let alto = crate::list_part_declarations_from_source(&edited, "test.poentaro", &[])
        .unwrap()
        .into_iter()
        .find(|declaration| declaration.abbreviation == "A")
        .unwrap();
    assert_eq!(alto.settings.mode, PartMode::Notes);
    assert_eq!(alto.settings.volume, 80);
    assert_eq!(alto.settings.octave_offset, -1);
    assert!(edited.contains("# The tune"), "{edited}");
    assert!(edited.contains("// first line"), "{edited}");
}

#[test]
fn editing_a_part_that_does_not_exist_changes_nothing() {
    let settings = PartSettings {
        mode: PartMode::Notes,
        range_instrument: None,
        program: None,
        volume: 100,
        octave_offset: 0,
    };
    assert_eq!(
        source_edit::update_part_declaration(DOCUMENT, "Nope", &settings),
        None
    );
}

// ── rename ───────────────────────────────────────────────────────────────

#[test]
fn renaming_a_part_spells_it_for_where_it_sits() {
    let document = crate::parser::parse(WITH_A_PRESET, "test.poentaro", &[]).unwrap();
    let edits = rename_edits_in_text(
        WITH_A_PRESET,
        &document,
        SymbolKind::Abbreviation,
        "A",
        "A 2",
    );
    let renamed = edits
        .iter()
        .sorted_by_key(|edit| std::cmp::Reverse(edit.span.start))
        .fold(WITH_A_PRESET.to_owned(), |mut text, edit| {
            text.replace_range(edit.span.start..edit.span.end, &edit.replacement);
            text
        });
    assert!(renamed.contains("\"Alto\" [\"A 2\"]"), "{renamed}");
    assert!(renamed.contains("[A 2] 5 6 7 1"), "{renamed}");
}

// ── part stream ──────────────────────────────────────────────────────────

#[test]
fn a_part_stream_edit_stays_indented_in_the_raw_string() {
    let stream = extract_part_stream(DOCUMENT, "S").unwrap();
    let edited = stream.text.replace("2 2 2 2", "2 2 2 5");
    let applied = apply_part_stream(DOCUMENT, "S", &edited).source.unwrap();
    assert_eq!(applied, DOCUMENT.replace("[S] 2 2 2 2", "[S] 2 2 2 5"));
}

#[test]
fn a_measure_written_for_a_part_without_one_keeps_the_indent() {
    let text =
        "parts [ \"S\" [S] : notes, \"A\" [A] : notes ],\nscore \"\"\"\n  [S] 1 2 3 4\n  \"\"\",\n";
    let applied = apply_part_stream(text, "A", "5 5 5 5").source.unwrap();
    assert_reads(&applied);
    assert!(applied.contains("\n  [A] 5"), "{applied}");
}

#[test]
fn measures_appended_to_an_empty_score_keep_the_closing_quotes_alone() {
    let text = "parts [ \"S\" [S] : notes ],\nscore \"\"\"\n  \"\"\",\n";
    let applied = apply_part_stream(text, "S", "1 2 3 4").source.unwrap();
    assert_reads(&applied);
    assert_eq!(
        applied,
        "parts [ \"S\" [S] : notes ],\nscore \"\"\"\n  [S] 1 2 3 4\n  \"\"\",\n"
    );
}
