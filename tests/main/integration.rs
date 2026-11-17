#![allow(clippy::disallowed_macros)]
use poentaro::cli::generate::{generate_midi, generate_pdf, generate_svg, generate_wav};
use poentaro::cli::GenerateInput;
use std::fs;
use std::path::PathBuf;

fn basic_poentaro_input() -> &'static str {
    r#"metadata { title "test score", author "tester" },

parts ["Melody": notes, "Melody lyrics" [Melodyv1]: lyrics [Melody]],

score """
    time=4/4 key=C4 bpm=120
    [Melody] 1 2 3 4
    [Melodyv1] do re mi fa
    """,
"#
}

#[test]
fn generate_pdf_produces_pdf() {
    let input_path = "/tmp/test_pdf_basic.poentaro";
    let output_stem_arg = "/tmp/test_pdf_basic";
    let output_path = "/tmp/test_pdf_basic.pdf";
    fs::write(input_path, basic_poentaro_input()).unwrap();

    generate_pdf(&GenerateInput {
        input: PathBuf::from(input_path),
        output: Some(PathBuf::from(output_stem_arg)),
        tracks: vec![],
        split_tracks: false,
        preset: None,
    })
    .expect("generate pdf command failed");

    let bytes = fs::read(output_path).unwrap();
    assert!(bytes.starts_with(b"%PDF"), "output is not a valid PDF");

    fs::remove_file(input_path).ok();
    fs::remove_file(output_path).ok();
}

#[test]
fn generate_midi_produces_midi() {
    let input_path = "/tmp/test_score_midi.poentaro";
    let output_stem_arg = "/tmp/test_score_midi_out";
    let output_path = "/tmp/test_score_midi_out.mid";
    fs::write(input_path, basic_poentaro_input()).unwrap();

    generate_midi(&GenerateInput {
        input: PathBuf::from(input_path),
        output: Some(PathBuf::from(output_stem_arg)),
        tracks: vec![],
        split_tracks: false,
        preset: None,
    })
    .expect("generate midi command failed");

    let bytes = fs::read(output_path).unwrap();
    // MIDI files start with "MThd"
    assert!(
        bytes.starts_with(b"MThd"),
        "output is not a valid MIDI file"
    );

    fs::remove_file(input_path).ok();
    fs::remove_file(output_path).ok();
}

#[test]
fn output_stem_appends_extension() {
    let input_path = "/tmp/test_stem.poentaro";
    let output_stem = "/tmp/test_stem_out";
    let expected_output = "/tmp/test_stem_out.pdf";
    fs::write(input_path, basic_poentaro_input()).unwrap();
    fs::remove_file(expected_output).ok();

    generate_pdf(&GenerateInput {
        input: PathBuf::from(input_path),
        output: Some(PathBuf::from(output_stem)),
        tracks: vec![],
        split_tracks: false,
        preset: None,
    })
    .expect("generate pdf command failed");

    let bytes = fs::read(expected_output).expect("output file not found at expected stem path");
    assert!(bytes.starts_with(b"%PDF"), "output is not a valid PDF");

    fs::remove_file(input_path).ok();
    fs::remove_file(expected_output).ok();
}

#[test]
fn generate_wav_produces_wav() {
    let input_path = "/tmp/test_score_wav.poentaro";
    let output_stem_arg = "/tmp/test_score_wav_out";
    let output_path = "/tmp/test_score_wav_out.wav";
    fs::write(input_path, basic_poentaro_input()).unwrap();

    generate_wav(&GenerateInput {
        input: PathBuf::from(input_path),
        output: Some(PathBuf::from(output_stem_arg)),
        tracks: vec![],
        split_tracks: false,
        preset: None,
    })
    .expect("generate wav command failed");

    let bytes = fs::read(output_path).unwrap();
    assert_eq!(&bytes[0..4], b"RIFF", "output is not a valid WAV file");
    assert_eq!(&bytes[8..12], b"WAVE", "output is not a valid WAV file");
    // Expect meaningful audio data (>100 KB for a few seconds of stereo 16-bit 44100 Hz)
    assert!(bytes.len() > 100_000, "WAV output is suspiciously small");

    fs::remove_file(input_path).ok();
    fs::remove_file(output_path).ok();
}

fn multi_track_poentaro_input() -> &'static str {
    r#"metadata { title "test score", author "tester" },

parts [
    "Soprano 1" [S1]: notes,
    "Soprano 1 lyrics" [S1v1]: lyrics [S1],
    "Soprano 2" [S2]: notes,
    "Soprano 2 lyrics" [S2v1]: lyrics [S2],
],

