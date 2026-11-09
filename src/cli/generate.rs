use crate as jg;
use std::path::{Path, PathBuf};

fn output_stem(input: &Path, tracks: &[String], output: Option<&Path>) -> PathBuf {
    match output {
        Some(out) => out.with_extension(""),
        None => {
            let stem = input.file_stem().unwrap_or_default().to_string_lossy();
            let suffix = if tracks.is_empty() {
                stem.into_owned()
            } else {
                format!("{} - {}", stem, tracks.join("&"))
            };
            input.with_file_name(suffix)
        }
    }
}

pub struct GenerateInput {
    pub input: PathBuf,
    pub output: Option<PathBuf>,
    pub tracks: Vec<String>,
    pub split_tracks: bool,
    /// The `preset` whose metadata and visible (or soloed) parts to export.
    pub preset: Option<String>,
}

/// The parts to export: `--tracks` when given, otherwise the `--preset`'s
/// soloed parts, or else its visible parts. Empty means every part.
fn selected_tracks(opts: &GenerateInput) -> Result<Vec<String>, jg::error::IrrecoverableError> {
    let preset_parts = match opts.preset.as_deref() {
        Some(name) => chosen_preset(&opts.input, name)?.exported_parts(),
        None => Vec::new(),
    };
    Ok(if opts.tracks.is_empty() {
        preset_parts
    } else {
        opts.tracks.clone()
    })
}

fn chosen_preset(
    input: &Path,
    name: &str,
) -> Result<jg::ChosenPreset, jg::error::IrrecoverableError> {
    let source = super::read_document_source(input)?;
    jg::chosen_preset_from_source(&source, &input.to_string_lossy(), name)?.ok_or_else(|| {
        jg::error::IrrecoverableError::new(jg::error::IrrecoverableErrorKind::UnknownPreset {
            span: jg::error::Span::new(0, 0),
            name: name.to_string(),
        })
        .with_path(input)
    })
}

/// Names the output file after `--tracks`, or else after `--preset`.
fn output_label(opts: &GenerateInput) -> Vec<String> {
    if opts.tracks.is_empty() {
        opts.preset.iter().cloned().collect()
    } else {
        opts.tracks.clone()
    }
}

fn visibility(opts: &GenerateInput, tracks: &[String]) -> jg::ResolvedPartVisibility {
    jg::ResolvedPartVisibility {
        rendered_tracks: (!tracks.is_empty()).then(|| tracks.to_vec()),
        preset_name: opts.preset.clone(),
        ..Default::default()
    }
}

fn effective_tracks(tracks: &[String], score: &jg::ast::grouped::Score) -> Vec<String> {
    if tracks.is_empty() {
        jg::collect_track_names(score)
    } else {
        tracks.to_vec()
    }
}

fn split_track_base(input: &Path, output: Option<&Path>) -> (PathBuf, String) {
    let base = output_stem(input, &[], output);
    let base_name = base
        .file_stem()
        .unwrap_or_default()
        .to_string_lossy()
        .into_owned();
    (base, base_name)
}

fn track_output_path(base: &Path, base_name: &str, label: &str, extension: &str) -> PathBuf {
    base.with_file_name(jg::split_track_filename(base_name, label, extension))
}

/// Returns `true` when split-track output was written and the caller should return early.
fn try_split_tracks<F>(
    score: &jg::ast::grouped::Score,
    input: &Path,
    output: Option<&Path>,
    tracks: &[String],
    display_names: &std::collections::HashMap<String, String>,
    mut write_track: F,
) -> Result<bool, jg::error::IrrecoverableError>
where
    F: FnMut(
        &jg::ast::grouped::Score,
        &str,
        &str,
        &Path,
        &str,
    ) -> Result<(), jg::error::IrrecoverableError>,
{
    let effective_tracks = effective_tracks(tracks, score);
    if effective_tracks.is_empty() {
        eprintln!(
            "warning: --split-tracks given but score has no named tracks; generating single file"
        );
        return Ok(false);
    }

    let (base, base_name) = split_track_base(input, output);
    for track in &effective_tracks {
        let mut score_clone = score.clone();
        jg::filter_tracks(
            &mut score_clone,
            &jg::track_with_its_lyric_parts(score, track),
        );
        let label = jg::split_track_label(display_names, track);
        write_track(&score_clone, track, &label, &base, &base_name)?;
    }
    Ok(true)
}

fn write_svgs_to_path(
    svgs: &[String],
    output_path: &Path,
) -> Result<(), jg::error::IrrecoverableError> {
    for (i, svg) in svgs.iter().enumerate() {
        let path = if svgs.len() == 1 {
            output_path.to_path_buf()
        } else {
            output_path.with_extension(format!("{}.svg", i + 1))
        };
        super::write_file(&path, svg.as_bytes())?;
        println!("written to {path:?}");
    }
    Ok(())
}

