use super::beam::{flush_beam_buffer, BeamEntry};
use super::part_slice_unit::{compile_unit, CompiledUnit, PartState};
use super::slur_chains::{extend_note_chains, PendingSlurOpen, SlurChainContext, SlurKey};
use super::timed_unit::TimedUnit;
use super::tuplet_spans::{finish_tuplet_spans, record_tuplet_tag, TupletSpanContext};
use super::PartSliceResult;
use crate::ast::grouped::{GroupedRest, LyricsSlice, NoteEvent, PartSlice};
use crate::chord_lyric_placement::place_chord_syllables;
use crate::compiler::types::{ArcKind, ColumnElement, ElementContent, SlurSpan, TupletSpan};
use crate::lyric_slots::measure_lyric_slots;
use crate::pinyin::PinyinSyllable;

// ── Top-level entry point ─────────────────────────────────────────────────────

pub(super) struct PartSliceInput {
    pub(super) pending_opens: Vec<Option<PendingSlurOpen>>,
    pub(super) prev_tie: bool,
    pub(super) prev_tie_column: Option<u32>,
    pub(super) prev_tie_measure: Option<usize>,
    pub(super) prev_tie_note_id: Option<usize>,
    pub(super) next_note_id: usize,
    pub(super) measure_index: usize,
    pub(super) part_index: usize,
}

pub(super) fn compile_part_slice(
    slice: &PartSlice,
    input: PartSliceInput,
    slur_spans: &mut Vec<SlurSpan>,
    tuplet_spans: &mut Vec<TupletSpan>,
) -> PartSliceResult {
    if let Some(lyrics) = &slice.lyrics {
        return compile_lyrics_slice(lyrics, &input);
    }
    let mut elements: Vec<ColumnElement> = Vec::new();
    let mut beam_buf: Vec<BeamEntry> = Vec::new();
    let mut pending_chains: Vec<Vec<(u32, SlurKey)>> = Vec::new();
    let mut pending_slur_opens: Vec<Option<PendingSlurOpen>> = input.pending_opens;
    let mut prev_tie = input.prev_tie;
    let mut prev_tie_column = input.prev_tie_column;
    let mut prev_tie_measure = input.prev_tie_measure;
    let mut prev_tie_note_id = input.prev_tie_note_id;
    let mut next_note_id = input.next_note_id;
    let mut col: u32 = 0;
    let measure_index = input.measure_index;
    let mut current_tuplet = None;

    {
        let mut state = PartState {
            elements: &mut elements,
            beam_buf: &mut beam_buf,
            pending_chains: &mut pending_chains,
            pending_slur_opens: &mut pending_slur_opens,
            slur_spans,
            current_tuplet: &mut current_tuplet,
            tuplet_spans,
            col: &mut col,
            prev_tie: &mut prev_tie,
            prev_tie_column: &mut prev_tie_column,
            prev_tie_measure: &mut prev_tie_measure,
            prev_tie_note_id: &mut prev_tie_note_id,
            next_note_id: &mut next_note_id,
            measure_index,
            part_index: input.part_index,
            multiplier: slice.resolution_multiplier,
            beat_group_size: slice.beat_group_size,
        };
        process_events(&mut state, slice);
        finish_tuplet_spans(&mut TupletSpanContext {
            current: state.current_tuplet,
            tuplet_spans: state.tuplet_spans,
            measure_index: state.measure_index,
            part_index: state.part_index,
        });
    }

    preserve_cross_measure_slur_opens(&pending_chains, &mut pending_slur_opens, measure_index);

    elements.push(ColumnElement {
        column: col,
        content: ElementContent::BarLine,
        note_id: None,
    });

    PartSliceResult {
        elements,
        final_pending_opens: pending_slur_opens,
        final_tie: prev_tie,
        final_tie_column: prev_tie_column,
        final_tie_measure: prev_tie_measure,
        final_tie_note_id: prev_tie_note_id,
        final_next_note_id: next_note_id,
    }
}

