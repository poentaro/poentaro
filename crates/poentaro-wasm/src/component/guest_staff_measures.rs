//! Real bodies for the staff view's measure geometry `Guest` methods,
//! converting to and from `crate::staff_measures`. Kept at the exact
//! by-value parameter types the `Guest` trait requires (see `mod.rs`'s doc
//! comment), so `needless_pass_by_value` is relaxed here.
#![allow(clippy::needless_pass_by_value)]

use itertools::Itertools;

use super::*;
use crate::staff_measures;

fn bounding_box_from_wit(bounding_box: BoundingBox) -> staff_measures::BoundingBox {
    staff_measures::BoundingBox {
        left: bounding_box.left,
        right: bounding_box.right,
        top: bounding_box.top,
        bottom: bounding_box.bottom,
    }
}

fn bounding_box_to_wit(bounding_box: staff_measures::BoundingBox) -> BoundingBox {
    BoundingBox {
        left: bounding_box.left,
        right: bounding_box.right,
        top: bounding_box.top,
        bottom: bounding_box.bottom,
    }
}

fn staff_measure_geometry_from_wit(
    measure: StaffMeasureGeometry,
) -> staff_measures::StaffMeasureGeometry {
    staff_measures::StaffMeasureGeometry {
        measure_index: measure.measure_index as usize,
        measure_box: bounding_box_from_wit(measure.measure_box),
        staff_line_boxes: measure
            .staff_line_boxes
            .into_iter()
            .map(bounding_box_from_wit)
            .collect_vec(),
    }
}

fn measure_indices_from_wit(measure_indices: Vec<u32>) -> Vec<usize> {
    measure_indices
        .into_iter()
        .map(|index| index as usize)
        .collect_vec()
}

fn measure_indices_to_wit(measure_indices: Vec<usize>) -> Vec<u32> {
    measure_indices
        .into_iter()
        .map(|index| index as u32)
        .collect_vec()
}

pub(super) fn staff_measure_at_point(
    measures: Vec<StaffMeasureGeometry>,
    x: f64,
    y: f64,
) -> Option<u32> {
    let measures = measures
        .into_iter()
        .map(staff_measure_geometry_from_wit)
        .collect_vec();
    staff_measures::staff_measure_at_point(&measures, staff_measures::Point { x, y })
        .map(|index| index as u32)
}

pub(super) fn staff_measure_area(measure: StaffMeasureGeometry) -> BoundingBox {
    bounding_box_to_wit(staff_measures::staff_measure_area(
        &staff_measure_geometry_from_wit(measure),
    ))
}

pub(super) fn highlighted_staff_measures(
    measure_indices: Vec<u32>,
    ranges: Vec<MeasureRangeIn>,
) -> Vec<u32> {
    let ranges = ranges
        .into_iter()
        .map(measure_range_in_from_wit)
        .collect_vec();
    measure_indices_to_wit(staff_measures::highlighted_staff_measures(
        &measure_indices_from_wit(measure_indices),
        &ranges,
    ))
}

pub(super) fn pending_staff_measures(
    measure_indices: Vec<u32>,
    anchor: MeasureRangeIn,
    current: MeasureRangeIn,
) -> Vec<u32> {
    measure_indices_to_wit(staff_measures::pending_staff_measures(
        &measure_indices_from_wit(measure_indices),
        measure_range_in_from_wit(anchor),
        measure_range_in_from_wit(current),
    ))
}
