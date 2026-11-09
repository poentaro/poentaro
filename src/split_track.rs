use crate::ast::grouped::Score;
use crate::error::IrrecoverableError;
#[cfg(any(feature = "pdf", feature = "midi"))]
use crate::error::{IrrecoverableErrorKind, Span};
#[cfg(any(feature = "pdf", feature = "midi"))]
use crate::filters::filter_tracks;
use crate::list_parts_from_source;
#[cfg(feature = "pdf")]
use crate::{filter_part_list, render_svgs_with_parts};

/// Sanitize a track name for use in filenames (mirrors CLI).
pub fn sanitize_track_name(name: &str) -> String {
    name.replace(['/', '\\', ':', '*', '?', '"', '<', '>', '|'], "-")
}

/// Abbreviation → display name from `parts` declarations.
pub fn part_display_name_map(
    source: &str,
    filename: &str,
) -> Result<std::collections::HashMap<String, String>, IrrecoverableError> {
    Ok(list_parts_from_source(source, filename, &[])?
        .into_iter()
        .map(|part| (part.abbreviation, part.display_name))
        .collect())
}

/// Resolve the filename label for a track (display name when declared, else abbreviation).
pub fn split_track_label(
    display_names: &std::collections::HashMap<String, String>,
    abbreviation: &str,
) -> String {
    display_names
        .get(abbreviation)
        .cloned()
        .unwrap_or_else(|| abbreviation.to_string())
}

/// Build a split-track filename: `{base_name} - {label}.{extension}`.
pub fn split_track_filename(base_name: &str, label: &str, extension: &str) -> String {
    format!(
        "{} - {}.{}",
        base_name,
        sanitize_track_name(label),
        extension
    )
}

/// `track` followed by the lyric parts that sing along to it, so a split
/// export of one track keeps its lyrics.
pub fn track_with_its_lyric_parts(score: &Score, track: &str) -> Vec<String> {
    let mut seen = std::collections::HashSet::new();
    let lyric_names = score
        .measures
        .iter()
        .flat_map(|measure| measure.parts.iter())
        .filter(|part| {
            part.slice()
                .lyrics
                .as_ref()
                .is_some_and(|lyrics| lyrics.target_name == track)
        })
        .filter_map(|part| part.name().cloned())
        .filter(|name| seen.insert(name.clone()));
    std::iter::once(track.to_string())
        .chain(lyric_names)
        .collect()
}

/// Collect unique sounding part names from score measures (order of first
/// appearance); lyric parts are excluded since they export with their target.
pub fn collect_track_names(score: &Score) -> Vec<String> {
    let mut seen = std::collections::HashSet::new();
    let mut names = Vec::new();
    for measure in &score.measures {
        for part in &measure.parts {
            if part.slice().lyrics.is_some() {
                continue;
            }
            if let Some(name) = part.name() {
                if seen.insert(name.clone()) {
                    names.push(name.clone());
                }
            }
        }
    }
    names
}

/// Build a split-track PDF filename: `{base_name} - {label}.pdf`.
pub fn split_pdf_filename(base_name: &str, label: &str) -> String {
    split_track_filename(base_name, label, "pdf")
}

/// Track list for split export: `selection.tracks` when given, else every
/// score track (falling back to the `parts` declarations when the score has
/// no named parts), narrowed to the preset's `visible_parts`.
pub fn split_track_names(
    source: &str,
    filename: &str,
    score: &Score,
    selection: SplitSelection,
) -> Result<Vec<String>, IrrecoverableError> {
    if !selection.tracks.is_empty() {
        return Ok(selection.tracks.to_vec());
    }
    let mut names = collect_track_names(score);
    if names.is_empty() {
        names = list_parts_from_source(source, filename, &[])?
            .into_iter()
            .filter(|part| part.sounds)
            .map(|part| part.abbreviation)
            .collect();
    }
    let preset = match selection.preset_name {
        Some(name) => crate::chosen_preset_from_source(source, filename, name)?,
        None => None,
    };
    if let Some(visible) = preset.and_then(|preset| preset.visible_parts) {
        names.retain(|name| visible.contains(name));
    }
    Ok(names)
}

/// One PDF produced by split-track export.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SplitPdfEntry {
    pub track_name: String,
    pub filename: String,
    pub pdf: Vec<u8>,
}

