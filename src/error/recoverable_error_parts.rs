use crate::error::{DocumentSection, RecoverableError, RecoverableErrorKind, Span};

impl RecoverableError {
    pub fn parts_duplicate_abbreviation(span: Span, abbrev: &str) -> Self {
        Self {
            span,
            kind: RecoverableErrorKind::PartsDuplicateAbbreviation {
                abbrev: abbrev.to_string(),
            },
        }
    }

    pub fn parts_empty_section(span: Span) -> Self {
        Self {
            span,
            kind: RecoverableErrorKind::PartsEmptySection,
        }
    }

    pub fn parts_empty_display_name(span: Span) -> Self {
        Self {
            span,
            kind: RecoverableErrorKind::PartsEmptyDisplayName,
        }
    }

    pub fn parts_empty_abbreviation(span: Span) -> Self {
        Self {
            span,
            kind: RecoverableErrorKind::PartsEmptyAbbreviation,
        }
    }

    pub fn parts_empty_track_name(span: Span) -> Self {
        Self {
            span,
            kind: RecoverableErrorKind::PartsEmptyTrackName,
        }
    }

    pub fn parts_invalid_columns(span: Span, rhs: &str) -> Self {
        Self {
            span,
            kind: RecoverableErrorKind::PartsInvalidColumns {
                rhs: rhs.to_string(),
            },
        }
    }

    pub fn section_duplicate(span: Span, section: DocumentSection) -> Self {
        Self {
            span,
            kind: RecoverableErrorKind::SectionDuplicate { section },
        }
    }

    pub fn section_missing(span: Span, section: DocumentSection) -> Self {
        Self {
            span,
            kind: RecoverableErrorKind::SectionMissing { section },
        }
    }

    pub fn lyrics_line_empty(span: Span) -> Self {
        Self {
            span,
            kind: RecoverableErrorKind::LyricsLineEmpty,
        }
    }

    pub fn lyrics_bracket_not_one_syllable(span: Span, found: usize) -> Self {
        Self {
            span,
            kind: RecoverableErrorKind::LyricsBracketNotOneSyllable { found },
        }
    }

    pub fn lyrics_bracket_count_mismatch(
        span: Span,
        brackets: usize,
        attacks: usize,
        first_syllable_takes_beat_one: bool,
    ) -> Self {
        Self {
            span,
            kind: RecoverableErrorKind::LyricsBracketCountMismatch {
                brackets,
                attacks,
                first_syllable_takes_beat_one,
            },
        }
    }

    pub fn lyrics_bracket_unclosed(span: Span) -> Self {
        Self {
            span,
            kind: RecoverableErrorKind::LyricsBracketUnclosed,
        }
    }

    pub fn lyrics_bracket_unmatched(span: Span) -> Self {
        Self {
            span,
            kind: RecoverableErrorKind::LyricsBracketUnmatched,
        }
    }

    pub fn parts_first_part_cannot_follow(span: Span) -> Self {
        Self {
            span,
            kind: RecoverableErrorKind::PartsFirstPartCannotFollow,
        }
    }

    pub fn parts_unknown_soundfont(span: Span, soundfont: &str, suggestions: Vec<String>) -> Self {
        Self {
            span,
            kind: RecoverableErrorKind::PartsUnknownSoundfont {
                soundfont: soundfont.to_string(),
                suggestions,
            },
        }
    }

    pub fn parts_volume_out_of_range(span: Span, volume: u16) -> Self {
        Self {
            span,
            kind: RecoverableErrorKind::PartsVolumeOutOfRange { volume },
        }
    }

    pub fn parts_octave_offset_out_of_range(span: Span, offset: i16) -> Self {
        Self {
            span,
            kind: RecoverableErrorKind::PartsOctaveOffsetOutOfRange { offset },
        }
    }

    pub fn parts_unknown_instrument(span: Span, name: &str) -> Self {
        Self {
            span,
            kind: RecoverableErrorKind::PartsUnknownInstrument {
                name: name.to_string(),
            },
        }
    }

    pub fn parts_instrument_on_non_notes_part(span: Span) -> Self {
        Self {
            span,
            kind: RecoverableErrorKind::PartsInstrumentOnNonNotesPart,
        }
    }

    pub fn parts_follow_unknown_target(span: Span, target: &str) -> Self {
        Self {
            span,
            kind: RecoverableErrorKind::PartsFollowUnknownTarget {
                target: target.to_string(),
            },
        }
    }

    pub fn parts_lyrics_invalid_target(span: Span, target: &str) -> Self {
        Self {
            span,
            kind: RecoverableErrorKind::PartsLyricsInvalidTarget {
                target: target.to_string(),
            },
        }
    }

    pub fn part_key_unknown(span: Span, key: &str) -> Self {
        Self {
            span,
            kind: RecoverableErrorKind::PartKeyUnknown {
                key: key.to_string(),
            },
        }
    }

    pub fn score_line_missing_key_prefix(span: Span) -> Self {
        Self {
            span,
            kind: RecoverableErrorKind::ScoreLineMissingKeyPrefix,
        }
    }

    pub fn tie_on_rest(span: Span) -> Self {
        Self {
            span,
            kind: RecoverableErrorKind::TieOnRest,
        }
    }

    pub fn repeat_no_prior_note(span: Span) -> Self {
        Self {
            span,
            kind: RecoverableErrorKind::RepeatNoPriorNote,
        }
    }

    pub fn dangling_tie(span: Span) -> Self {
        Self {
            span,
            kind: RecoverableErrorKind::DanglingTie,
        }
    }

    pub fn tie_pitch_mismatch(span: Span, expected: String, got: String) -> Self {
        Self {
            span,
            kind: RecoverableErrorKind::TiePitchMismatch { expected, got },
        }
    }
}
