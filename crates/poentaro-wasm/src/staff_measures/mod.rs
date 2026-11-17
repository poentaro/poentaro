//! The staff view's measure geometry: which of Verovio's measures a click
//! lands on, the area each measure's hover target and highlight cover, and
//! which measures a selection paints. The web layer only measures Verovio's
//! SVG elements and paints what comes back.

use itertools::Itertools;
use poentaro::grid_layout::MeasureRange;

/// A box in whichever coordinate space its caller measured it in (client
/// pixels for hit-testing, the SVG's user units for painting).
#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) struct BoundingBox {
    pub left: f64,
    pub right: f64,
    pub top: f64,
    pub bottom: f64,
}

impl BoundingBox {
    fn contains(&self, point: Point) -> bool {
        self.left <= point.x
            && point.x <= self.right
            && self.top <= point.y
            && point.y <= self.bottom
    }

    /// How far `y` is above or below this box (0 inside it).
    fn vertical_distance(&self, y: f64) -> f64 {
        (self.top - y).max(y - self.bottom).max(0.0)
    }

    fn union(boxes: impl IntoIterator<Item = BoundingBox>) -> Option<BoundingBox> {
        boxes.into_iter().reduce(|union, next| BoundingBox {
            left: union.left.min(next.left),
            right: union.right.max(next.right),
            top: union.top.min(next.top),
            bottom: union.bottom.max(next.bottom),
        })
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) struct Point {
    pub x: f64,
    pub y: f64,
}

/// One Verovio measure as the web layer measured it.
#[derive(Debug, Clone, PartialEq)]
pub(crate) struct StaffMeasureGeometry {
    pub measure_index: usize,
    /// The measure group's whole box: everything it draws, including slurs,
    /// ties and lyrics that cross its barlines into a neighbour.
    pub measure_box: BoundingBox,
    /// The box of each of its staff lines.
    pub staff_line_boxes: Vec<BoundingBox>,
}

/// The area that is `measure`'s own: across, from barline to barline (its
/// staff lines' extent), since anything crossing a barline stretches the
/// measure group's box into its neighbour's; down, the group's whole box,
/// so a ledger-line note above the staff still counts. A measure without
/// staff lines falls back to its whole box.
pub(crate) fn staff_measure_area(measure: &StaffMeasureGeometry) -> BoundingBox {
    match BoundingBox::union(measure.staff_line_boxes.iter().copied()) {
        Some(lines) => BoundingBox {
            left: lines.left,
            right: lines.right,
            ..measure.measure_box
        },
        None => measure.measure_box,
    }
}

/// The measure whose area contains `point`. Where the areas of measures on
/// neighbouring systems overlap (a low note below one system reaching into
/// the next), the one whose staff lines are vertically nearest wins.
pub(crate) fn staff_measure_at_point(
    measures: &[StaffMeasureGeometry],
    point: Point,
) -> Option<usize> {
    measures
        .iter()
        .filter(|measure| staff_measure_area(measure).contains(point))
        .min_by(|a, b| {
            staff_lines_vertical_distance(a, point.y)
                .total_cmp(&staff_lines_vertical_distance(b, point.y))
        })
        .map(|measure| measure.measure_index)
}

fn staff_lines_vertical_distance(measure: &StaffMeasureGeometry, y: f64) -> f64 {
    BoundingBox::union(measure.staff_line_boxes.iter().copied())
        .unwrap_or(measure.measure_box)
        .vertical_distance(y)
}

/// The measures of `measure_indices` inside any of `ranges`, in their given
/// order: what the staff view paints for the editor's selection.
pub(crate) fn highlighted_staff_measures(
    measure_indices: &[usize],
    ranges: &[MeasureRange],
) -> Vec<usize> {
    measure_indices
        .iter()
        .copied()
        .filter(|index| {
            ranges
                .iter()
                .any(|range| range.start <= *index && *index <= range.end)
        })
        .collect_vec()
}

/// The measures a click-and-click gesture spans between its `anchor` and
/// its `current` end, in either order: what the hover preview paints.
pub(crate) fn pending_staff_measures(
    measure_indices: &[usize],
    anchor: MeasureRange,
    current: MeasureRange,
) -> Vec<usize> {
    highlighted_staff_measures(
        measure_indices,
        &[MeasureRange {
            start: anchor.start.min(current.start),
            end: anchor.end.max(current.end),
        }],
    )
}

#[cfg(test)]
mod tests;