score """
    time=4/4 key=C4 bpm=120
    [S1] 1 2 3 4
    [S1v1] do re mi fa
    [S2] 5 6 7 1
    [S2v1] sol la ti do
    """,
"#
}

#[test]
fn split_tracks_generates_one_pdf_per_track() {
    let input_path = "/tmp/test_split.poentaro";
    let s1_path = "/tmp/test_split - Soprano 1.pdf";
    let s2_path = "/tmp/test_split - Soprano 2.pdf";
    fs::write(input_path, multi_track_poentaro_input()).unwrap();
    fs::remove_file(s1_path).ok();
    fs::remove_file(s2_path).ok();

    generate_pdf(&GenerateInput {
        input: PathBuf::from(input_path),
        output: None,
        tracks: vec![],
        split_tracks: true,
        preset: None,
    })
    .expect("generate pdf --split-tracks command failed");

    let s1_bytes = fs::read(s1_path).expect("S1 output file not found");
    assert!(
        s1_bytes.starts_with(b"%PDF"),
        "S1 output is not a valid PDF"
    );

    let s2_bytes = fs::read(s2_path).expect("S2 output file not found");
    assert!(
        s2_bytes.starts_with(b"%PDF"),
        "S2 output is not a valid PDF"
    );

    fs::remove_file(input_path).ok();
    fs::remove_file(s1_path).ok();
    fs::remove_file(s2_path).ok();
}

#[test]
fn split_tracks_with_output_stem() {
    let input_path = "/tmp/test_split_out.poentaro";
    let s1_path = "/tmp/split_out - Soprano 1.pdf";
    let s2_path = "/tmp/split_out - Soprano 2.pdf";
    fs::write(input_path, multi_track_poentaro_input()).unwrap();
    fs::remove_file(s1_path).ok();
    fs::remove_file(s2_path).ok();

    generate_pdf(&GenerateInput {
        input: PathBuf::from(input_path),
        output: Some(PathBuf::from("/tmp/split_out")),
        tracks: vec![],
        split_tracks: true,
        preset: None,
    })
    .expect("generate pdf --output --split-tracks command failed");

    let s1_bytes = fs::read(s1_path).expect("S1 output file not found");
    assert!(
        s1_bytes.starts_with(b"%PDF"),
        "S1 output is not a valid PDF"
    );

    let s2_bytes = fs::read(s2_path).expect("S2 output file not found");
    assert!(
        s2_bytes.starts_with(b"%PDF"),
        "S2 output is not a valid PDF"
    );

    fs::remove_file(input_path).ok();
    fs::remove_file(s1_path).ok();
    fs::remove_file(s2_path).ok();
}

fn choir_preset_input() -> &'static str {
    r#"metadata { title "Song", subtitle "Full score" },

parts ["Soprano" [S]: notes, "Alto" [A]: notes],

score """
    time=4/4 key=C4 bpm=120
    [S] 1 2 3 4
    [A] 5 6 7 1
    """,

preset "Choir" { visible parts [A], subtitle "Alto part" },
"#
}

#[test]
fn preset_exports_its_parts_and_metadata_named_after_it() {
    let input_path = "/tmp/test_preset.poentaro";
    let output_path = "/tmp/test_preset - Choir.svg";
    fs::write(input_path, choir_preset_input()).unwrap();
    fs::remove_file(output_path).ok();

    generate_svg(&GenerateInput {
        input: PathBuf::from(input_path),
        output: None,
        tracks: vec![],
        split_tracks: false,
        preset: Some("Choir".to_string()),
    })
    .expect("generate svg --preset command failed");

    let svg = fs::read_to_string(output_path).expect("preset output file not found");
    assert!(svg.contains("Alto part"));
    assert!(svg.contains("A \u{2014} Alto"));
    assert!(!svg.contains("S \u{2014} Soprano"));

    fs::remove_file(input_path).ok();
    fs::remove_file(output_path).ok();
}

#[test]
fn unknown_preset_is_an_error() {
    let input_path = "/tmp/test_unknown_preset.poentaro";
    fs::write(input_path, choir_preset_input()).unwrap();

    let error = generate_svg(&GenerateInput {
        input: PathBuf::from(input_path),
        output: None,
        tracks: vec![],
        split_tracks: false,
        preset: Some("Nope".to_string()),
    })
    .expect_err("an unknown preset should fail");
    assert!(error.to_string().contains("Nope"));

    fs::remove_file(input_path).ok();
}
