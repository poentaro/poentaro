import { expect } from '@playwright/test'
import { Given, Then, When } from './fixtures'
import { primeMeasureSpans } from './part-label-cmd-click-play-selection-with-hidden-part.steps'

type CaptureWindow = typeof window & { __lastEnabledTracks?: string[] }

Given(
  'the preview has settled, as seen in play selection follows part visibility',
  async ({ page }) => {
    await primeMeasureSpans(page)
  },
)

When(
  'I toggle the visibility of the Harmony part, as seen in play selection follows part visibility',
  async ({ page }) => {
    const harmonyPill = page.locator('.part-toggle-pill').filter({
      has: page.locator('.part-toggle-abbr', { hasText: /^H$/ }),
    })
    await harmonyPill.locator('.part-toggle-segment--eye').click()
  },
)

When('I play the selection again', async ({ page }) => {
  // While the first clip still plays, the button pauses it instead.
  const pause = page.locator('button.play-measure-btn--playing')
  if (await pause.isVisible()) await pause.click()
  const play = page.locator(
    'button.play-measure-btn:not(.play-measure-btn--playing)',
  )
  await expect(play).toBeEnabled()
  await play.click()
})

Then(
  'the captured enabled tracks are exactly {string}',
  async ({ page }, expected: string) => {
    await page.waitForFunction(
      () => (window as CaptureWindow).__lastEnabledTracks !== undefined,
      { timeout: 10_000 },
    )
    const enabledTracks = await page.evaluate(() => {
      const win = window as CaptureWindow
      const tracks = win.__lastEnabledTracks
      win.__lastEnabledTracks = undefined
      return tracks
    })
    expect([...(enabledTracks ?? [])].sort().join(',')).toBe(expected)
  },
)
