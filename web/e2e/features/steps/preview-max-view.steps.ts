import { expect } from '@playwright/test'
import { Given, Then, When } from './fixtures'

const MEASURE = "    [S] 1_ 2_ 3_ 4_ 5_ 6_ 7_ 1'_"

const SOURCE = [
  'metadata { title "Max View Test" },',
  '',
  'parts ["Soprano" [S]: notes],',
  '',
  'score """',
  '    (bpm=120 key=C4 time=4/4)',
  ...Array.from({ length: 80 }, () => [MEASURE, '']).flat(),
  '    """,',
].join('\n')

/** One part, so one part label per system. */
const SYSTEMS = '.preview-page [data-tag="part-label"]'

Given(
  'a score several A4 pages long is loaded for the max view',
  async ({ page }) => {
    await page.addInitScript((src) => {
      localStorage.setItem(
        'poentaro:files:v1',
        JSON.stringify({
          active: 'max-view.poentaro',
          userFiles: { 'max-view.poentaro': src },
          bin: {},
          fileIds: { 'max-view.poentaro': crypto.randomUUID() },
        }),
      )
    }, SOURCE)
    await page.goto('/')
    await page.waitForSelector('.preview-page', { timeout: 15_000 })
  },
)

Given('the window is {int} pixels wide', async ({ page }, width: number) => {
  await page.setViewportSize({ width, height: 900 })
})

When(
  'I switch the preview to {string} view',
  async ({ page }, view: string) => {
    await page.getByTestId('page-view-toggle').click()
    await page.getByTestId(`page-view-toggle-${view}`).click()
  },
)

Then(
  'the preview shows more than {int} page(s)',
  async ({ page }, count: number) => {
    await expect
      .poll(() => page.locator('.preview-page').count())
      .toBeGreaterThan(count)
  },
)

Then('the preview shows 1 page as wide as the preview', async ({ page }) => {
  await expect(page.locator('.preview-page')).toHaveCount(1)
  await expect
    .poll(() =>
      page.evaluate(() => {
        const pages = document.querySelector('.preview-pages--max')
        const svg = document.querySelector('.preview-page svg')
        if (!pages || !svg) return Number.POSITIVE_INFINITY
        return Math.abs(svg.getBoundingClientRect().width - pages.clientWidth)
      }),
    )
    .toBeLessThan(2)
})

let countedSystems = 0

When("I count the preview's systems", async ({ page }) => {
  await expect(page.locator('.preview-page')).toHaveCount(1)
  await expect.poll(() => page.locator(SYSTEMS).count()).toBeGreaterThan(0)
  countedSystems = await page.locator(SYSTEMS).count()
})

When('the window narrows to {int} pixels', async ({ page }, width: number) => {
  await page.setViewportSize({ width, height: 900 })
})

Then('the preview shows more systems than it counted', async ({ page }) => {
  await expect
    .poll(() => page.locator(SYSTEMS).count())
    .toBeGreaterThan(countedSystems)
})

Then('the page view toggle reads {string}', async ({ page }, label: string) => {
  await expect(page.getByTestId('page-view-toggle')).toHaveText(label)
})
