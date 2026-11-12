use crate::ast::parsed::{AlternativeLine, PartKind};

fn declarations_of(score_body: &str) -> Vec<crate::ast::parsed::PartDecl> {
    let source = format!(
        "parts [\"Melody\" [M]: notes, \"Words\" [w]: lyrics [M], \"Bass\" [B]: notes],\nscore \"\"\"\n{score_body}\"\"\",\n"
    );
    crate::parser::parse(&source, "test.poentaro", &[])
        .expect("parses")
        .declarations
}

fn abbreviations(declarations: &[crate::ast::parsed::PartDecl]) -> Vec<&str> {
    declarations
        .iter()
        .map(|declaration| declaration.abbreviation.as_str())
        .collect()
}

#[test]
fn a_score_writing_each_part_once_declares_no_alternative_lines() {
    let declarations = declarations_of("[M] 1 2 3 4\n[w] 一 二 三 四\n[B] 1 1 1 1\n");
    assert_eq!(abbreviations(&declarations), vec!["M", "w", "B"]);
}

#[test]
fn each_extra_line_is_declared_right_after_its_part() {
    let declarations = declarations_of(
        "[M] 1 2 3 4\n[w] 一 二 三 四\n[w] 五 六 七 八\n[B] 1 1 1 1\n\n[M] 1 2 3 4\n[w] 一 二 三 四\n[w] 五 六 七 八\n[w] 九 十 百 千\n[B] 1 1 1 1\n",
    );
    assert_eq!(
        abbreviations(&declarations),
        vec!["M", "w", "w.2", "w.3", "B"]
    );
    assert_eq!(
        declarations[2].alternative_line,
        Some(AlternativeLine {
            base: "w".to_string(),
            number: 2
        })
    );
}

#[test]
fn a_lyric_line_still_sings_on_its_part_after_lines_are_added_before_it() {
    let declarations = declarations_of("[M] 1 2 3 4\n[M] 3 2 1 2\n[w] 一 二 三 四\n[B] 1 1 1 1\n");
    assert_eq!(abbreviations(&declarations), vec!["M", "M.2", "w", "B"]);
    assert!(matches!(
        declarations[2].kind,
        PartKind::Lyrics {
            target_part_index: 0
        }
    ));
    assert_eq!(declarations[2].lyric_targets, vec![0]);
}

#[test]
fn an_alternative_abbreviation_names_its_part_and_line() {
    assert_eq!(
        AlternativeLine::of("S1.2"),
        Some(AlternativeLine {
            base: "S1".to_string(),
            number: 2
        })
    );
    assert_eq!(AlternativeLine::of("S1"), None);
    assert_eq!(AlternativeLine::of("S.1"), None);
}
