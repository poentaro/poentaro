//! `src/generated.rs` is written from `poentaro.anyfix`. Run with
//! `UPDATE_GENERATED=1` (`just generate-parser`) to write it again.

const GRAMMAR_PATH: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/poentaro.anyfix");
const GENERATED_PATH: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/src/generated.rs");

#[test]
fn the_decoder_is_generated_from_the_grammar() {
    let grammar = std::fs::read_to_string(GRAMMAR_PATH).expect("the grammar is readable");
    let generated = anyfix_generator::generate(&grammar).unwrap_or_else(|diagnostics| {
        let rendered = diagnostics
            .iter()
            .map(|diagnostic| diagnostic.render(&grammar, "poentaro.anyfix"))
            .collect::<Vec<_>>();
        panic!("{}", rendered.join("\n"))
    });
    let warnings = generated
        .warnings
        .iter()
        .map(|warning| warning.render(&grammar, "poentaro.anyfix"))
        .collect::<Vec<_>>();
    assert!(warnings.is_empty(), "{}", warnings.join("\n"));
    let expected = anyfix_generator::format(&generated.code).expect("rustfmt is installed");
    if std::env::var_os("UPDATE_GENERATED").is_some() {
        std::fs::write(GENERATED_PATH, &expected).expect("generated.rs is writable");
    }
    let committed = std::fs::read_to_string(GENERATED_PATH).expect("generated.rs is readable");
    assert!(
        committed == expected,
        "src/generated.rs is not what poentaro.anyfix generates; run `just generate-parser`"
    );
}
