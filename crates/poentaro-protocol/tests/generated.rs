//! `src/generated.rs` is written from `poentaro.protocol.anyx`. Run with
//! `UPDATE_GENERATED=1` (`just generate-protocol`) to write it again.

const SCHEMA_PATH: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/poentaro.protocol.anyx");
const GENERATED_PATH: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/src/generated.rs");

#[test]
fn the_server_and_client_are_generated_from_the_schema() {
    let source = std::fs::read_to_string(SCHEMA_PATH).expect("the schema is readable");
    let schema = anyx_protocol::Schema::parse(&source).unwrap_or_else(|diagnostics| {
        let rendered = diagnostics
            .iter()
            .map(|diagnostic| diagnostic.render(&source, "poentaro.protocol.anyx"))
            .collect::<Vec<_>>();
        panic!("{}", rendered.join("\n"))
    });
    let code = anyx_protocol_generate::rust(&schema).unwrap_or_else(|error| panic!("{error}"));
    let expected = anyx_protocol_generate::format_rust(&code).expect("rustfmt is installed");
    if std::env::var_os("UPDATE_GENERATED").is_some() {
        std::fs::write(GENERATED_PATH, &expected).expect("generated.rs is writable");
    }
    let committed = std::fs::read_to_string(GENERATED_PATH).unwrap_or_default();
    assert!(
        committed == expected,
        "src/generated.rs is not what poentaro.protocol.anyx generates; run `just generate-protocol`"
    );
}
