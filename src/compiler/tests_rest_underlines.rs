use crate::compiler::{compile, types::ElementContent};
use crate::grouper::group;
use crate::parser::parse;

#[test]
fn rests_keep_their_written_underlines() {
    for (line, expected_levels) in [
        ("0_. 1_ 2_. 3 4", vec![0]),
        ("0_ 1_ 2 3 4", vec![0]),
        ("0= 1= 2_ 3 4 5", vec![0, 1]),
        ("0. 1_ 2 3", vec![]),
        ("3:{0_ 1_ 2_} 3 4 5", vec![0]),
        ("3:{0= 1= 2=} 3_ 4 5 6", vec![0, 1]),
        ("3:{0_. 1_. 2_.} 3 4_ 5", vec![0]),
    ] {
        let source = format!(
            "# metadata\ntitle=\"Rest underlines\"\n# parts\nM = notes\n# score\n[M] {line}\n"
        );
        let doc = parse(&source, "test.jianpu", &[]).unwrap();
        let score = group(doc).unwrap();
        let result = compile(&score);
        let row = &result.blocks[0].rows[0];
        let levels: Vec<_> = row
            .elements
            .iter()
            .filter_map(|element| match element.content {
                ElementContent::Underline {
                    from_column: 0,
                    level,
                    ..
                } => Some(level),
                _ => None,
            })
            .collect();
        assert_eq!(levels, expected_levels, "{line}");
    }
}
