use std::path::PathBuf;

use super::Span;

#[derive(Debug, Clone)]
pub enum IrrecoverableErrorKind {
    MidiWriteFailed {
        span: Span,
    },
    WavInvalidMidiBytes {
        span: Span,
    },
    WavSynthInitFailed {
        span: Span,
    },
    WavSoundfontLoadFailed {
        span: Span,
    },
    WavWriterCreateFailed {
        span: Span,
        source: String,
    },
    WavWriteSampleFailed {
        span: Span,
        source: String,
    },
    WavFinalizeFailed {
        span: Span,
        source: String,
    },
    Mp3EncodeFailed {
        span: Span,
        source: String,
    },
    PdfSvgParseFailed {
        span: Span,
        detail: String,
    },
    PdfSvgConversionFailed {
        span: Span,
        detail: String,
    },
    ZipStartFileFailed {
        span: Span,
        source: String,
    },
    ZipWriteFailed {
        span: Span,
        source: String,
    },
    ZipFinishFailed {
        span: Span,
        source: String,
    },
    IoReadFailed {
        span: Span,
        path: PathBuf,
        source: String,
    },
    IoWriteFailed {
        span: Span,
        path: PathBuf,
        source: String,
    },
    InternalInvariant {
        span: Span,
        detail: String,
    },
    /// An `.anyfix` document that can't be read.
    /// `report` lists every problem, one `path:line:column: message` per line.
    AnyfixSourceInvalid {
        span: Span,
        report: String,
    },
    /// The CLI's `--preset` names no `preset` section in the file.
    UnknownPreset {
        span: Span,
        name: String,
    },
}

impl IrrecoverableErrorKind {
    pub fn span(&self) -> Option<&Span> {
        super::span::span(self)
    }

    pub fn internal_invariant(span: Span, detail: impl Into<String>) -> Self {
        Self::InternalInvariant {
            span,
            detail: detail.into(),
        }
    }
}
