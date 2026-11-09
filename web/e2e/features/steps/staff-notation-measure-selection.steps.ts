import { expect, type Page, test } from '@playwright/test'
import {
  STAFF_MEASURE_HIGHLIGHT_TEST_ID,
  STAFF_MEASURE_HIT_TARGET_TEST_ID,
} from '../../../src/components/previewStaffMeasures'
import { Given, Then, When } from './fixtures'

const SECTION_A_MEASURES = 24

/** Section A is long enough to push section B ("B", measures 24-25) off the
 * first staff page, so jumping to it has to scroll. */
const SOURCE = [
  'metadata { title "Staff Selection Test" },',
  '',
  'parts ["Melody" [M]: notes],',
  '',
  'preset "Violin" { notation: violin },',
  '',
  'score """',
  'time=4/4 key=C4 bpm=240 label="A"',
  Array.from({ length: SECTION_A_MEASURES }, () => '[M] 1 2 3 4').join('\n\n'),
  '',
  'label="B"',
  '[M] 5 6 7 1',
  '',
  '[M] 4 3 2 1',
  '""",',
].join('\n')

/** Verovio's first render also downloads its ~7 MB module. */
const STAFF_RENDER_TIMEOUT = 30_000

/** Staff measure `index` (0-based), located by Verovio's own `measure`
 * class in document order — never by the element id, whose format is
 * Rust's alone. */
function staffMeasure(page: Page, index: number) {
  return page.locator('[data-testid="staff-page"] g.measure').nth(index)
}

function staffHighlights(page: Page) {
  return page.locator(
    `[data-testid="staff-page"] [data-testid="${STAFF_MEASURE_HIGHLIGHT_TEST_ID}"]`,
  )
}

Given(
  'a long two-section score with measure-audio capture is shown in the staff notation view',
  async ({ page }) => {
    test.info().setTimeout(90_000)
    await page.addInitScript((src) => {
      localStorage.setItem(
        'poentaro:files:v1',
        JSON.stringify({
          active: 'staff-selection.poentaro',
          userFiles: { 'staff-selection.poentaro': src },
          bin: {},
          fileIds: { 'staff-selection.poentaro': 'staff-selection-file' },
        }),
      )
      localStorage.setItem(
        'poentaro:chosen-preset:v1',
        JSON.stringify({ 'staff-selection-file': 'Violin' }),
      )
      // Records the range of every measure-audio request (see
      // `useMeasureAudioPlayback`), so playback can be checked without
      // waiting on real-time audio.
      const win = window as typeof window & {
        __measureAudioRanges?: { start: number; end: number }[]
      }
      win.__measureAudioRanges = []
      const origPostMessage = Worker.prototype.postMessage
      Worker.prototype.postMessage = function (
        this: Worker,
        message: unknown,
        ...rest: unknown[]
      ) {
        const request = message as {
          type?: string
          startMeasureIndex?: number
          endMeasureIndex?: number
        }
        if (request?.type === 'generateMeasureRangeAudio') {
          win.__measureAudioRanges?.push({
            start: request.startMeasureIndex ?? -1,
            end: request.endMeasureIndex ?? -1,
          })
        }
        // biome-ignore lint/suspicious/noExplicitAny: forwarding to the native postMessage overload
        return (origPostMessage as any).call(this, message, ...rest)
      }
    }, SOURCE)
    await page.goto('/')
    await expect(staffMeasure(page, SECTION_A_MEASURES + 1)).toBeAttached({
      timeout: STAFF_RENDER_TIMEOUT,
    })
  },
)

// Playwright's own `hover`/`click` wait for the measure to stop moving and
// check it's what's under the pointer; each measure's hover target covers
// its whole box, so its centre always lands inside it.
When('I click staff measure {int}', async ({ page }, index: number) => {
  await staffMeasure(page, index).click()
})

When('I hover staff measure {int}', async ({ page }, index: number) => {
  await staffMeasure(page, index).hover()
})

Then(
  'the pending-second-click banner is shown in the staff view',
  async ({ page }) => {
    await expect(page.getByTestId('pending-second-click-banner')).toBeVisible()
  },
)

Then(
  'the highlighted staff measures are {int} to {int}',
  async ({ page }, start: number, end: number) => {
    const count = end - start + 1
    await expect(staffHighlights(page)).toHaveCount(count)
    for (let index = start; index <= end; index++) {
      await expect(
        staffMeasure(page, index).getByTestId(STAFF_MEASURE_HIGHLIGHT_TEST_ID),
      ).toHaveCount(1)
    }
  },
)

Then(
  'staff measure {int} is scrolled into view',
  async ({ page }, index: number) => {
    await expect(staffMeasure(page, index)).toBeInViewport()
  },
)

Then(
  'the requested measure audio covers measures {int} to {int}',
  async ({ page }, start: number, end: number) => {
    await expect
      .poll(() =>
        page.evaluate(() =>
          (
            window as typeof window & {
              __measureAudioRanges?: { start: number; end: number }[]
            }
          ).__measureAudioRanges?.at(-1),
        ),
      )
      .toEqual({ start, end })
  },
)

/** The hover target's painted fill and the cursor over staff measure
 * `index`'s blank space. */
async function staffMeasureHoverStyle(page: Page, index: number) {
  return staffMeasure(page, index)
    .getByTestId(STAFF_MEASURE_HIT_TARGET_TEST_ID)
    .evaluate((rect) => {
      const style = getComputedStyle(rect)
      return { fill: style.fill, cursor: style.cursor }
    })
}

const TRANSPARENT_FILLS = ['transparent', 'rgba(0, 0, 0, 0)']

Then(
  'staff measure {int} shows the hover state',
  async ({ page }, index: number) => {
    await expect
      .poll(async () => {
        const { fill, cursor } = await staffMeasureHoverStyle(page, index)
        return { tinted: !TRANSPARENT_FILLS.includes(fill), cursor }
      })
      .toEqual({ tinted: true, cursor: 'pointer' })
  },
)

Then(
  'staff measure {int} does not show the hover state',
  async ({ page }, index: number) => {
    const { fill } = await staffMeasureHoverStyle(page, index)
    expect(TRANSPARENT_FILLS).toContain(fill)
  },
)
