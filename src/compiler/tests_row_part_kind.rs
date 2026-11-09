use crate::ast::parsed::PartKind;
use crate::compiler::compile;
use crate::grouper::group;
use crate::parser::parse;

#[test]
fn compiled_rows_carry_their_parts_kind() {
    let source = r#"metadata { hide resting parts: no },

parts ["S": notes, "V": lyrics [S], "C": chords, "D": percussion],

score """
    time=4/4 key=C4 bpm=120
    [S] 1 2 3 4
    [V] la la la la
    [C] 1 - - -
    [D] 1 1 1 1
    """,
"#;
    let score = group(parse(source, "test", &[]).unwrap()).unwrap();
    let kinds: Vec<(String, PartKind)> = compile(&score).blocks[0]
        .rows
        .iter()
        .map(|row| (row.id.0.clone(), row.kind))
        .collect();
    assert_eq!(
        kinds,
        vec![
            ("S".to_string(), PartKind::Notes),
            (
                "V".to_string(),
                PartKind::Lyrics {
                    target_part_index: 0
                }
            ),
            ("C".to_string(), PartKind::Chords),
            ("D".to_string(), PartKind::Percussion),
        ]
    );
}
