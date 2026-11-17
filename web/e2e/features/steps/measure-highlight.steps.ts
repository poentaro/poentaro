import { expect } from '@playwright/test'
import { Then, When } from './fixtures'

/**
 * The default demo source (demo/01-pitches.poentaro) has the following Monaco
 * line numbers (1-based):
 *
 *   1  metadata { title "Pitches", max measures per system 1 },
 *   ...
 *   5  score """
 *   6      label="Scale degrees & rest"
 *   7      [M] 1 2 3 0   ← melody note line → measure 1
 *
 * Line 1 (`metadata`) sits outside any measure span.
 */

function highlightRect(page: import('@playwright/test').Page) {
  return page.locator('.preview-page [data-testid="measure-highlight"]')
}

When('I jump to line {int}', async ({ page }, line: number) => {
  await page.keyboard.press('Control+g')
  await page.keyboard.type(String(line))
  await page.keyboard.press('Enter')
  // Allow the 300 ms debounce plus the highlight render worker round-trip.
  await page.waitForTimeout(1_000)
})

Then('the measure highlight rect is visible', async ({ page }) => {
  await expect(highlightRect(page)).toBeVisible({ timeout: 5_000 })
})

Then('the measure highlight rect is not visible', async ({ page }) => {
  await expect(highlightRect(page)).not.toBeVisible({ timeout: 3_000 })
})

When('I select the whole current line', async ({ page }) => {
  // A real (non-empty) selection, not just a caret.
  await page.keyboard.press('Home')
  await page.keyboard.press('Shift+End')
})

When(
  'I collapse the selection back to a caret at the end of the line',
  async ({ page }) => {
    await page.keyboard.press('End')
    await page.waitForTimeout(1_000)
  },
)
