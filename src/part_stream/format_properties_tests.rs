//! Property tests for `format_part_stream`: random token streams, joined by
//! random whitespace, checked against the formatter's invariants.

use itertools::Itertools;
use quickcheck::{QuickCheck, TestResult};

use crate::part_stream::{apply_part_stream, extract_part_stream};

use super::super::stream_generator::GeneratedStream;
use super::{format_part_stream, split_part_stream, FormattedPartStream};

fn apply_edits(stream: &str, formatted: &FormattedPartStream) -> String {
    formatted
        .edits
        .iter()
        .rev()
        .fold(stream.to_string(), |mut text, edit| {
            text.replace_range(edit.span.start..edit.span.end, &edit.replacement);
            text
        })
}

fn without_whitespace(text: &str) -> String {
    text.chars()
        .filter(|character| !character.is_whitespace())
        .collect()
}

/// Whether `formatted` is `text` without whitespace, less any trailing `_`
/// placeholders the formatter deleted.
fn keeps_non_whitespace(text: &str, formatted: &str) -> bool {
    let kept = without_whitespace(formatted);
    without_whitespace(text)
        .strip_prefix(&kept)
        .is_some_and(|deleted| deleted.chars().all(|character| character == '_'))
}

fn check(property: fn(GeneratedStream) -> TestResult) {
    QuickCheck::new()
        .tests(3000)
        .quickcheck(property as fn(GeneratedStream) -> TestResult);
}

/// Fails with `message` and the stream, so a shrunk failure reads directly.
fn failure(stream: &GeneratedStream, message: &str) -> TestResult {
    TestResult::error(format!(
        "{message}\n  part: {}\n  stream: {:?}",
        stream.fixture().abbreviation,
        stream.text()
    ))
}

#[test]
fn edits_are_sorted_non_overlapping_on_char_boundaries_and_only_touch_whitespace_or_placeholders() {
    check(|stream| {
        let text = stream.text();
        let formatted = stream.formatted();
        let in_order = formatted
            .edits
            .iter()
            .tuple_windows()
            .all(|(before, after)| before.span.end <= after.span.start);
        let well_formed = formatted.edits.iter().all(|edit| {
            let replaced = &text[edit.span.start..edit.span.end];
            let deletes_placeholders = edit.replacement.is_empty()
                && replaced
                    .chars()
                    .all(|character| character.is_whitespace() || character == '_');
            edit.span.start <= edit.span.end
                && text.is_char_boundary(edit.span.start)
                && text.is_char_boundary(edit.span.end)
                && (deletes_placeholders
                    || (replaced.chars().all(char::is_whitespace)
                        && edit.replacement.chars().all(char::is_whitespace)))
        });
        if in_order && well_formed {
            TestResult::passed()
        } else {
            failure(&stream, &format!("malformed edits: {:?}", formatted.edits))
        }
    });
}

#[test]
fn formatting_changes_only_whitespace_and_trailing_placeholders() {
    check(|stream| {
        let text = stream.text();
        let formatted_text = apply_edits(&text, &stream.formatted());
        if keeps_non_whitespace(&text, &formatted_text) {
            TestResult::passed()
        } else {
            failure(&stream, &format!("formatted to {formatted_text:?}"))
        }
    });
}

#[test]
fn formatting_is_idempotent() {
    check(|stream| {
        let fixture = stream.fixture();
        let once = apply_edits(&stream.text(), &stream.formatted());
        let again = format_part_stream(fixture.source, fixture.abbreviation, &once, &[]);
        if again.edits.is_empty() {
            TestResult::passed()
        } else {
            failure(
                &stream,
                &format!(
                    "formatted to {once:?}, which formats again with {:?}",
                    again.edits
                ),
            )
        }
    });
}

#[test]
fn formatting_does_not_change_what_applies() {
    check(|stream| {
        let fixture = stream.fixture();
        let text = stream.text();
        let formatted_text = apply_edits(&text, &stream.formatted());
        let before = apply_part_stream(fixture.source, fixture.abbreviation, &text);
        let after = apply_part_stream(fixture.source, fixture.abbreviation, &formatted_text);
        let messages = |diagnostics: &[super::super::StreamDiagnostic]| {
            diagnostics
                .iter()
                .map(|diagnostic| diagnostic.message.clone())
                .collect_vec()
        };
        if before.source == after.source
            && before.changed_measure_count == after.changed_measure_count
            && messages(&before.diagnostics) == messages(&after.diagnostics)
        {
            TestResult::passed()
        } else {
            failure(
                &stream,
                &format!("formatted to {formatted_text:?}; before: {before:?}; after: {after:?}"),
            )
        }
    });
}

