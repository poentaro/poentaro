import { expect, test } from '@playwright/test'
import { Given, Then, When } from './fixtures'

const SINGLE_PART_SOURCE = [
  'metadata { title "Test" },',
  '',
  'parts ["Melody" [M]: notes],',
  '',
  'score """',
  '    (bpm=120 key=C4 time=4/4)',
  '    1 2 3 4',
  '    """,',
].join('\n')

/**
 * Exercises the "Play All" button on a small fixed score, mirroring
 * `export-wav-toast.spec.ts`'s source-loading pattern: clicking it generates
 * and autoplays audio for the whole score, asserting on the button's
 * visible "playing" state rather than just "didn't crash". Mirrors
 * `play-measure-audio.spec.ts`.
 */
Given(
  'a single-part four-note score is loaded with the disk cache workaround',
  async ({ page }) => {
    test.setTimeout(90_000)

    await page.addInitScript((src) => {
      localStorage.setItem(
        'poentaro:files:v1',
        JSON.stringify({
          active: 'test.poentaro',
          userFiles: { 'test.poentaro': src },
          bin: {},
          fileIds: { 'test.poentaro': crypto.randomUUID() },
        }),
      )
    }, SINGLE_PART_SOURCE)
    await page.goto('/')
  },
)

Then('the Play All button is visible', async ({ page }) => {
  const playAllBtn = page.locator('button.play-all-btn')
  await expect(playAllBtn).toBeVisible({ timeout: 15_000 })
})

Then(
  'the Play All button becomes enabled once the soundfont loads',
  async ({ page }) => {
    const playAllBtn = page.locator('button.play-all-btn')
    // The button stays disabled until the soundfont (a real ~30 MB asset)
    // finishes loading; wait for that instead of asserting a fixed delay.
    await expect(playAllBtn).toBeEnabled({ timeout: 30_000 })
  },
)

When('I click the Play All button', async ({ page }) => {
  const playAllBtn = page.locator('button.play-all-btn')
  await playAllBtn.click()
})

Then('the Play All button shows the playing state', async ({ page }) => {
  const playAllBtn = page.locator('button.play-all-btn')
  // Playback engaged: button switches to the pause/playing variant. Audio
  // synthesis for the whole score takes noticeably longer than a single
  // measure (see `play-measure-audio.spec.ts`), so this allows more time.
  await expect(playAllBtn).toHaveClass(/play-all-btn--playing/, {
    timeout: 15_000,
  })
})

Then(
  'the Play All button eventually stops showing the playing state',
  async ({ page }) => {
    const playAllBtn = page.locator('button.play-all-btn')
    // This fixed score (four quarter notes) is short — playback should finish
    // and the button should revert to its normal (non-playing) state on its
    // own, without the user pausing it. Generous timeout: real-time `<audio>`
    // playback duration is sensitive to CPU throttling/contention in
    // sandboxed/CI runners, so wall-clock completion can lag well past the
    // audio's nominal duration (see FLAKY_TESTS.md).
    await expect(playAllBtn).not.toHaveClass(/play-all-btn--playing/, {
      timeout: 45_000,
    })
  },
)
