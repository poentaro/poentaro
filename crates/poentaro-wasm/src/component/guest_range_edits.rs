//! Bodies for the `Guest` methods that edit or describe the editor's
//! selected byte ranges (octave shift, slur and tie toggles, the pitch
//! description). Kept at the exact by-value parameter types the `Guest`
//! trait requires (see `mod.rs`'s doc comment), so `needless_pass_by_value`
//! is relaxed here.
#![allow(clippy::needless_pass_by_value)]

use super::*;

pub(super) fn shift_range_octave(
    source: String,
    ranges: Vec<ByteRange>,
    delta: i32,
) -> RangeEditResponse {
    edit_ranges(&source, ranges, |source, ranges| {
        poentaro::source_edit::shift_range_octave(source, ranges, delta as i8)
    })
}

pub(super) fn toggle_range_slur(source: String, ranges: Vec<ByteRange>) -> RangeEditResponse {
    edit_ranges(&source, ranges, poentaro::source_edit::toggle_range_slur)
}

pub(super) fn toggle_range_tie(source: String, ranges: Vec<ByteRange>) -> RangeEditResponse {
    edit_ranges(&source, ranges, poentaro::source_edit::toggle_range_tie)
}

/// Runs a range edit with the guest's ranges and result.
fn edit_ranges(
    source: &str,
    ranges: Vec<ByteRange>,
    edit: impl FnOnce(
        &str,
        &[poentaro::source_edit::ByteRange],
    ) -> poentaro::source_edit::RangeEditResult,
) -> RangeEditResponse {
    let result = edit(source, &core_ranges(ranges));
    RangeEditResponse {
        source: result.source,
        ranges: result
            .ranges
            .into_iter()
            .map(|range| ByteRange {
                start_byte: range.start_byte as _,
                end_byte: range.end_byte as _,
            })
            .collect(),
    }
}

pub(super) fn describe_selection(
    source: String,
    ranges: Vec<ByteRange>,
) -> Option<PitchDescription> {
    use poentaro::pitch_description as description;
    description::describe_selection(&source, &core_ranges(ranges)).map(
        |described| match described {
            description::PitchDescription::Note(note) => PitchDescription::Note(NoteDescription {
                letter_name: note.letter_name,
            }),
            description::PitchDescription::Chord(chord) => {
                PitchDescription::Chord(ChordDescription {
                    chord_name: chord.chord_name,
                    tone_names: chord.tone_names,
                    bass_note: chord.bass_note,
                    guitar_diagram_svgs: chord.guitar_diagram_svgs,
                    piano_diagram_svg: chord.piano_diagram_svg,
                })
            }
        },
    )
}

fn core_ranges(ranges: Vec<ByteRange>) -> Vec<poentaro::source_edit::ByteRange> {
    ranges
        .into_iter()
        .map(|range| poentaro::source_edit::ByteRange {
            start_byte: range.start_byte as _,
            end_byte: range.end_byte as _,
        })
        .collect()
}
