use crate::ast::grouped::{
    GroupedMeasure, GroupedScore, GroupedTrack, LyricsSlice, MeasureDirectives, MultiPartMeasure,
    Notes, PartRow, PartSlice, DEFAULT_HIDE_RESTING_PARTS,
    DEFAULT_MERGE_DUPLICATE_MEASURES_ACROSS_PARTS,
};
use crate::ast::parsed::PartKind;
use crate::error::{Diagnostic, RecoverableError, Span};

fn collect_part_measure_diagnostics(m: Option<&GroupedMeasure>) -> Vec<Diagnostic> {
    [
        m.and_then(|m| m.beat_overflow_error.clone())
            .map(Diagnostic::Warning),
        m.and_then(|m| m.lex_error.clone()).map(Diagnostic::Error),
        m.and_then(|m| m.lyrics_parse_error.clone())
            .map(Diagnostic::Error),
        m.and_then(|m| m.extension_no_preceding_event_error.clone())
            .map(Diagnostic::Error),
    ]
    .into_iter()
    .flatten()
    .chain(
        m.into_iter()
            .flat_map(|m| m.lyrics_error.iter().cloned())
            .map(Diagnostic::Warning),
    )
    .chain(
        m.into_iter()
            .flat_map(|m| m.dotted_eighth_errors.iter().cloned()),
    )
    .chain(m.into_iter().flat_map(|m| m.chord_errors.iter().cloned()))
    .collect()
}

fn measure_has_error(m: &GroupedMeasure) -> bool {
    m.lex_error.is_some()
        || m.lyrics_parse_error.is_some()
        || m.extension_no_preceding_event_error.is_some()
        || m.dotted_eighth_errors
            .iter()
            .any(|d| matches!(d, Diagnostic::Error(_)))
        || m.chord_errors
            .iter()
            .any(|d| matches!(d, Diagnostic::Error(_)))
}

fn combine_measure(
    grouped_score: &GroupedScore,
    measure_idx: usize,
    directives_fallback: &MeasureDirectives,
) -> MultiPartMeasure {
    let (directives, directives_error) = match grouped_score.measure_directives.get(measure_idx) {
        Some(d) => (d, None),
        None => (
            directives_fallback,
            Some(Diagnostic::Error(
                RecoverableError::measure_directives_missing(Span::new(0, 0)),
            )),
        ),
    };
    let (part_rows, part_row_diagnostics) = build_part_rows(&grouped_score.parts, measure_idx);
    let (source_span, source_span_error) = grouped_score
        .parts
        .iter()
        .filter_map(|track| match track {
            GroupedTrack::Timed(part) => part.measures.get(measure_idx),
        })
        .fold(None, |acc: Option<Span>, m| {
            Some(match acc {
                None => m.source_span,
                Some(prev) => Span::new(
                    prev.start.min(m.source_span.start),
                    prev.end.max(m.source_span.end),
                ),
            })
        })
        .map(|span| (span, None))
        .unwrap_or_else(|| {
            (
                Span::new(0, 0),
                Some(Diagnostic::Error(RecoverableError::source_span_missing(
                    Span::new(0, 0),
                    measure_idx,
                ))),
            )
        });
    let parse_error = grouped_score
        .per_measure_parse_errors
        .get(measure_idx)
        .and_then(|e| e.clone());
    let measure_diagnostics: Vec<Diagnostic> = directives_error
        .into_iter()
        .chain(source_span_error)
        .chain(parse_error)
        .chain(grouped_score.parts.iter().flat_map(|track| match track {
            GroupedTrack::Timed(part) => {
                collect_part_measure_diagnostics(part.measures.get(measure_idx))
            }
        }))
        .chain(part_row_diagnostics)
        .collect();
    MultiPartMeasure {
        time_signature: directives.time_signature.clone(),
        bpm: directives.bpm,
        key: directives.key.clone(),
        label: directives.label.clone(),
        merge_duplicate_measures_across_parts: directives.merge_duplicate_measures_across_parts,
        hide_resting_parts: directives.hide_resting_parts,
        system_break: directives.system_break,
        parts: part_rows,
        source_span,
        diagnostics: measure_diagnostics,
    }
}

pub(crate) fn combine(grouped_score: &GroupedScore) -> (Vec<MultiPartMeasure>, Vec<Diagnostic>) {
    if grouped_score.parts.is_empty() {
        return (Vec::new(), Vec::new());
    }

    let expected_len = grouped_score
        .parts
        .first()
        .map(GroupedTrack::measure_count)
        .unwrap_or(0);
    let max_len = grouped_score
        .parts
        .iter()
        .map(GroupedTrack::measure_count)
        .max()
        .unwrap_or(0);

    let mismatch_diagnostics: Vec<Diagnostic> = grouped_score
        .parts
        .iter()
        .skip(1)
        .filter(|track| track.measure_count() != expected_len)
        .map(|track| {
            let span = match track {
                GroupedTrack::Timed(part) => part
                    .measures
                    .last()
                    .map(|m| Span::new(m.source_span.end, m.source_span.end))
                    .unwrap_or(Span::new(0, 0)),
            };
            Diagnostic::Error(RecoverableError::part_measure_count_mismatch(
                span,
                format!("{:?}", track.track_name()),
                track.measure_count(),
                expected_len,
            ))
        })
        .collect();

    let directives_fallback = MeasureDirectives {
        time_signature: None,
        bpm: None,
        key: None,
        label: None,
        merge_duplicate_measures_across_parts: DEFAULT_MERGE_DUPLICATE_MEASURES_ACROSS_PARTS,
        hide_resting_parts: DEFAULT_HIDE_RESTING_PARTS,
        system_break: false,
    };
    let combined = (0..max_len)
        .map(|measure_idx| combine_measure(grouped_score, measure_idx, &directives_fallback))
        .collect();

    (combined, mismatch_diagnostics)
}