/// Which tracks a split export writes, and the `preset` it exports under.
#[derive(Debug, Clone, Copy, Default)]
pub struct SplitSelection<'a> {
    /// Empty → all tracks; non-empty → only these abbreviations.
    pub tracks: &'a [String],
    /// Overlays that `preset`'s metadata on every file, keeps only its
    /// `visible_parts` (unless `tracks` is given), and names the files after it.
    pub preset_name: Option<&'a str>,
}

impl SplitSelection<'_> {
    /// `base_name`, followed by ` (<preset>)` when exporting under a preset, the way a
    /// single export is labelled.
    pub fn file_base_name(&self, base_name: &str) -> String {
        match self.preset_name {
            Some(preset) => format!("{base_name} ({})", sanitize_track_name(preset)),
            None => base_name.to_string(),
        }
    }
}

/// Parse once, render one PDF per track (CLI `--split-tracks` semantics).
///
/// A track exports together with the lyric parts that sing along to it.
#[cfg(feature = "pdf")]
pub fn write_split_pdfs_from_source(
    source: &str,
    filename: &str,
    base_name: &str,
    selection: SplitSelection,
    fonts: &crate::fonts::FontBytesByFamily,
    note_labels: crate::compiler::NoteLabels,
) -> Result<Vec<SplitPdfEntry>, IrrecoverableError> {
    let score = crate::compile_with_preset(source, filename, &[], selection.preset_name)?;
    let track_names = split_track_names(source, filename, &score, selection)?;
    let base_name = selection.file_base_name(base_name);
    let display_names = part_display_name_map(source, filename)?;
    let all_parts = list_parts_from_source(source, filename, &[])?;
    let mut entries = Vec::with_capacity(track_names.len());
    for track in track_names {
        let mut score_clone = score.clone();
        let enabled_tracks = track_with_its_lyric_parts(&score, &track);
        filter_tracks(&mut score_clone, &enabled_tracks);
        let parts = filter_part_list(all_parts.clone(), Some(&enabled_tracks));
        let svgs = render_svgs_with_parts(&score_clone, &parts, None, note_labels)?.svgs;
        let pdf = crate::pdf::write_pdf(&svgs, fonts, None)?;
        let label = split_track_label(&display_names, &track);
        entries.push(SplitPdfEntry {
            track_name: track.clone(),
            filename: split_pdf_filename(&base_name, &label),
            pdf,
        });
    }
    Ok(entries)
}

#[cfg(feature = "pdf")]
pub fn zip_split_pdfs(entries: &[SplitPdfEntry]) -> Result<Vec<u8>, IrrecoverableError> {
    use std::io::Write;
    use zip::write::SimpleFileOptions;
    use zip::ZipWriter;

    let mut buffer = Vec::new();
    {
        let mut writer = ZipWriter::new(std::io::Cursor::new(&mut buffer));
        let options =
            SimpleFileOptions::default().compression_method(zip::CompressionMethod::Deflated);
        for entry in entries {
            writer.start_file(&entry.filename, options).map_err(|e| {
                IrrecoverableError::new(IrrecoverableErrorKind::ZipStartFileFailed {
                    span: Span::new(0, 0),
                    source: e.to_string(),
                })
            })?;
            writer.write_all(&entry.pdf).map_err(|e| {
                IrrecoverableError::new(IrrecoverableErrorKind::ZipWriteFailed {
                    span: Span::new(0, 0),
                    source: e.to_string(),
                })
            })?;
        }
        writer.finish().map_err(|e| {
            IrrecoverableError::new(IrrecoverableErrorKind::ZipFinishFailed {
                span: Span::new(0, 0),
                source: e.to_string(),
            })
        })?;
    }
    Ok(buffer)
}

/// One file (WAV or MIDI) produced by split-track export.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SplitFileEntry {
    pub track_name: String,
    pub filename: String,
    pub bytes: Vec<u8>,
}

