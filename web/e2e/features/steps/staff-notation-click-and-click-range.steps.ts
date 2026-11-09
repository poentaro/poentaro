import { expect, type Page, test } from '@playwright/test'
import fc from 'fast-check'
import { STAFF_MEASURE_HIGHLIGHT_TEST_ID } from '../../../src/components/previewStaffMeasures'
import { Given, Then } from './fixtures'

const MEASURE_COUNT = 24

/** Bars that alternate a slur and a tie across the barline into the next
 * bar, with ledger-line notes above the melody's staff, lyrics under it and
 * a bass staff below — each of which can push a Verovio measure's box past
 * its own barlines. */
function measureLines(index: number): string[] {
  const melody = ['1 2 3 (4', "5) 6 7 1''", "3'' 2'' 1'' 5---~", '5--- 0'][
    index % 4
  ]
  return [
    `[M] ${melody}`,
    '[W] 白陽旗旛',
    `[B] ${index % 2 === 0 ? '1, 5, 1, 5,' : '(4, 3, 2, 1,)'}`,
  ]
}

const SOURCE = [
  'metadata { title "Staff Click Range Test" },',
  '',
  'parts ["Melody" [M]: notes, "Words" [W]: lyrics [M], "Bass" [B]: notes],',
  '',
  'preset "Staff" { notation: violin },',
  '',
  'score """',
  'time=4/4 key=C4 bpm=240',
  Array.from({ length: MEASURE_COUNT }, (_, index) =>
    measureLines(index).join('\n'),
  ).join('\n\n'),
  '""",',
].join('\n')

/** Verovio's first render also downloads its ~7 MB module. */
const STAFF_RENDER_TIMEOUT = 30_000

const STAFF_MEASURES = '[data-testid="staff-page"] g.measure'

Given(
  'a multi-system two-staff score with cross-barline slurs is shown in the staff notation view',
  async ({ page }) => {
    await page.addInitScript((src) => {
      localStorage.setItem(
        'poentaro:files:v1',
        JSON.stringify({
          active: 'staff-click-range.poentaro',
          userFiles: { 'staff-click-range.poentaro': src },
          bin: {},
          fileIds: { 'staff-click-range.poentaro': 'staff-click-range-file' },
        }),
      )
      localStorage.setItem(
        'poentaro:chosen-preset:v1',
        JSON.stringify({ 'staff-click-range-file': 'Staff' }),
      )
    }, SOURCE)
    await page.goto('/')
    await expect(page.locator(STAFF_MEASURES)).toHaveCount(MEASURE_COUNT, {
      timeout: STAFF_RENDER_TIMEOUT,
    })
  },
)

/** Where a click lands: on staff `staff` (taken modulo the measure's staff
 * count) of measure `measure`, at fractions `x`/`y` across the box of that
 * staff's five lines — the region a reader sees as that measure, unlike the
 * measure group's own box, which slurs, ledger lines and lyrics stretch. */
interface StaffClick {
  measure: number
  staff: number
  x: number
  y: number
}

const fraction = fc.double({ min: 0.05, max: 0.95, noNaN: true })

const staffClick: fc.Arbitrary<StaffClick> = fc.record({
  measure: fc.nat({ max: MEASURE_COUNT - 1 }),
  staff: fc.nat({ max: 1 }),
  x: fraction,
  y: fraction,
})

/** Scrolls the clicked staff to the middle of the preview, clear of what
 * overlays its edges (the floating controls at the top, the pitch drawer at
 * the bottom), and returns the click's point in client coordinates. Verovio
 * draws a staff's lines as the `path`s directly under its `g.staff`. */
async function staffClickPoint(
  page: Page,
  click: StaffClick,
): Promise<{ x: number; y: number }> {
  const staffs = page
    .locator(STAFF_MEASURES)
    .nth(click.measure)
    .locator(':scope > g.staff')
  const staff = staffs.nth(click.staff % (await staffs.count()))
  await staff.evaluate((element) =>
    element.scrollIntoView({ block: 'center', inline: 'center' }),
  )
  const box = await staff.evaluate((element) => {
    const lines = Array.from(element.querySelectorAll(':scope > path')).map(
      (line) => line.getBoundingClientRect(),
    )
    return {
      left: Math.min(...lines.map((line) => line.left)),
      right: Math.max(...lines.map((line) => line.right)),
      top: Math.min(...lines.map((line) => line.top)),
      bottom: Math.max(...lines.map((line) => line.bottom)),
    }
  })
  return {
    x: box.left + (box.right - box.left) * click.x,
    y: box.top + (box.bottom - box.top) * click.y,
  }
}

/** The indices of the staff measures painted as selected. */
function highlightedMeasures(page: Page): Promise<number[]> {
  return page
    .locator(STAFF_MEASURES)
    .evaluateAll(
      (measures, testId) =>
        measures.flatMap((measure, index) =>
          measure.querySelector(`:scope > [data-testid="${testId}"]`)
            ? [index]
            : [],
        ),
      STAFF_MEASURE_HIGHLIGHT_TEST_ID,
    )
}

Then(
  'every click-and-click on two staff measures selects exactly the measures between them',
  async ({ page }) => {
    test.info().setTimeout(300_000)
    await fc.assert(
      fc.asyncProperty(staffClick, staffClick, async (first, second) => {
        // Starts each run from no anchored gesture.
        await page.keyboard.press('Escape')
        const firstPoint = await staffClickPoint(page, first)
        await page.mouse.click(firstPoint.x, firstPoint.y)
        const secondPoint = await staffClickPoint(page, second)
        await page.mouse.click(secondPoint.x, secondPoint.y)
        const start = Math.min(first.measure, second.measure)
        const end = Math.max(first.measure, second.measure)
        await expect(page.getByTestId('selected-measure-range')).toHaveText(
          `${start}-${end}`,
          { timeout: 3_000 },
        )
        await expect
          .poll(() => highlightedMeasures(page), { timeout: 3_000 })
          .toEqual(Array.from({ length: end - start + 1 }, (_, i) => start + i))
      }),
      { numRuns: 40 },
    )
  },
)
