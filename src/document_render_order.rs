use crate::ast::grouped::Score;
use crate::error::IrrecoverableError;
use crate::grid_layout::MeasureRange;
use crate::midi::ExpandedMeasureOrigin;
use crate::renderer::new_types::{SvgElement, SvgKind, Tag};

/// The order in which the preview lays out the score.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum ScoreOrder {
    #[default]
    Written,
    Expanded,
}

pub(super) struct ExpandedScore {
    pub(super) score: Score,
    pub(super) origins: Vec<ExpandedMeasureOrigin>,
}

impl ScoreOrder {
    pub(super) fn expand(
        self,
        written: &Score,
    ) -> Result<Option<ExpandedScore>, IrrecoverableError> {
        if self == Self::Written || written.sequence.is_none() {
            return Ok(None);
        }
        let (mut score, mut origins) = crate::midi::expand_navigation_with_note_positions(written)?;
        let mut written_time = None;
        let written_times: Vec<_> = written
            .measures
            .iter()
            .map(|measure| {
                if let Some(time) = &measure.time_signature {
                    written_time = Some(time.clone());
                }
                written_time.clone()
            })
            .collect();
        let (mut time, mut bpm, mut key) = (None, None, None);
        for (measure, origin) in score.measures.iter_mut().zip(&mut origins) {
            let Some(source) = written.measures.get(origin.written_measure_index) else {
                continue;
            };
            // The selected alternative already occupies the base part's row.
            (measure.parts, origin.part_written_indices) = std::mem::take(&mut measure.parts)
                .into_iter()
                .zip(std::mem::take(&mut origin.part_written_indices))
                .filter(|(row, _)| !row.is_alternative_line())
                .unzip();
            let next_time = written_times
                .get(origin.written_measure_index)
                .cloned()
                .flatten();
            if measure.time_signature.is_none() && time != next_time {
                measure.time_signature = next_time.clone();
            }
            time = next_time;
            if source.bpm.is_none() && measure.bpm == bpm {
                measure.bpm = None;
            } else if measure.bpm.is_some() {
                bpm = measure.bpm;
            }
            if source.key.is_none() && measure.key == key {
                measure.key = None;
            } else if measure.key.is_some() {
                key.clone_from(&measure.key);
            }
            measure.system_break |= measure.label.is_some();
        }
        Ok(Some(ExpandedScore { score, origins }))
    }
}

impl ExpandedScore {
    fn written_index(&self, expanded_index: usize) -> usize {
        self.origins
            .get(expanded_index)
            .map_or(expanded_index, |origin| origin.written_measure_index)
    }

    pub(super) fn highlight_ranges(&self, ranges: &[MeasureRange]) -> Vec<MeasureRange> {
        use itertools::Itertools;

        self.origins
            .iter()
            .enumerate()
            .chunk_by(|(_, origin)| {
                let index = origin.written_measure_index;
                ranges
                    .iter()
                    .any(|range| (range.start..=range.end).contains(&index))
            })
            .into_iter()
            .filter_map(|(selected, mut group)| {
                if !selected {
                    return None;
                }
                let (start, _) = group.next()?;
                let end = group.last().map_or(start, |(index, _)| index);
                Some(MeasureRange { start, end })
            })
            .collect()
    }

    pub(super) fn remap_tags(&self, elements: &mut [SvgElement]) {
        for element in elements {
            let SvgKind::Group { tag, children } = &mut element.kind else {
                continue;
            };
            match tag {
                Some(Tag::Measure { index, end } | Tag::BarNumber { index, end }) => {
                    *index = self.written_index(*index);
                    *end = self.written_index(*end);
                }
                Some(
                    Tag::PartLabel {
                        measure_index_start,
                        measure_index_end,
                        ..
                    }
                    | Tag::LyricLabel {
                        measure_index_start,
                        measure_index_end,
                        ..
                    },
                ) => {
                    *measure_index_start = self.written_index(*measure_index_start);
                    *measure_index_end = self.written_index(*measure_index_end);
                }
                Some(Tag::BarLine {
                    measure_index_next,
                    measure_index_prev,
                }) => {
                    *measure_index_next = measure_index_next.map(|index| self.written_index(index));
                    *measure_index_prev = measure_index_prev.map(|index| self.written_index(index));
                }
                _ => {}
            }
            self.remap_tags(children);
        }
    }

    pub(super) fn remap_staff_measures(&self, measures: &mut [crate::mei_export::StaffMeasure]) {
        for measure in measures {
            measure.measure_index = self.written_index(measure.measure_index);
        }
    }
}
