import { expect } from '@playwright/test'
import { tagSelector } from '../../../src/dataAttributes'
import { Given, Then, When } from './fixtures'

/**
 * Regression test for PENDING_TASK_section-preview-scroll-sync.md: clicking
 * a section button moved the Monaco selection but left the SVG preview
 * scrolled wherever it already was, because the preview's only scroll
 * trigger was gated behind the amber caret-only highlight, and section
 * jumps deliberately select a real (non-empty) range to keep that highlight
 * off. Preview.tsx now also scrolls to the plain `[data-tag="measure"]`
 * group for the selection's first measure when no highlight rect exists.
 *
 * Builds a source with section "A" (measure 0) followed by 59 more
 * measures, then section "B" starting at the last measure — enough content
 * to overflow `.preview-pages` vertically so a scroll is observable.
 */
function buildLongSectionedSource(totalMeasures: number): string {
  const lines = [
    'metadata { title "section scroll sync test" },',
    '',
    'parts ["Melody" [M]: notes],',
    '',
    'score """',
    'label="A"',
    '[M] 1 2 3 4',
    '',
  ]
  for (let i = 1; i < totalMeasures - 1; i++) {
    lines.push('[M] 1 2 3 4')
    lines.push('')
  }
  lines.push('label="B"')
  lines.push('[M] 1 2 3 4')
  return [...lines, '""",'].join('\n')
}

const TOTAL_MEASURES = 60
const LAST_MEASURE_INDEX = TOTAL_MEASURES - 1

let scrollTopBeforeJump: number | undefined

Given(
  'a long sectioned source with sections {string} and {string} is loaded',
  async ({ page }, _sectionA: string, _sectionB: string) => {
    const source = buildLongSectionedSource(TOTAL_MEASURES)

    await page.addInitScript((src) => {
      localStorage.setItem(
        'poentaro:files:v1',
        JSON.stringify({
          active: 'section-scroll-sync-test.poentaro',
          userFiles: { 'section-scroll-sync-test.poentaro': src },
          bin: {},
          fileIds: { 'section-scroll-sync-test.poentaro': crypto.randomUUID() },
        }),
      )
    }, source)

    await page.goto('/')
    await page.waitForSelector('button.section-jump-btn', { timeout: 15_000 })

    await page.waitForSelector(
      tagSelector('measure', { index: LAST_MEASURE_INDEX }),
      { timeout: 15_000 },
    )
  },
)

When(
  'I click the section jump button labeled {string} to scroll to that section',
  async ({ page }, label: string) => {
    const previewPages = page.locator('.preview-pages')
    scrollTopBeforeJump = await previewPages.evaluate((el) => el.scrollTop)

    await page.locator('button.section-jump-btn', { hasText: label }).click()
  },
)

Then(
  'the selected measure range shows the last measure selected',
  async ({ page }) => {
    await expect(page.getByTestId('selected-measure-range')).toHaveText(
      `${LAST_MEASURE_INDEX}-${LAST_MEASURE_INDEX}`,
      { timeout: 3_000 },
    )
  },
)

Then(
  'the SVG preview scrolls to bring the last measure into view',
  async ({ page }) => {
    const previewPages = page.locator('.preview-pages')

    // The rAF-driven scrollIntoView should move the preview even though a
    // section jump's real (non-empty) selection never populates
    // `highlightedDocuments`/the amber highlight rect.
    await expect
      .poll(async () => previewPages.evaluate((el) => el.scrollTop), {
        timeout: 5_000,
      })
      .not.toBe(scrollTopBeforeJump)

    const lastMeasureGroup = page
      .locator(tagSelector('measure', { index: LAST_MEASURE_INDEX }))
      .first()
    await expect(lastMeasureGroup).toBeInViewport({ timeout: 5_000 })
  },
)