/// Compiles a lyric part's row: a `Lyric` element at the column of each of its
/// target's notes that takes a syllable, then the bar line. It has no arcs,
/// beams or tuplets of its own, and its note ids count the target's events the
/// same way the target's own compile does, so `(part, note_id)` of a syllable
/// names the note it is sung on.
fn compile_lyrics_slice(lyrics: &LyricsSlice, input: &PartSliceInput) -> PartSliceResult {
    let measure_slots = measure_lyric_slots(&lyrics.target_events, lyrics.entering_tied);
    let syllable_elements: Vec<ColumnElement> = if lyrics.targets_chords {
        // Under a chord part the line places itself: brackets at the chord
        // attacks, other syllables spread between (see `chord_lyric_placement`).
        place_chord_syllables(&lyrics.target_events, &lyrics.syllables)
            .into_iter()
            .filter_map(|placed| {
                let syllable = lyrics.syllables.get(placed.syllable_index)?;
                Some(ColumnElement {
                    column: placed.column,
                    content: ElementContent::Lyric {
                        text: if syllable.bracketed {
                            format!("({})", syllable.text)
                        } else {
                            syllable.text.clone()
                        },
                        ruby: syllable.pinyin.as_ref().map(PinyinSyllable::marked),
                        note_id: placed.note_id(input.next_note_id),
                    },
                    note_id: None,
                })
            })
            .collect()
    } else {
        measure_slots
            .slots
            .iter()
            .zip(&lyrics.syllables)
            .map(|(slot, syllable)| ColumnElement {
                column: slot.column,
                content: ElementContent::Lyric {
                    text: syllable.text.clone(),
                    ruby: syllable.pinyin.as_ref().map(PinyinSyllable::marked),
                    note_id: input.next_note_id + slot.event_index,
                },
                note_id: None,
            })
            .collect()
    };
    let elements = syllable_elements
        .into_iter()
        .chain(std::iter::once(ColumnElement {
            column: measure_slots.end_column,
            content: ElementContent::BarLine,
            note_id: None,
        }))
        .collect();
    PartSliceResult {
        elements,
        final_pending_opens: Vec::new(),
        final_tie: measure_slots.exits_tied,
        final_tie_column: None,
        final_tie_measure: None,
        final_tie_note_id: None,
        final_next_note_id: input.next_note_id + lyrics.target_events.len(),
    }
}

fn process_events(state: &mut PartState<'_>, slice: &PartSlice) {
    for event in &slice.notes.events {
        let note_id = *state.next_note_id;
        *state.next_note_id += 1;
        match event {
            NoteEvent::Note(note) => compile_timed_unit(state, note, 0, note_id),
            NoteEvent::Rest(rest) => compile_rest(state, rest, 0, note_id),
            NoteEvent::Chord(chord) => compile_timed_unit(state, chord, 0, note_id),
            NoteEvent::Percussion(hit) => compile_timed_unit(state, hit, 0, note_id),
        }
    }
    flush_beam_buffer(state.beam_buf, state.elements);
}

fn preserve_cross_measure_slur_opens(
    pending_chains: &[Vec<(u32, SlurKey)>],
    pending_slur_opens: &mut Vec<Option<PendingSlurOpen>>,
    measure_index: usize,
) {
    for (depth, chain) in pending_chains.iter().enumerate() {
        if chain.len() > 1 {
            let origin = pending_slur_opens
                .get(depth)
                .and_then(|o| o.as_ref())
                .map(|o| (o.measure_index, o.from_column))
                .or_else(|| chain.first().map(|(column, _)| (measure_index, *column)));
            while pending_slur_opens.len() <= depth {
                pending_slur_opens.push(None);
            }
            if let (Some(origin), Some(slot)) = (origin, pending_slur_opens.get_mut(depth)) {
                *slot = Some(PendingSlurOpen {
                    measure_index: origin.0,
                    from_column: origin.1,
                });
            }
        } else if let Some((chain_col, _)) = chain.first() {
            while pending_slur_opens.len() <= depth {
                pending_slur_opens.push(None);
            }
            if pending_slur_opens
                .get(depth)
                .and_then(|o| o.as_ref())
                .is_none()
            {
                if let Some(slot) = pending_slur_opens.get_mut(depth) {
                    *slot = Some(PendingSlurOpen {
                        measure_index,
                        from_column: *chain_col,
                    });
                }
            }
        }
    }
}

