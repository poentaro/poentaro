import { expect } from '@playwright/test'
import { focusEditor } from '../../fileSwitcherHelpers'
import { Given, Then, When } from './fixtures'

let svgBefore = ''

Given(
  "the app has loaded and the cursor is on the first measure's note line",
  async ({ page }) => {
    await page.goto('/')
    await page.waitForSelector('[data-testid="play-measure-button"]', {
      timeout: 15_000,
    })

    // Focus the Monaco editor and navigate to the first measure.
    await focusEditor(page)
    await page.keyboard.press('Control+g')
    await page.keyboard.type('7')
    await page.keyboard.press('Enter')

    // Allow the debounce + highlight render worker round-trip.
    await page.waitForTimeout(1_000)
  },
)

Then(
  'the measure highlight rect is visible, as seen in part toggle while measure focused',
  async ({ page }) => {
    // Confirm the measure highlight is visible (i.e. highlightedSvgs is in use).
    const highlightRect = page.locator(
      '.preview-page [data-testid="measure-highlight"]',
    )
    await expect(highlightRect).toBeVisible({ timeout: 5_000 })

    // Capture the rendered SVG content before any part toggle.
    svgBefore = await page.locator('.preview-pages').innerHTML()
  },
)

When('I hide the first part via its eye toggle', async ({ page }) => {
  // Uncheck the first part toggle (the "Melody" part). The checkbox itself
  // is visually hidden (opacity: 0, 0x0 box) in favor of its icon label, so
  // click the label that wraps it instead of the input directly.
  const firstPartToggle = page
    .locator('.part-toggles .part-toggle-segment--eye')
    .first()
  await firstPartToggle.click()

  // Give the worker time to re-render with the updated parts filter.
  await page.waitForTimeout(1_500)
})

Then('the highlighted preview SVG content changes', async ({ page }) => {
  // The SVG should have changed to reflect the disabled part.
  // When the bug is present the highlighted SVG is never re-requested after a
  // parts toggle, so innerHTML will be identical to svgBefore.
  const svgAfter = await page.locator('.preview-pages').innerHTML()
  expect(svgAfter).not.toBe(svgBefore)
})
