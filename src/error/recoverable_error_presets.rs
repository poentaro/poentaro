use crate::error::{RecoverableError, RecoverableErrorKind, Span};

impl RecoverableError {
    pub fn preset_duplicate_name(span: Span, name: &str) -> Self {
        Self {
            span,
            kind: RecoverableErrorKind::PresetDuplicateName {
                name: name.to_string(),
            },
        }
    }

    pub fn preset_reserved_name(span: Span, name: &str) -> Self {
        Self {
            span,
            kind: RecoverableErrorKind::PresetReservedName {
                name: name.to_string(),
            },
        }
    }

    pub fn preset_unknown_part(span: Span, abbreviation: &str) -> Self {
        Self {
            span,
            kind: RecoverableErrorKind::PresetUnknownPart {
                abbreviation: abbreviation.to_string(),
            },
        }
    }

    pub fn preset_only_field(span: Span, field: &str) -> Self {
        Self {
            span,
            kind: RecoverableErrorKind::PresetOnlyField {
                field: field.to_string(),
            },
        }
    }
}
