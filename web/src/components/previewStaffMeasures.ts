import {
  type BoundingBox,
  poentaroWasm,
  type StaffMeasure,
  type StaffMeasureGeometry,
} from '../poentaroWasm'
import type { MeasureRange } from './previewSelection'

const SVG_NS = 'http://www.w3.org/2000/svg'

/** `data-testid` of the rect painted behind each selected staff measure. */
export const STAFF_MEASURE_HIGHLIGHT_TEST_ID = 'staff-measure-highlight'

/** `data-testid` of each staff measure's transparent hover/click target. */
export const STAFF_MEASURE_HIT_TARGET_TEST_ID = 'staff-measure-hit-target'

interface StaffMeasureElement {
  element: SVGGraphicsElement
  measureIndex: number
}

/** Each measure's group in Verovio's SVG, found by the element id Rust
 * listed for it (`StaffMeasure`), so the id format stays Rust's alone. */
function staffMeasureElements(
  container: HTMLElement,
  measures: StaffMeasure[],
): StaffMeasureElement[] {
  return measures.flatMap(({ elementId, measureIndex }) => {
    const element = container.querySelector<SVGGraphicsElement>(
      `#${CSS.escape(elementId)}`,
    )
    return element ? [{ element, measureIndex }] : []
  })
}

/** Copied out of the `DOMRect`, whose fields are getters. */
function clientBox(element: SVGGraphicsElement): BoundingBox {
  const { left, right, top, bottom } = element.getBoundingClientRect()
  return { left, right, top, bottom }
}

function userSpaceBox(element: SVGGraphicsElement): BoundingBox {
  const { x, y, width, height } = element.getBBox()
  return { left: x, right: x + width, top: y, bottom: y + height }
}

/** The measure group's box and its staff lines' (the `path`s directly under
 * each `g.staff`), measured with `boxOf`, for Rust's `staff_measures`. */
function staffMeasureGeometry(
  { element, measureIndex }: StaffMeasureElement,
  boxOf: (element: SVGGraphicsElement) => BoundingBox,
): StaffMeasureGeometry {
  return {
    measureIndex,
    measureBox: boxOf(element),
    staffLineBoxes: Array.from(
      element.querySelectorAll<SVGGraphicsElement>(':scope > g.staff > path'),
    ).map(boxOf),
  }
}

/** The staff measure whose area contains `(x, y)`. Tested against that area
 * rather than the element under the pointer, since the blank space between a
 * measure's notes isn't painted and so never hit. */
export function staffMeasureRangeAtPoint(
  container: HTMLElement,
  measures: StaffMeasure[],
  x: number,
  y: number,
): MeasureRange | undefined {
  const index = poentaroWasm().staffMeasureAtPoint(
    staffMeasureElements(container, measures).map((measure) =>
      staffMeasureGeometry(measure, clientBox),
    ),
    x,
    y,
  )
  return index === undefined ? undefined : { start: index, end: index }
}

/** The group of the staff measure `measureIndex`, to scroll into view. */
export function staffMeasureElement(
  container: HTMLElement,
  measures: StaffMeasure[],
  measureIndex: number,
): Element | undefined {
  return staffMeasureElements(container, measures).find(
    (measure) => measure.measureIndex === measureIndex,
  )?.element
}

/** A rect covering `measure`'s area (Rust's `staff_measure_area`),
 * inserted behind its contents. */
function insertMeasureRect(
  measure: StaffMeasureElement,
  className: string,
  testId: string,
): void {
  const box = poentaroWasm().staffMeasureArea(
    staffMeasureGeometry(measure, userSpaceBox),
  )
  const rect = document.createElementNS(SVG_NS, 'rect')
  rect.setAttribute('x', String(box.left))
  rect.setAttribute('y', String(box.top))
  rect.setAttribute('width', String(box.right - box.left))
  rect.setAttribute('height', String(box.bottom - box.top))
  rect.setAttribute('class', className)
  rect.dataset.testid = testId
  measure.element.insertBefore(rect, measure.element.firstChild)
}

/** Gives every staff measure a transparent rect over its area, so the
 * blank space between its notes (which Verovio leaves unpainted) is hovered
 * too — `preview.css` tints it and shows a pointer, like the numbered view's
 * click targets. Run once per set of pages. */
export function applyStaffMeasureHitTargets(
  container: HTMLElement,
  measures: StaffMeasure[],
): void {
  for (const measure of staffMeasureElements(container, measures)) {
    if (
      measure.element.querySelector(
        `:scope > [data-testid="${STAFF_MEASURE_HIT_TARGET_TEST_ID}"]`,
      )
    )
      continue
    insertMeasureRect(
      measure,
      'staff-measure-hit-target',
      STAFF_MEASURE_HIT_TARGET_TEST_ID,
    )
  }
}

/** Paints a rect behind every staff measure in `measureIndices`, clearing
 * the previous ones. Idempotent, so it's re-run whenever Verovio's pages are
 * swapped in. */
function paintStaffMeasures(
  container: HTMLElement,
  measures: StaffMeasureElement[],
  measureIndices: Uint32Array,
): void {
  for (const rect of Array.from(
    container.querySelectorAll(
      `[data-testid="${STAFF_MEASURE_HIGHLIGHT_TEST_ID}"]`,
    ),
  )) {
    rect.remove()
  }
  const painted = new Set(measureIndices)
  for (const measure of measures) {
    if (!painted.has(measure.measureIndex)) continue
    insertMeasureRect(
      measure,
      'staff-measure-highlight',
      STAFF_MEASURE_HIGHLIGHT_TEST_ID,
    )
  }
}

/** Paints the staff measures the editor's selection `ranges` cover. */
export function applyStaffMeasureHighlights(
  container: HTMLElement,
  measures: StaffMeasure[],
  ranges: MeasureRange[],
): void {
  const elements = staffMeasureElements(container, measures)
  paintStaffMeasures(
    container,
    elements,
    poentaroWasm().highlightedStaffMeasures(measureIndicesOf(elements), ranges),
  )
}

/** Paints the staff measures a click-and-click gesture spans, from its
 * `anchor` to its `current` end. */
export function applyPendingStaffMeasures(
  container: HTMLElement,
  measures: StaffMeasure[],
  anchor: MeasureRange,
  current: MeasureRange,
): void {
  const elements = staffMeasureElements(container, measures)
  paintStaffMeasures(
    container,
    elements,
    poentaroWasm().pendingStaffMeasures(
      measureIndicesOf(elements),
      anchor,
      current,
    ),
  )
}

function measureIndicesOf(measures: StaffMeasureElement[]): Uint32Array {
  return Uint32Array.from(measures, ({ measureIndex }) => measureIndex)
}
