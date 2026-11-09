import { expect } from '@playwright/test'
import { gotoShareUrl } from '../../shareUrlHelper'
import { Given, Then, When } from './fixtures'

const SHARED_SOURCE = [
  'metadata { title "Shared Mobile Test" },',
  '',
  'parts ["Melody": notes],',
  '',
  'score """',
  '    (time=4/4 key=C4 bpm=120)',
  '    1 2 3 4',
  '    """,',
].join('\n')

Given('local storage is cleared on a mobile viewport', async ({ page }) => {
  // The original spec set the viewport via `test.use({ viewport })`, which
  // isn't allowed at module scope in a playwright-bdd step file — set it
  // directly instead, before the app's first navigation.
  await page.setViewportSize({ width: 375, height: 700 })
  await page.addInitScript(() => {
    localStorage.clear()
  })
})

When(
  'I open the share URL for {string} on the mobile viewport',
  async ({ page }, filename: string) => {
    await gotoShareUrl(page, filename, SHARED_SOURCE)
  },
)

Then(
  'the shared preview banner is visible, as seen in shared preview mobile',
  async ({ page }) => {
    await expect(page.locator('.shared-preview-banner')).toBeVisible()
  },
)

Then(
  'the app header only scrolls horizontally, not vertically',
  async ({ page }) => {
    const header = page.locator('.app-header')

    const { scrollWidth, clientHeight, overflowY } = await header.evaluate(
      (el) => ({
        scrollWidth: el.scrollWidth,
        clientHeight: el.getBoundingClientRect().height,
        overflowY: getComputedStyle(el).overflowY,
      }),
    )

    // On a 375px-wide viewport the header's content (shared-preview banner,
    // playback buttons, export controls) is wider than the viewport,
    // so it only fits without wrapping if it overflows horizontally instead.
    expect(scrollWidth).toBeGreaterThan(375)
    // A single, non-wrapped row stays well under the height a two-row wrap
    // would produce.
    expect(clientHeight).toBeLessThan(60)
    // The header must scroll horizontally only. `overflow-x: auto` alone
    // implicitly computes `overflow-y` to `auto` too (per the CSS overflow
    // spec), so any sub-pixel content/box height mismatch makes it vertically
    // scrollable as well unless `overflow-y` is pinned to `hidden`.
    expect(overflowY).toBe('hidden')
  },
)
