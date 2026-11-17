use crate::ast::grouped::GroupedChordNote;
use crate::ast::parsed::{Extension, NumberedPitch, TriadQuality};
use crate::compiler::types::{ArcKind, SlurSpan};

pub(super) struct PendingSlurOpen {
    pub(super) measure_index: usize,
    pub(super) from_column: u32,
}

/// Per-part state carried across measure boundaries.
pub(super) struct PartCrossState {
    pub(super) pending_slur_opens: Vec<Option<PendingSlurOpen>>,
    pub(super) prev_tie: bool,
    pub(super) prev_tie_column: Option<u32>,
    pub(super) prev_tie_measure: Option<usize>,
    /// `note_id` of the note a pending tie continues from, so the
    /// continuation reuses it instead of allocating a fresh id.
    pub(super) prev_tie_note_id: Option<usize>,
    /// Next `note_id` to assign for this part; a running counter across the
    /// whole score, never reset per measure (see `ColumnElement::note_id`).
    pub(super) next_note_id: usize,
}

impl PartCrossState {
    pub(super) fn new() -> Self {
        PartCrossState {
            pending_slur_opens: Vec::new(),
            prev_tie: false,
            prev_tie_column: None,
            prev_tie_measure: None,
            prev_tie_note_id: None,
            next_note_id: 0,
        }
    }

    pub(super) fn clone_pending_opens(&self) -> Vec<Option<PendingSlurOpen>> {
        self.pending_slur_opens
            .iter()
            .map(|opt| {
                opt.as_ref().map(|o| PendingSlurOpen {
                    measure_index: o.measure_index,
                    from_column: o.from_column,
                })
            })
            .collect()
    }
}

#[derive(Clone, PartialEq)]
pub(super) enum SlurKey {
    Pitch(NumberedPitch),
    Chord {
        degree: NumberedPitch,
        triad: TriadQuality,
        extension: Option<Extension>,
        bass_degree: Option<NumberedPitch>,
    },
    Rest,
}

impl SlurKey {
    pub(super) fn from_chord(chord: &GroupedChordNote) -> Self {
        SlurKey::Chord {
            degree: chord.degree.clone(),
            triad: chord.triad.clone(),
            extension: chord.extension.clone(),
            bass_degree: chord.bass.as_ref().map(|b| b.degree.clone()),
        }
    }
}

/// Emit a single `SlurSpan` covering the full chain from first to last node.
///
/// `pending_open`: if `Some`, the chain started in a previous measure; use it as the origin
/// instead of `chain.first()`. Passing `None` treats the whole chain as same-measure.
pub(super) fn flush_chain(
    chain: &[(u32, SlurKey)],
    pending_open: Option<&PendingSlurOpen>,
    slur_spans: &mut Vec<SlurSpan>,
    measure_index: usize,
    part_index: usize,
) {
    if chain.len() <= 1 {
        return;
    }

    if let Some((first, last)) = chain.first().zip(chain.last()) {
        let (from_measure, from_column) = pending_open
            .map(|o| (o.measure_index, o.from_column))
            .unwrap_or((measure_index, first.0));
        slur_spans.push(SlurSpan {
            kind: ArcKind::Slur,
            part_index,
            from_measure,
            from_column,
            to_measure: measure_index,
            to_column: last.0,
        });
    }
}

pub(super) struct SlurChainContext<'a> {
    pub(super) chains: &'a mut Vec<Vec<(u32, SlurKey)>>,
    pub(super) pending_slur_opens: &'a mut Vec<Option<PendingSlurOpen>>,
    pub(super) slur_spans: &'a mut Vec<SlurSpan>,
    pub(super) measure_index: usize,
    pub(super) part_index: usize,
}

pub(super) fn extend_note_chains(
    context: SlurChainContext<'_>,
    membership: u8,
    continuation: u8,
    col: u32,
    key: &SlurKey,
) {
    let SlurChainContext {
        chains,
        pending_slur_opens,
        slur_spans,
        measure_index,
        part_index,
    } = context;
    while chains.len() < membership as usize {
        chains.push(Vec::new());
    }
    for chain in chains.iter_mut().take(membership as usize) {
        chain.push((col, key.clone()));
    }
    for depth in (continuation as usize)..(membership as usize) {
        if let Some(chain) = chains.get(depth) {
            if chain.len() > 1 {
                let pending_open = pending_slur_opens.get_mut(depth).and_then(|o| o.take());
                flush_chain(
                    chain,
                    pending_open.as_ref(),
                    slur_spans,
                    measure_index,
                    part_index,
                );
            } else if chain.len() == 1 {
                // Cross-measure close: origin is in pending_slur_opens[depth]
                if let Some(open) = pending_slur_opens.get_mut(depth).and_then(|o| o.take()) {
                    slur_spans.push(SlurSpan {
                        kind: ArcKind::Slur,
                        part_index,
                        from_measure: open.measure_index,
                        from_column: open.from_column,
                        to_measure: measure_index,
                        to_column: col,
                    });
                }
            }
        }
        if let Some(chain) = chains.get_mut(depth) {
            chain.clear();
        }
    }
}
