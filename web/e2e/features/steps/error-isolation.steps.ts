import { expect } from '@playwright/test'
import { Given, Then, When } from './fixtures'

const SOURCE_WITH_UNDERFLOW_IN_MEASURE_1 = [
  'metadata { title "t", author "a" },',
  '',
  'parts ["Melody": notes, "Melody lyrics" [Melodyv1]: lyrics [Melody]],',
  '',
  'score """',
  '    (time=4/4 key=C4 bpm=120)',
  '    [Melody] 1 2 3 4',
  '    [Melodyv1] a b',
  '',
  '    [Melody] 5 6 7 1',
  '    [Melodyv1] do re mi fa',
  '    """,',
].join('\n')

const FILE_STORE_KEY = 'poentaro:files:v1'

Given(
  'a user file with lyric underflow in measure 1 but valid lyrics in measure 2',
  async ({ page }) => {
    // Pre-seed localStorage with a user file that has lyric underflow in measure 1
    // but valid lyrics in measure 2. The demo file is read-only, so we use a user file.
    await page.addInitScript(
      ({ key, source }: { key: string; source: string }) => {
        const store = {
          active: 'test.poentaro',
          userFiles: { 'test.poentaro': source },
          bin: {},
          fileIds: { 'test.poentaro': 'test-file-id' },
        }
        localStorage.setItem(key, JSON.stringify(store))
      },
      { key: FILE_STORE_KEY, source: SOURCE_WITH_UNDERFLOW_IN_MEASURE_1 },
    )
  },
)

When('the app loads', async ({ page }) => {
  await page.goto('/')
  await page.waitForSelector('[data-testid="play-measure-button"]', {
    timeout: 15_000,
  })

  // Wait for the debounce + render worker round-trip.
  await page.waitForTimeout(2_000)
})

Then(
  'the error overlay rect for the erroneous measure appears in the SVG',
  async ({ page }) => {
    // The error overlay rect for the erroneous measure must appear in the SVG.
    const errorHighlight = page.locator(
      '.preview-page [data-testid="error-highlight"]',
    )
    await expect(errorHighlight).toBeVisible({ timeout: 5_000 })
  },
)

Then(
  "measure 2's lyrics still appear, confirming best-effort render",
  async ({ page }) => {
    // Measure 2 lyrics ("do") must also appear — confirming best-effort render.
    const previewContent = await page
      .locator('.preview-page')
      .first()
      .innerHTML()
    expect(previewContent).toContain('do')
  },
)
