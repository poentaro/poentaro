//! Property tests: wherever on its staves a measure is clicked, that's the
//! measure hit, however far slurs, ties, ledger-line notes and lyrics spill
//! the measure groups' boxes into their neighbours, and a selection paints
//! exactly the measures it spans.
//!
//! Layouts are generated the way Verovio draws them: systems stacked down
//! the page, each a row of measures of varying widths sharing their
//! barlines, each measure with one to three staves of five lines, and each
//! measure group's box its staff lines stretched by a random spill on every
//! side, sideways up to a whole neighbour and down into the next system.

use itertools::Itertools;
use poentaro::grid_layout::MeasureRange;
use proptest::prelude::*;
use proptest::sample::Index;

use super::{
    highlighted_staff_measures, pending_staff_measures, staff_measure_area, staff_measure_at_point,
    BoundingBox, Point, StaffMeasureGeometry,
};

const LINE_SPACING: f64 = 10.0;
const LINE_THICKNESS: f64 = 1.0;
const STAFF_HEIGHT: f64 = 4.0 * LINE_SPACING + LINE_THICKNESS;

#[derive(Debug, Clone)]
struct Spill {
    left: f64,
    right: f64,
    top: f64,
    bottom: f64,
}

#[derive(Debug, Clone)]
struct SystemLayout {
    staff_count: usize,
    staff_gap: f64,
    gap_below: f64,
    measure_widths: Vec<f64>,
    spills: Vec<Spill>,
}

fn spill_strategy() -> impl Strategy<Value = Spill> {
    (0.0..200.0, 0.0..200.0, 0.0..150.0, 0.0..150.0).prop_map(|(left, right, top, bottom)| Spill {
        left,
        right,
        top,
        bottom,
    })
}

fn system_strategy() -> impl Strategy<Value = SystemLayout> {
    (1..=3_usize, 20.0..80.0, 30.0..120.0, 1..=5_usize).prop_flat_map(
        |(staff_count, staff_gap, gap_below, measure_count)| {
            (
                prop::collection::vec(20.0..200.0, measure_count),
                prop::collection::vec(spill_strategy(), measure_count),
            )
                .prop_map(move |(measure_widths, spills)| SystemLayout {
                    staff_count,
                    staff_gap,
                    gap_below,
                    measure_widths,
                    spills,
                })
        },
    )
}

/// The five line boxes of a staff whose top line is at `top`.
fn staff_lines(left: f64, right: f64, top: f64) -> Vec<BoundingBox> {
    (0..5)
        .map(|line| {
            let line_top = top + f64::from(line) * LINE_SPACING;
            BoundingBox {
                left,
                right,
                top: line_top,
                bottom: line_top + LINE_THICKNESS,
            }
        })
        .collect_vec()
}

/// The page's measures, numbered left to right and then down the systems.
fn layout(systems: &[SystemLayout]) -> Vec<StaffMeasureGeometry> {
    let system_tops = systems
        .iter()
        .scan(0.0, |top, system| {
            let this_top = *top;
            *top += system_height(system) + system.gap_below;
            Some(this_top)
        })
        .collect_vec();
    systems
        .iter()
        .zip(system_tops)
        .flat_map(|(system, system_top)| system_measures(system, system_top))
        .enumerate()
        .map(|(measure_index, measure)| StaffMeasureGeometry {
            measure_index,
            ..measure
        })
        .collect_vec()
}

fn system_height(system: &SystemLayout) -> f64 {
    system.staff_count as f64 * STAFF_HEIGHT + (system.staff_count - 1) as f64 * system.staff_gap
}

fn system_measures(system: &SystemLayout, system_top: f64) -> Vec<StaffMeasureGeometry> {
    let barlines = system
        .measure_widths
        .iter()
        .scan(0.0, |left, width| {
            *left += width;
            Some(*left)
        })
        .collect_vec();
    std::iter::once(0.0)
        .chain(barlines.iter().copied())
        .tuple_windows()
        .zip(&system.spills)
        .map(|((left, right), spill)| {
            let staff_line_boxes = (0..system.staff_count)
                .flat_map(|staff| {
                    let top = system_top + staff as f64 * (STAFF_HEIGHT + system.staff_gap);
                    staff_lines(left, right, top)
                })
                .collect_vec();
            StaffMeasureGeometry {
                measure_index: 0,
                measure_box: BoundingBox {
                    left: left - spill.left,
                    right: right + spill.right,
                    top: system_top - spill.top,
                    bottom: system_top + system_height(system) + spill.bottom,
                },
                staff_line_boxes,
            }
        })
        .collect_vec()
}