#[cfg(feature = "pdf")]
pub fn generate_pdf(opts: &GenerateInput) -> Result<(), jg::error::IrrecoverableError> {
    let tracks = selected_tracks(opts)?;
    if opts.split_tracks {
        let content = std::fs::read_to_string(&opts.input).map_err(|e| {
            jg::error::IrrecoverableError::new(jg::error::IrrecoverableErrorKind::IoReadFailed {
                span: jg::error::Span::new(0, 0),
                path: opts.input.clone(),
                source: e.to_string(),
            })
        })?;
        let filename = opts.input.to_string_lossy();
        let (_, base_name) = split_track_base(&opts.input, opts.output.as_deref());
        let entries = jg::write_split_pdfs_from_source(
            &content,
            &filename,
            &base_name,
            jg::SplitSelection {
                tracks: &opts.tracks,
                preset_name: opts.preset.as_deref(),
            },
            &crate::fonts::FontBytesByFamily::embedded(),
            jg::NoteLabels::Degrees,
        )?;
        if entries.is_empty() {
            eprintln!(
                "warning: --split-tracks given but score has no named tracks; generating single file"
            );
        } else {
            let (base, _) = split_track_base(&opts.input, opts.output.as_deref());
            for entry in &entries {
                let track_path = base.with_file_name(&entry.filename);
                super::write_file(&track_path, &entry.pdf)?;
                println!("written to {track_path:?}");
            }
            return Ok(());
        }
    }

    let text = super::read_document_source(&opts.input)?;
    let filename = opts.input.to_string_lossy();
    let pdf_bytes = jg::write_pdf_from_source_with_visibility(
        &text,
        &filename,
        &visibility(opts, &tracks),
        &crate::fonts::FontBytesByFamily::embedded(),
        &[],
        jg::NoteLabels::Degrees,
    )?;
    let output_path =
        output_stem(&opts.input, &output_label(opts), opts.output.as_deref()).with_extension("pdf");
    super::write_file(&output_path, &pdf_bytes)?;
    println!("written to {output_path:?}");
    Ok(())
}

fn read_display_names(
    input: &Path,
) -> Result<std::collections::HashMap<String, String>, jg::error::IrrecoverableError> {
    let content = std::fs::read_to_string(input).map_err(|e| {
        jg::error::IrrecoverableError::new(jg::error::IrrecoverableErrorKind::IoReadFailed {
            span: jg::error::Span::new(0, 0),
            path: input.to_path_buf(),
            source: e.to_string(),
        })
    })?;
    let filename = input.to_string_lossy();
    jg::part_display_name_map(&content, &filename)
}

pub fn generate_svg(opts: &GenerateInput) -> Result<(), jg::error::IrrecoverableError> {
    let tracks = selected_tracks(opts)?;
    let score = super::parse_and_group(&opts.input, opts.preset.as_deref())?;
    if opts.split_tracks {
        let display_names = read_display_names(&opts.input)?;
        let split = try_split_tracks(
            &score,
            &opts.input,
            opts.output.as_deref(),
            &tracks,
            &display_names,
            |score_clone, _, label, base, base_name| {
                let svgs = jg::render_svgs(score_clone)?;
                for (i, svg) in svgs.iter().enumerate() {
                    let path = if svgs.len() == 1 {
                        base.with_file_name(jg::split_track_filename(base_name, label, "svg"))
                    } else {
                        base.with_file_name(format!(
                            "{} - {}.{}.svg",
                            base_name,
                            jg::sanitize_track_name(label),
                            i + 1
                        ))
                    };
                    super::write_file(&path, svg.as_bytes())?;
                    println!("written to {path:?}");
                }
                Ok(())
            },
        )?;
        if split {
            return Ok(());
        }
    }

    let content = super::read_document_source(&opts.input)?;
    let filename = opts.input.to_string_lossy();
    let render_output = jg::render_svgs_from_source_with_visibility(
        &content,
        &filename,
        &visibility(opts, &tracks),
        &[],
        jg::NoteLabels::Degrees,
    )?;
    let output_path =
        output_stem(&opts.input, &output_label(opts), opts.output.as_deref()).with_extension("svg");
    write_svgs_to_path(&render_output.svgs, &output_path)
}

