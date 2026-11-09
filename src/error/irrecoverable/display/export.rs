use crate::error::irrecoverable::kind::IrrecoverableErrorKind;
use std::fmt;

pub(super) fn write(
    kind: &IrrecoverableErrorKind,
    formatter: &mut fmt::Formatter<'_>,
) -> Option<fmt::Result> {
    match kind {
        IrrecoverableErrorKind::MidiWriteFailed { .. } => {
            Some(write!(formatter, "failed to write MIDI data"))
        }
        IrrecoverableErrorKind::WavInvalidMidiBytes { .. } => {
            Some(write!(formatter, "invalid MIDI bytes"))
        }
        IrrecoverableErrorKind::WavSynthInitFailed { .. } => {
            Some(write!(formatter, "failed to initialize synthesizer"))
        }
        IrrecoverableErrorKind::WavSoundfontLoadFailed { .. } => {
            Some(write!(formatter, "failed to load soundfont"))
        }
        IrrecoverableErrorKind::WavWriterCreateFailed { source, .. } => {
            Some(write!(formatter, "failed to create WAV writer: {source}"))
        }
        IrrecoverableErrorKind::WavWriteSampleFailed { source, .. } => {
            Some(write!(formatter, "failed to write WAV sample: {source}"))
        }
        IrrecoverableErrorKind::WavFinalizeFailed { source, .. } => {
            Some(write!(formatter, "failed to finalize WAV file: {source}"))
        }
        IrrecoverableErrorKind::Mp3EncodeFailed { source, .. } => {
            Some(write!(formatter, "failed to encode MP3: {source}"))
        }
        IrrecoverableErrorKind::PdfSvgParseFailed { detail, .. } => {
            Some(write!(formatter, "SVG parse error: {detail}"))
        }
        IrrecoverableErrorKind::PdfSvgConversionFailed { detail, .. } => {
            Some(write!(formatter, "SVG to PDF chunk failed: {detail}"))
        }
        IrrecoverableErrorKind::ZipStartFileFailed { source, .. } => {
            Some(write!(formatter, "zip start_file: {source}"))
        }
        IrrecoverableErrorKind::ZipWriteFailed { source, .. } => {
            Some(write!(formatter, "zip write: {source}"))
        }
        IrrecoverableErrorKind::ZipFinishFailed { source, .. } => {
            Some(write!(formatter, "zip finish: {source}"))
        }
        IrrecoverableErrorKind::IoReadFailed { path, source, .. } => {
            Some(write!(formatter, "could not read {path:?}: {source}"))
        }
        IrrecoverableErrorKind::IoWriteFailed { path, source, .. } => {
            Some(write!(formatter, "could not write {path:?}: {source}"))
        }
        IrrecoverableErrorKind::InternalInvariant { detail, .. } => {
            Some(write!(formatter, "{detail}"))
        }
        IrrecoverableErrorKind::AnyfixSourceInvalid { report, .. } => {
            Some(write!(formatter, "{report}"))
        }
        IrrecoverableErrorKind::UnknownPreset { name, .. } => {
            Some(write!(formatter, "no preset \"{name}\" in this file"))
        }
    }
}