#[cfg(any(feature = "pdf", feature = "midi"))]
pub fn zip_split_entries(entries: &[SplitFileEntry]) -> Result<Vec<u8>, IrrecoverableError> {
    use std::io::Write;
    use zip::write::SimpleFileOptions;
    use zip::ZipWriter;

    let mut buffer = Vec::new();
    {
        let mut writer = ZipWriter::new(std::io::Cursor::new(&mut buffer));
        let options =
            SimpleFileOptions::default().compression_method(zip::CompressionMethod::Deflated);
        for entry in entries {
            writer.start_file(&entry.filename, options).map_err(|e| {
                IrrecoverableError::new(IrrecoverableErrorKind::ZipStartFileFailed {
                    span: Span::new(0, 0),
                    source: e.to_string(),
                })
            })?;
            writer.write_all(&entry.bytes).map_err(|e| {
                IrrecoverableError::new(IrrecoverableErrorKind::ZipWriteFailed {
                    span: Span::new(0, 0),
                    source: e.to_string(),
                })
            })?;
        }
        writer.finish().map_err(|e| {
            IrrecoverableError::new(IrrecoverableErrorKind::ZipFinishFailed {
                span: Span::new(0, 0),
                source: e.to_string(),
            })
        })?;
    }
    Ok(buffer)
}

/// Parse once, write one MIDI file per track.
#[cfg(feature = "midi")]
pub fn write_split_midis_from_source(
    source: &str,
    filename: &str,
    base_name: &str,
    selection: SplitSelection,
) -> Result<Vec<SplitFileEntry>, IrrecoverableError> {
    let score = crate::compile_with_preset(source, filename, &[], selection.preset_name)?;
    let track_names = split_track_names(source, filename, &score, selection)?;
    let base_name = selection.file_base_name(base_name);
    let display_names = part_display_name_map(source, filename)?;
    let mut entries = Vec::with_capacity(track_names.len());
    for track in track_names {
        let mut score_clone = score.clone();
        filter_tracks(&mut score_clone, std::slice::from_ref(&track));
        let midi = crate::midi::write_midi(&score_clone)?;
        let label = split_track_label(&display_names, &track);
        entries.push(SplitFileEntry {
            track_name: track.clone(),
            filename: split_track_filename(&base_name, &label, "mid"),
            bytes: midi,
        });
    }
    Ok(entries)
}

/// Parse once, synthesize one WAV file per track.
#[cfg(feature = "wav")]
pub fn write_split_wavs_from_source(
    source: &str,
    filename: &str,
    base_name: &str,
    selection: SplitSelection,
    sf2_bytes: &[u8],
) -> Result<Vec<SplitFileEntry>, IrrecoverableError> {
    let score = crate::compile_with_preset(source, filename, &[], selection.preset_name)?;
    let track_names = split_track_names(source, filename, &score, selection)?;
    let base_name = selection.file_base_name(base_name);
    let display_names = part_display_name_map(source, filename)?;
    let mut entries = Vec::with_capacity(track_names.len());
    for track in track_names {
        let mut score_clone = score.clone();
        filter_tracks(&mut score_clone, std::slice::from_ref(&track));
        let midi_bytes = crate::midi::write_midi(&score_clone)?;
        let wav = crate::wav::write_wav(&midi_bytes, sf2_bytes, None)?;
        let label = split_track_label(&display_names, &track);
        entries.push(SplitFileEntry {
            track_name: track.clone(),
            filename: split_track_filename(&base_name, &label, "wav"),
            bytes: wav,
        });
    }
    Ok(entries)
}

/// Parse once, synthesize one MP3 file per track.
#[cfg(feature = "mp3")]
pub fn write_split_mp3s_from_source(
    source: &str,
    filename: &str,
    base_name: &str,
    selection: SplitSelection,
    sf2_bytes: &[u8],
) -> Result<Vec<SplitFileEntry>, IrrecoverableError> {
    let score = crate::compile_with_preset(source, filename, &[], selection.preset_name)?;
    let track_names = split_track_names(source, filename, &score, selection)?;
    let base_name = selection.file_base_name(base_name);
    let display_names = part_display_name_map(source, filename)?;
    let mut entries = Vec::with_capacity(track_names.len());
    for track in track_names {
        let mut score_clone = score.clone();
        filter_tracks(&mut score_clone, std::slice::from_ref(&track));
        let midi_bytes = crate::midi::write_midi(&score_clone)?;
        let mp3 = crate::wav::write_mp3(&midi_bytes, sf2_bytes, None)?;
        let label = split_track_label(&display_names, &track);
        entries.push(SplitFileEntry {
            track_name: track.clone(),
            filename: split_track_filename(&base_name, &label, "mp3"),
            bytes: mp3,
        });
    }
    Ok(entries)
}