fn build_part_rows(
    grouped_tracks: &[GroupedTrack],
    measure_idx: usize,
) -> (Vec<PartRow>, Vec<Diagnostic>) {
    let mut part_rows = Vec::new();
    let mut diagnostics = Vec::new();

    for track in grouped_tracks.iter() {
        match track {
            GroupedTrack::Timed(part) => {
                let Some(measure) = part.measures.get(measure_idx) else {
                    let span = part
                        .measures
                        .last()
                        .map(|m| Span::new(m.source_span.end, m.source_span.end))
                        .unwrap_or(Span::new(0, 0));
                    diagnostics.push(Diagnostic::Error(
                        RecoverableError::timed_part_measure_missing(span),
                    ));
                    continue;
                };
                let lyrics = match part.kind {
                    PartKind::Lyrics { target_part_index } => {
                        grouped_tracks.get(target_part_index).map(|target| {
                            let GroupedTrack::Timed(target) = target;
                            LyricsSlice {
                                target_name: target.name.clone().unwrap_or_default(),
                                target_events: target
                                    .measures
                                    .get(measure_idx)
                                    .map_or_else(Vec::new, |measure| measure.notes.events.clone()),
                                targets_chords: target.kind == PartKind::Chords,
                                syllables: measure.paired_lyrics.clone(),
                            }
                        })
                    }
                    _ => None,
                };
                let slice = PartSlice {
                    name: part.name.clone(),
                    kind: part.kind,
                    soundfont: part.soundfont,
                    volume: part.volume,
                    octave_offset: part.octave_offset,
                    range_instrument: part.range_instrument,
                    notes: Notes {
                        events: measure.notes.events.clone(),
                    },
                    lyrics,
                    has_error: measure_has_error(measure),
                    resolution_multiplier: measure.resolution_multiplier,
                    beat_group_size: measure.beat_group_size,
                };
                part_rows.push(PartRow::Timed(slice));
            }
        }
    }

    (part_rows, diagnostics)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{grouper, parser};

    fn make_two_part_score(soprano: &str, alto: &str) -> Vec<MultiPartMeasure> {
        let input = format!(
            r#"metadata {{ title "t", author "a" }},

parts ["Soprano": notes, "Alto": notes],

score """
    time=4/4 key=C4 bpm=120
    [Soprano] {soprano}
    [Alto] {alto}
    """,
"#
        );
        let doc = parser::parse(&input, "test.poentaro", &[]).unwrap();
        grouper::group(doc).unwrap().measures
    }

    #[test]
    fn combines_two_parts_into_measures() {
        let measures = make_two_part_score("1 2 3 4", "5 6 7 1");
        assert_eq!(measures.len(), 1);
        assert_eq!(measures[0].parts.len(), 2);
    }

    #[test]
    fn directives_come_from_first_part() {
        let measures = make_two_part_score("1 2 3 4", "5 6 7 1");
        assert_eq!(measures[0].bpm, Some(120));
        assert!(measures[0].time_signature.is_some());
    }

    #[test]
    fn beat_overflow_in_one_part_attaches_error_to_measure() {
        // Alto has 5 notes in a 4/4 bar — overflow is recoverable; the measure
        // gets an error and the 5th note is trimmed.
        let input = r#"metadata { title "t", author "a" },

parts ["Soprano": notes, "Alto": notes],

score """
    time=4/4 key=C4 bpm=120
    [Soprano] 1 2 3 4
    [Alto] 5 6 7 1 5
    """,
"#;
        let doc = parser::parse(input, "test.poentaro", &[])
            .expect("beat overflow must not abort parsing");
        let score = grouper::group(doc).expect("grouping must succeed");
        assert_eq!(score.measures.len(), 1);
        assert_eq!(score.measures[0].diagnostics.len(), 1);
        assert!(
            score.measures[0].diagnostics[0]
                .message()
                .contains("beat overflow"),
            "got: {}",
            score.measures[0].diagnostics[0].message()
        );
    }

    #[test]
    fn measure_source_span_is_nonzero_after_combine() {
        let measures = make_two_part_score("1 2 3 4", "5 6 7 1");
        assert_eq!(measures.len(), 1);
        // span should not be the dummy (0, 0)
        assert!(
            measures[0].source_span.end > 0,
            "source_span.end should be > 0, got {:?}",
            measures[0].source_span
        );
    }
}