#[test]
fn a_stream_with_an_error_re_barring_cant_cause_is_left_as_typed() {
    check(|stream| {
        let fixture = stream.fixture();
        let blocked = split_part_stream(fixture.source, fixture.abbreviation, &stream.text())
            .is_none_or(|split| split.blocks_rebarring);
        if !blocked {
            return TestResult::discard();
        }
        let formatted = stream.formatted();
        if formatted.edits.is_empty() {
            TestResult::passed()
        } else {
            failure(
                &stream,
                &format!("edited despite errors: {:?}", formatted.edits),
            )
        }
    });
}

#[test]
fn every_line_of_a_formatted_stream_is_one_measure() {
    check(|stream| {
        let fixture = stream.fixture();
        let formatted_text = apply_edits(&stream.text(), &stream.formatted());
        let applied = apply_part_stream(fixture.source, fixture.abbreviation, &formatted_text);
        if applied.source.is_none() {
            return TestResult::discard();
        }
        // Trailing whitespace is left as typed, so lines inside it don't count.
        let content_end = formatted_text.trim_end().len();
        let line_starts = [0]
            .into_iter()
            .chain(
                formatted_text
                    .match_indices('\n')
                    .map(|(offset, _)| offset + 1),
            )
            .filter(|&start| start < content_end)
            .collect_vec();
        let measure_starts = applied
            .measure_starts
            .iter()
            .copied()
            .filter(|&start| start < content_end)
            .dedup()
            .collect_vec();
        if line_starts == measure_starts {
            TestResult::passed()
        } else {
            failure(
                &stream,
                &format!(
                    "formatted to {formatted_text:?}: lines start at {line_starts:?}, measures at {measure_starts:?}"
                ),
            )
        }
    });
}

#[test]
fn a_cursor_keeps_its_place_among_the_non_whitespace() {
    check(|stream| {
        let text = stream.text();
        let formatted = stream.formatted();
        let formatted_text = apply_edits(&text, &formatted);
        let cursors = stream.cursors();
        // Deleted placeholders are the stream's last non-whitespace, so a
        // cursor among them counts only what's kept.
        let kept_count = without_whitespace(&formatted_text).len();
        let non_whitespace_before =
            |text: &str, offset: usize| without_whitespace(&text[..offset]).len().min(kept_count);
        let all_kept = cursors
            .iter()
            .zip(&formatted.cursors)
            .all(|(before, after)| {
                after.offset <= formatted_text.len()
                    && formatted_text.is_char_boundary(after.offset)
                    && non_whitespace_before(&text, before.offset)
                        == non_whitespace_before(&formatted_text, after.offset)
            });
        let in_order = cursors
            .iter()
            .zip(&formatted.cursors)
            .tuple_combinations()
            .all(|((before_a, after_a), (before_b, after_b))| {
                // Cursors in one whitespace run may meet, but never cross.
                (before_a.offset > before_b.offset || after_a.offset <= after_b.offset)
                    && (before_b.offset > before_a.offset || after_b.offset <= after_a.offset)
            });
        if formatted.cursors.len() == cursors.len() && all_kept && in_order {
            TestResult::passed()
        } else {
            failure(
                &stream,
                &format!(
                    "formatted to {formatted_text:?}; cursors {cursors:?} mapped to {:?}",
                    formatted.cursors
                ),
            )
        }
    });
}

#[test]
fn an_applied_stream_extracts_already_formatted() {
    check(|stream| {
        let fixture = stream.fixture();
        let formatted_text = apply_edits(&stream.text(), &stream.formatted());
        let applied = apply_part_stream(fixture.source, fixture.abbreviation, &formatted_text);
        let Some(new_source) = applied.source else {
            return TestResult::discard();
        };
        let Ok(extracted) = extract_part_stream(&new_source, fixture.abbreviation) else {
            return TestResult::discard();
        };
        let again = format_part_stream(&new_source, fixture.abbreviation, &extracted.text, &[]);
        if again.edits.is_empty() {
            TestResult::passed()
        } else {
            failure(
                &stream,
                &format!(
                    "applied, the source extracts {:?}, which formats with {:?}",
                    extracted.text, again.edits
                ),
            )
        }
    });
}