fn compile_timed_unit<T: TimedUnit>(
    state: &mut PartState<'_>,
    unit: &T,
    measure_col_start: u32,
    note_id: usize,
) {
    let is_tie_continuation = *state.prev_tie;
    // A tie continuation is the same sounding note as the one it continues
    // from, so it reuses that note's id rather than allocating a fresh one —
    // this mirrors the MIDI side merging tied notes into a single NoteOn/NoteOff.
    let note_id = if is_tie_continuation {
        state.prev_tie_note_id.unwrap_or(note_id)
    } else {
        note_id
    };
    if is_tie_continuation {
        if let (Some(from_col), Some(from_measure)) =
            (*state.prev_tie_column, *state.prev_tie_measure)
        {
            state.slur_spans.push(SlurSpan {
                kind: ArcKind::Tie,
                part_index: state.part_index,
                from_measure,
                from_column: from_col,
                to_measure: state.measure_index,
                to_column: *state.col,
            });
        }
        *state.prev_tie = false;
        *state.prev_tie_column = None;
        *state.prev_tie_measure = None;
        *state.prev_tie_note_id = None;
    }

    let event_col = *state.col;
    compile_unit(
        state,
        CompiledUnit {
            duration: unit.duration(),
            dotted: unit.dotted(),
            double_dotted: unit.double_dotted(),
            group_membership: unit.group_membership(),
            group_continuation: unit.group_continuation(),
            slur_close_at: unit.slur_close_at(),
            slur_key: unit.slur_key(),
            tuplet: unit.tuplet(),
            head: unit.element_content(),
        },
        measure_col_start,
        note_id,
    );

    if unit.tie_to_next() {
        *state.prev_tie = true;
        *state.prev_tie_column = Some(event_col);
        *state.prev_tie_measure = Some(state.measure_index);
        *state.prev_tie_note_id = Some(note_id);
    } else {
        *state.prev_tie = false;
        *state.prev_tie_column = None;
        *state.prev_tie_measure = None;
        *state.prev_tie_note_id = None;
    }
}

fn compile_rest(
    state: &mut PartState<'_>,
    rest: &GroupedRest,
    measure_col_start: u32,
    note_id: usize,
) {
    let multiplier = state.multiplier;
    let underline_count = if rest.duration == multiplier {
        2
    } else if rest.duration == 2 * multiplier {
        1
    } else {
        0
    };

    if underline_count == 0 {
        flush_beam_buffer(state.beam_buf, state.elements);
    }

    record_tuplet_tag(
        &mut TupletSpanContext {
            current: state.current_tuplet,
            tuplet_spans: state.tuplet_spans,
            measure_index: state.measure_index,
            part_index: state.part_index,
        },
        *state.col,
        rest.tuplet,
    );

    state.elements.push(ColumnElement {
        column: *state.col,
        content: ElementContent::Rest {
            dotted: rest.dotted,
            double_dotted: rest.double_dotted,
            implicit_fill: rest.implicit_fill,
        },
        note_id: Some(note_id),
    });

    if rest.group_membership > 0 {
        extend_note_chains(
            SlurChainContext {
                chains: state.pending_chains,
                pending_slur_opens: state.pending_slur_opens,
                slur_spans: state.slur_spans,
                measure_index: state.measure_index,
                part_index: state.part_index,
            },
            rest.group_membership,
            rest.group_continuation,
            *state.col,
            &SlurKey::Rest,
        );
    }

    if underline_count > 0 {
        state.beam_buf.push(BeamEntry {
            column: *state.col,
            underline_count,
            duration: rest.duration,
        });
    }

    // See the matching comment in `part_slice_unit::compile_unit`: a dotted (or
    // double-dotted) rest's own written duration is one full dotted beat, so its
    // `-.`/`-..` extensions land on 6-/7-quarter-beat boundaries rather than the
    // 4-quarter-beat ones a plain `-` uses.
    let beat = if rest.double_dotted {
        7 * multiplier
    } else if rest.dotted {
        6 * multiplier
    } else {
        4 * multiplier
    };
    let rest_col = *state.col;
    for dash_col in (rest_col + beat..rest_col + rest.duration).step_by(beat as usize) {
        state.elements.push(ColumnElement {
            column: dash_col,
            content: ElementContent::NoteDash {
                dotted: rest.dotted,
                double_dotted: rest.double_dotted,
            },
            note_id: Some(note_id),
        });
    }

    *state.col += rest.duration;
    *state.prev_tie = false;
    *state.prev_tie_column = None;
    *state.prev_tie_measure = None;
    *state.prev_tie_note_id = None;

    let beat_position = *state.col - measure_col_start;
    if underline_count > 0 && beat_position % (state.beat_group_size * multiplier) == 0 {
        flush_beam_buffer(state.beam_buf, state.elements);
    }
}
