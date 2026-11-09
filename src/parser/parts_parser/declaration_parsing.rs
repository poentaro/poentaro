use crate::ast::parsed::{PartKind, RangeInstrument};
use crate::error::{RecoverableError, Span, Spanned};

use super::{RawKind, PART_SETTING_LIMITS};

/// `range_instrument`, or `None` with a diagnostic when `kind` is a part
/// that can't name one (only `notes` and `follow[X]` parts can).
pub(crate) fn checked_range_instrument(
    range_instrument: Option<Spanned<RangeInstrument>>,
    kind: &RawKind,
    errors: &mut Vec<RecoverableError>,
) -> Option<RangeInstrument> {
    let range_instrument = range_instrument?;
    match kind {
        RawKind::Concrete(PartKind::Notes) | RawKind::Follow { .. } => Some(range_instrument.value),
        _ => {
            errors.push(RecoverableError::parts_instrument_on_non_notes_part(
                range_instrument.span,
            ));
            None
        }
    }
}

/// `written` clamped into [`PART_SETTING_LIMITS`]`.volume`, reporting a
/// diagnostic at `span` when it had to move.
pub(crate) fn checked_volume(written: u16, span: Span, errors: &mut Vec<RecoverableError>) -> u8 {
    let clamped = PART_SETTING_LIMITS.volume.clamp(i32::from(written));
    if clamped.was_out_of_range {
        errors.push(RecoverableError::parts_volume_out_of_range(span, written));
    }
    clamped.value
}

/// `written` clamped into [`PART_SETTING_LIMITS`]`.octave_offset`, reporting
/// a diagnostic at `span` when it had to move.
pub(crate) fn checked_octave_offset(
    written: i16,
    span: Span,
    errors: &mut Vec<RecoverableError>,
) -> i8 {
    let clamped = PART_SETTING_LIMITS.octave_offset.clamp(i32::from(written));
    if clamped.was_out_of_range {
        errors.push(RecoverableError::parts_octave_offset_out_of_range(
            span, written,
        ));
    }
    clamped.value
}
