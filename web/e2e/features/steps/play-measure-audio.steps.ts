import { expect, test } from '@playwright/test'
import { Given, Then, When } from './fixtures'

/**
 * `cmd-enter-play.spec.ts` only covers the no-op case (no measure selected).
 * This test exercises the real playback path: a measure is selected, the
 * soundfont has loaded, and clicking play actually starts audio — asserting
 * on the play button's visible "playing" state rather than just "didn't
 * crash".
 *
 * Uses the default demo file (demo/01-pitches.poentaro); line 7 is the first
 * note line ("[M] 1 2 3 0" → measure 0), as established by
 * `measure-label.spec.ts`.
 */
Given(
  'the app is loaded and the editor is focused, with the disk cache workaround',
  async ({ page, focusEditor }) => {
    test.setTimeout(75_000)

    await page.goto('/')
    await page.waitForSelector('[data-testid="play-measure-button"]', {
      timeout: 15_000,
    })

    // Focus the Monaco editor and place the cursor inside measure 0.
    await focusEditor()
  },
)

When(
  'I place the cursor inside measure {int} via line {int}',
  async ({ page }, _measureIndex: number, line: number) => {
    await page.keyboard.press('Control+g')
    await page.keyboard.type(String(line))
    await page.keyboard.press('Enter')
  },
)

Then('the play-measure button label mentions the measure', async ({ page }) => {
  const playBtn = page.locator('button.play-measure-btn')
  // Confirm the measure was actually selected (debounce + worker round-trip).
  await expect(playBtn).toHaveText(/Measure/, { timeout: 5_000 })
})

Then(
  'the play-measure button becomes enabled once the soundfont loads',
  async ({ page }) => {
    const playBtn = page.locator('button.play-measure-btn')
    // The button stays disabled until the soundfont (a real ~30 MB asset)
    // finishes loading; wait for that instead of asserting a fixed delay.
    await expect(playBtn).toBeEnabled({ timeout: 30_000 })
  },
)

When('I click the play-measure button', async ({ page }) => {
  const playBtn = page.locator('button.play-measure-btn')
  await playBtn.click()
})

Then('the play-measure button shows the playing state', async ({ page }) => {
  const playBtn = page.locator('button.play-measure-btn')
  // Playback engaged: button switches to the pause/playing variant.
  await expect(playBtn).toHaveClass(/play-measure-btn--playing/, {
    timeout: 5_000,
  })
})

Then(
  'the play-measure button eventually stops showing the playing state',
  async ({ page }) => {
    const playBtn = page.locator('button.play-measure-btn')
    // Measure 0 ("[M] 1 2 3 0", four quarter notes) is short — playback should
    // finish and the button should revert to its normal (non-playing) state on
    // its own, without the user pausing it. Generous timeout: real-time
    // `<audio>` playback duration is sensitive to CPU throttling/contention in
    // sandboxed/CI runners, so wall-clock completion can lag well past the
    // audio's nominal duration (see FLAKY_TESTS.md).
    await expect(playBtn).not.toHaveClass(/play-measure-btn--playing/, {
      timeout: 30_000,
    })
  },
)
