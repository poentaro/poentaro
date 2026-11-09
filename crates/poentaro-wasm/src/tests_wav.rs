use crate::responses::{generate_wav_for_measure_range_response, generate_wav_response};
use crate::types::GenerateWavResponse;

#[test]
fn generate_wav_for_measure_range_response_returns_riff_wav() {
    let source = r#"metadata { title "t", author "a" },

parts ["Melody": notes],

score """
    time=4/4 key=C4 bpm=120
    [Melody] 1 2 3 4
    """,
"#;
    let soundfont = include_bytes!("../../../fonts/GeneralUser_GS.sf2").to_vec();
    let resp = generate_wav_for_measure_range_response(
        source, 0, 0, false, true, None, None, false, None, soundfont,
    );
    match resp {
        GenerateWavResponse::Ok { wav } => {
            assert!(wav.len() > 4);
            assert_eq!(&wav[0..4], b"RIFF");
        }
        GenerateWavResponse::Err { diagnostics } => {
            panic!("expected Ok: {}", diagnostics[0].message);
        }
    }
}

#[test]
fn reference_poentaro_generates_wav() {
    let soundfont = include_bytes!("../../../fonts/GeneralUser_GS.sf2").to_vec();
    for path in super::demo_file_paths() {
        let source = super::read_demo_file(&path);
        let resp = generate_wav_response(&source, None, false, soundfont.clone());
        match resp {
            GenerateWavResponse::Ok { wav } => {
                assert!(wav.len() > 4);
                assert_eq!(&wav[0..4], b"RIFF");
            }
            GenerateWavResponse::Err { diagnostics } => {
                panic!(
                    "{path:?} failed in wasm wav path: {}",
                    diagnostics[0].message
                );
            }
        }
    }
}

#[test]
fn generate_wav_with_metronome_differs_from_without() {
    let source = r#"metadata { title "t", author "a" },

parts ["Melody": notes],

score """
    time=4/4 key=C4 bpm=120
    [Melody] 1 2 3 4
    """,
"#;
    let soundfont = include_bytes!("../../../fonts/GeneralUser_GS.sf2").to_vec();
    let wav_of =
        |metronome: bool| match generate_wav_response(source, None, metronome, soundfont.clone()) {
            GenerateWavResponse::Ok { wav } => wav,
            GenerateWavResponse::Err { diagnostics } => {
                panic!("expected Ok: {}", diagnostics[0].message);
            }
        };
    assert_ne!(wav_of(true), wav_of(false));
}
