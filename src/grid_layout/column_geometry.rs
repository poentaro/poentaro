use super::GridRow;
use crate::grid_layout::layout::LABEL_COLS;

impl GridRow {
    /// Column geometry for this row, given the usable page width and the
    /// score-wide fixed part-label width. Rows with `has_label_region: true`
    /// (system rows) get a label column width independent of the row's own
    /// musical density; the music region is split across `measure_layout`'s
    /// measures, and each measure's own width in turn across its columns,
    /// by the spring-and-rod model (see **Rod and spring** in
    /// `ARCHITECTURE.md`): every measure/column first gets its own hard-
    /// minimum rod (`rod_pt`/`column_rods`, derived from real content
    /// width), and only the slack left over after every rod in the row is
    /// satisfied is distributed proportionally by spacing weight (denser
    /// measures/columns get more of it). If a system's summed rods exceed
    /// the space available, slack clamps to zero and every measure/column
    /// renders at exactly its rod — the system overflows the page instead
    /// of compressing below its own content (see `layout::layout`'s
    /// overflow diagnostic). Rows without a label region (headers, footers)
    /// or an empty `measure_layout` divide the full width evenly, as
    /// before.
    pub fn column_geometry(&self, usable_width_pt: f32, label_width_pt: f32) -> ColumnGeometry {
        if self.has_label_region {
            let label_col_width = label_width_pt / LABEL_COLS as f32;
            if self.measure_layout.is_empty() {
                let music_cols = self.column_count - LABEL_COLS;
                return ColumnGeometry {
                    origin_x: 0.0,
                    label_cols: LABEL_COLS,
                    label_col_width,
                    first_segment: ColumnSegment {
                        start_col: LABEL_COLS as f32,
                        col_count: music_cols as f32,
                        col_width: (usable_width_pt - label_width_pt) / music_cols as f32,
                        x_start: label_width_pt,
                        accidental_lead: 0.0,
                    },
                    rest_segments: Vec::new(),
                };
            }
            let usable_music_width = usable_width_pt - label_width_pt;
            let total_rod: f32 = self.measure_layout.iter().map(|m| m.rod_pt).sum();
            let total_weight: f32 = self.measure_layout.iter().map(|m| m.weight).sum();
            let system_slack = (usable_music_width - total_rod).max(0.0);
            let mut x = label_width_pt;
            let segments: Vec<ColumnSegment> = self
                .measure_layout
                .iter()
                .flat_map(|m| {
                    let measure_width = m.rod_pt + system_slack * m.weight / total_weight;
                    // Always >= 0: `measure_width` is `rod_pt` plus a
                    // non-negative share of `system_slack`. `column_rods`
                    // always sums to exactly `rod_pt` (see that field's doc
                    // comment), so this measure's columns sum to exactly
                    // `measure_width` — no gap, no overlap.
                    let measure_slack = measure_width - m.rod_pt;
                    let column_weight_sum: f32 = m.column_weights.iter().sum();
                    m.column_weights
                        .iter()
                        .zip(&m.column_rods)
                        .zip(&m.column_accidental_leads)
                        .enumerate()
                        .map(|(i, ((&w, &rod), &accidental_lead))| {
                            let col_width = rod + measure_slack * w / column_weight_sum;
                            let seg = ColumnSegment {
                                start_col: m.start_col as f32 + i as f32,
                                col_count: 1.0,
                                col_width,
                                x_start: x,
                                accidental_lead,
                            };
                            x += col_width;
                            seg
                        })
                        .collect::<Vec<_>>()
                })
                .collect();
            let mut segments = segments.into_iter();
            let first_segment = segments.next().unwrap_or(ColumnSegment {
                start_col: LABEL_COLS as f32,
                col_count: 0.0,
                col_width: 0.0,
                x_start: label_width_pt,
                accidental_lead: 0.0,
            });
            ColumnGeometry {
                origin_x: 0.0,
                label_cols: LABEL_COLS,
                label_col_width,
                first_segment,
                rest_segments: segments.collect(),
            }
        } else {
            self.uniform_column_geometry(usable_width_pt)
        }
    }

    /// A row without a label region (header, footer, separator): its
    /// columns split the full width evenly.
    fn uniform_column_geometry(&self, usable_width_pt: f32) -> ColumnGeometry {
        let col_width = usable_width_pt / self.column_count as f32;
        ColumnGeometry {
            origin_x: 0.0,
            label_cols: 0,
            label_col_width: col_width,
            first_segment: ColumnSegment {
                start_col: 0.0,
                col_count: self.column_count as f32,
                col_width,
                x_start: 0.0,
                accidental_lead: 0.0,
            },
            rest_segments: Vec::new(),
        }
    }
}