/// A point inside one of `measure`'s staves, clear of its barlines (shared
/// with the neighbours), at fractions `x`/`y` across that staff.
fn point_on_staff(measure: &StaffMeasureGeometry, staff: Index, x: f64, y: f64) -> Point {
    let staves = measure.staff_line_boxes.chunks(5).collect_vec();
    let lines = staves[staff.index(staves.len())];
    let first = lines[0];
    let last = lines[lines.len() - 1];
    Point {
        x: first.left + (first.right - first.left) * x,
        y: first.top + (last.bottom - first.top) * y,
    }
}

#[derive(Debug, Clone)]
struct Click {
    measure: Index,
    staff: Index,
    x: f64,
    y: f64,
}

fn click_strategy() -> impl Strategy<Value = Click> {
    (any::<Index>(), any::<Index>(), 0.01..0.99, 0.0..=1.0).prop_map(|(measure, staff, x, y)| {
        Click {
            measure,
            staff,
            x,
            y,
        }
    })
}

proptest! {
    #[test]
    fn a_click_on_a_measures_staff_hits_that_measure(
        systems in prop::collection::vec(system_strategy(), 1..=4),
        click in click_strategy(),
    ) {
        let measures = layout(&systems);
        let clicked = &measures[click.measure.index(measures.len())];
        let point = point_on_staff(clicked, click.staff, click.x, click.y);
        prop_assert_eq!(
            staff_measure_at_point(&measures, point),
            Some(clicked.measure_index)
        );
    }

    #[test]
    fn measures_of_a_system_never_share_area(
        system in system_strategy(),
    ) {
        let measures = layout(&[system]);
        for (a, b) in measures.iter().tuple_windows() {
            prop_assert!(staff_measure_area(a).right <= staff_measure_area(b).left);
        }
    }

    #[test]
    fn a_gesture_paints_every_measure_between_its_ends(
        measure_count in 1..40_usize,
        anchor in 0..40_usize,
        current in 0..40_usize,
    ) {
        let measure_indices = (0..measure_count).collect_vec();
        let painted = pending_staff_measures(
            &measure_indices,
            MeasureRange { start: anchor, end: anchor },
            MeasureRange { start: current, end: current },
        );
        let expected = (anchor.min(current)..=anchor.max(current))
            .filter(|index| *index < measure_count)
            .collect_vec();
        prop_assert_eq!(painted, expected);
    }

    #[test]
    fn a_selection_paints_exactly_the_measures_in_its_ranges(
        measure_count in 1..40_usize,
        bounds in prop::collection::vec((0..40_usize, 0..40_usize), 0..4),
    ) {
        let measure_indices = (0..measure_count).collect_vec();
        let ranges = bounds
            .iter()
            .map(|(a, b)| MeasureRange { start: *a.min(b), end: *a.max(b) })
            .collect_vec();
        let painted = highlighted_staff_measures(&measure_indices, &ranges);
        let expected = measure_indices
            .iter()
            .copied()
            .filter(|index| ranges.iter().any(|range| (range.start..=range.end).contains(index)))
            .collect_vec();
        prop_assert_eq!(painted, expected);
    }
}

/// The bug's own layout: measure 0's slur into measure 1 stretched its
/// group's box from 726 to 865, past its last barline at 857, so a click
/// just right of that barline selected measure 0 as well.
#[test]
fn a_slur_across_the_barline_doesnt_claim_the_next_measures_staff() {
    let lines = |left, right| staff_lines(left, right, 100.0);
    let measures = [
        StaffMeasureGeometry {
            measure_index: 0,
            measure_box: BoundingBox {
                left: 726.0,
                right: 865.0,
                top: 60.0,
                bottom: 180.0,
            },
            staff_line_boxes: lines(726.0, 857.0),
        },
        StaffMeasureGeometry {
            measure_index: 1,
            measure_box: BoundingBox {
                left: 857.0,
                right: 990.0,
                top: 70.0,
                bottom: 170.0,
            },
            staff_line_boxes: lines(857.0, 990.0),
        },
    ];
    assert_eq!(
        staff_measure_at_point(&measures, Point { x: 860.0, y: 120.0 }),
        Some(1)
    );
}

#[test]
fn a_measure_without_staff_lines_falls_back_to_its_whole_box() {
    let measure_box = BoundingBox {
        left: 0.0,
        right: 50.0,
        top: 0.0,
        bottom: 40.0,
    };
    let measure = StaffMeasureGeometry {
        measure_index: 3,
        measure_box,
        staff_line_boxes: vec![],
    };
    assert_eq!(staff_measure_area(&measure), measure_box);
    assert_eq!(
        staff_measure_at_point(&[measure], Point { x: 25.0, y: 20.0 }),
        Some(3)
    );
}
