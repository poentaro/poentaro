use super::kind::IrrecoverableErrorKind;
use crate::error::Span;

pub(super) fn span(kind: &IrrecoverableErrorKind) -> Option<&Span> {
    parse_span(kind).or_else(|| export_span(kind))
}

fn parse_span(_kind: &IrrecoverableErrorKind) -> Option<&Span> {
    None
}

fn export_span(kind: &IrrecoverableErrorKind) -> Option<&Span> {
    match kind {
        IrrecoverableErrorKind::MidiWriteFailed { span }
        | IrrecoverableErrorKind::WavInvalidMidiBytes { span }
        | IrrecoverableErrorKind::WavSynthInitFailed { span }
        | IrrecoverableErrorKind::WavSoundfontLoadFailed { span }
        | IrrecoverableErrorKind::WavWriterCreateFailed { span, .. }
        | IrrecoverableErrorKind::WavWriteSampleFailed { span, .. }
        | IrrecoverableErrorKind::WavFinalizeFailed { span, .. }
        | IrrecoverableErrorKind::Mp3EncodeFailed { span, .. }
        | IrrecoverableErrorKind::PdfSvgParseFailed { span, .. }
        | IrrecoverableErrorKind::PdfSvgConversionFailed { span, .. }
        | IrrecoverableErrorKind::ZipStartFileFailed { span, .. }
        | IrrecoverableErrorKind::ZipWriteFailed { span, .. }
        | IrrecoverableErrorKind::ZipFinishFailed { span, .. }
        | IrrecoverableErrorKind::IoReadFailed { span, .. }
        | IrrecoverableErrorKind::IoWriteFailed { span, .. }
        | IrrecoverableErrorKind::InternalInvariant { span, .. }
        | IrrecoverableErrorKind::AnyxSourceInvalid { span, .. }
        | IrrecoverableErrorKind::UnknownPreset { span, .. } => Some(span),
    }
}