/// One contiguous run of uniform-width columns within `ColumnGeometry`'s
/// music region — one per musical column once per-column proportional
/// widths are in effect (`col_count` is then always `1.0`), or a single
/// segment spanning the whole music region otherwise.
#[derive(Debug, Clone, Copy)]
struct ColumnSegment {
    start_col: f32,
    col_count: f32,
    col_width: f32,
    /// x-offset (from the row's left edge) of this segment's start.
    x_start: f32,
    /// The column's accidental lead (see
    /// `MeasureColumnLayout::column_accidental_leads`); always `0.0` for a
    /// multi-column segment.
    accidental_lead: f32,
}

/// Resolves a grid column index to a pixel x-offset (from the row's left
/// edge) and column width, so the fixed-width label region (columns
/// `0..label_cols`) and the variable-width music region (`label_cols..`)
/// can share a `GridRow` without the label's rendered width depending on
/// how many musical columns the row has. The music region may itself be
/// split into multiple `ColumnSegment`s of differing width (one per
/// measure), making `x_start` a piecewise-linear (but still continuous)
/// function of `column`.
#[derive(Debug, Clone)]
pub struct ColumnGeometry {
    /// x-offset added to every position below: `0.0` for a row-relative
    /// geometry, the page's left margin once [`ColumnGeometry::offset_by`]
    /// places the row on the page.
    origin_x: f32,
    label_cols: u32,
    label_col_width: f32,
    /// The music region's first segment, split out from `rest_segments` so
    /// the region is structurally guaranteed non-empty — `segment_for` can
    /// then fall back to it without any panicking `unwrap`/`expect`.
    first_segment: ColumnSegment,
    rest_segments: Vec<ColumnSegment>,
}

impl ColumnGeometry {
    /// The same geometry, shifted right by `origin_x` (e.g. the page's left
    /// margin), so `x_start` returns page coordinates instead of positions
    /// relative to the row's left edge.
    pub fn offset_by(self, origin_x: f32) -> Self {
        Self { origin_x, ..self }
    }

    /// Finds the segment `column` falls into. Segments are contiguous and
    /// sorted by `start_col`; a column past the last segment's end resolves
    /// to that last segment (segments are never empty).
    fn segment_for(&self, column: f32) -> &ColumnSegment {
        if column < self.first_segment.start_col + self.first_segment.col_count {
            return &self.first_segment;
        }
        self.rest_segments
            .iter()
            .find(|seg| column < seg.start_col + seg.col_count)
            .unwrap_or_else(|| self.rest_segments.last().unwrap_or(&self.first_segment))
    }

    /// x-offset of the start of `column`, relative to the row's left edge
    /// plus `origin_x` (see [`ColumnGeometry::offset_by`]). `column` may be
    /// fractional (e.g. a highlight's `column_start`); it is never expected
    /// to straddle the label/music boundary.
    pub fn x_start(&self, column: f32) -> f32 {
        let row_x = if column < self.label_cols as f32 {
            column * self.label_col_width
        } else {
            let seg = self.segment_for(column);
            seg.x_start + (column - seg.start_col) * seg.col_width
        };
        self.origin_x + row_x
    }

    /// Width of a single column at `column`.
    pub fn col_width(&self, column: f32) -> f32 {
        if column < self.label_cols as f32 {
            self.label_col_width
        } else {
            self.segment_for(column).col_width
        }
    }

    /// x-offset of a glyph's anchor within `column` — flush at the column's
    /// left edge plus its accidental lead plus `padding`, the same for every
    /// glyph regardless of the column's own width or what else shares it.
    /// The lead leaves room for a sharp/flat drawn ahead of a note digit
    /// (see `render_note_head`), and applies to every glyph in the column so
    /// all parts' digits (and the ties/underlines keyed off them) stay
    /// vertically aligned on the same beat.
    pub fn glyph_left_anchor_x(&self, column: f32, padding: f32) -> f32 {
        self.x_start(column) + self.accidental_lead(column) + padding
    }

    fn accidental_lead(&self, column: f32) -> f32 {
        if column < self.label_cols as f32 {
            0.0
        } else {
            self.segment_for(column).accidental_lead
        }
    }
}