#[cfg(feature = "midi")]
pub fn generate_midi(opts: &GenerateInput) -> Result<(), jg::error::IrrecoverableError> {
    let tracks = selected_tracks(opts)?;
    let score = super::parse_and_group(&opts.input, opts.preset.as_deref())?;
    if opts.split_tracks {
        let display_names = read_display_names(&opts.input)?;
        let split = try_split_tracks(
            &score,
            &opts.input,
            opts.output.as_deref(),
            &tracks,
            &display_names,
            |score_clone, _, label, base, base_name| {
                let midi_bytes = jg::midi::write_midi(score_clone)?;
                let track_path = track_output_path(base, base_name, label, "mid");
                super::write_file(&track_path, &midi_bytes)?;
                println!("written to {track_path:?}");
                Ok(())
            },
        )?;
        if split {
            return Ok(());
        }
    }

    let mut score = score;
    jg::filter_tracks(&mut score, &tracks);
    let midi_bytes = jg::midi::write_midi(&score)?;
    let output_path =
        output_stem(&opts.input, &output_label(opts), opts.output.as_deref()).with_extension("mid");
    super::write_file(&output_path, &midi_bytes)?;
    println!("written to {output_path:?}");
    Ok(())
}

#[cfg(feature = "wav")]
pub fn generate_wav(opts: &GenerateInput) -> Result<(), jg::error::IrrecoverableError> {
    let tracks = selected_tracks(opts)?;
    let score = super::parse_and_group(&opts.input, opts.preset.as_deref())?;
    if opts.split_tracks {
        let display_names = read_display_names(&opts.input)?;
        let split = try_split_tracks(
            &score,
            &opts.input,
            opts.output.as_deref(),
            &tracks,
            &display_names,
            |score_clone, _, label, base, base_name| {
                let midi_bytes = jg::midi::write_midi(score_clone)?;
                let wav_bytes = jg::wav::write_wav(&midi_bytes, super::SF2_BYTES, None)?;
                let track_path = track_output_path(base, base_name, label, "wav");
                super::write_file(&track_path, &wav_bytes)?;
                println!("written to {track_path:?}");
                Ok(())
            },
        )?;
        if split {
            return Ok(());
        }
    }

    let mut score = score;
    jg::filter_tracks(&mut score, &tracks);
    let midi_bytes = jg::midi::write_midi(&score)?;
    let wav_bytes = jg::wav::write_wav(&midi_bytes, super::SF2_BYTES, None)?;
    let output_path =
        output_stem(&opts.input, &output_label(opts), opts.output.as_deref()).with_extension("wav");
    super::write_file(&output_path, &wav_bytes)?;
    println!("written to {output_path:?}");
    Ok(())
}

#[cfg(feature = "mp3")]
pub fn generate_mp3(opts: &GenerateInput) -> Result<(), jg::error::IrrecoverableError> {
    let tracks = selected_tracks(opts)?;
    let score = super::parse_and_group(&opts.input, opts.preset.as_deref())?;
    if opts.split_tracks {
        let display_names = read_display_names(&opts.input)?;
        let split = try_split_tracks(
            &score,
            &opts.input,
            opts.output.as_deref(),
            &tracks,
            &display_names,
            |score_clone, _, label, base, base_name| {
                let midi_bytes = jg::midi::write_midi(score_clone)?;
                let mp3_bytes = jg::wav::write_mp3(&midi_bytes, super::SF2_BYTES, None)?;
                let track_path = track_output_path(base, base_name, label, "mp3");
                super::write_file(&track_path, &mp3_bytes)?;
                println!("written to {track_path:?}");
                Ok(())
            },
        )?;
        if split {
            return Ok(());
        }
    }

    let mut score = score;
    jg::filter_tracks(&mut score, &tracks);
    let midi_bytes = jg::midi::write_midi(&score)?;
    let mp3_bytes = jg::wav::write_mp3(&midi_bytes, super::SF2_BYTES, None)?;
    let output_path =
        output_stem(&opts.input, &output_label(opts), opts.output.as_deref()).with_extension("mp3");
    super::write_file(&output_path, &mp3_bytes)?;
    println!("written to {output_path:?}");
    Ok(())
}

/// Experimental: write the score as MEI for Verovio to engrave as staff notation.
pub fn generate_mei(opts: &GenerateInput) -> Result<(), jg::error::IrrecoverableError> {
    let tracks = selected_tracks(opts)?;
    let mut score = super::parse_and_group(&opts.input, opts.preset.as_deref())?;
    jg::filter_tracks(&mut score, &tracks);
    let display_names = read_display_names(&opts.input)?;
    let mei = jg::mei_export::score_to_mei(
        &score,
        &display_names,
        jg::mei_export::ClefChoice::Automatic,
    )
    .mei;
    let output_path =
        output_stem(&opts.input, &output_label(opts), opts.output.as_deref()).with_extension("mei");
    super::write_file(&output_path, mei.as_bytes())?;
    println!("written to {output_path:?}");
    Ok(())
}
