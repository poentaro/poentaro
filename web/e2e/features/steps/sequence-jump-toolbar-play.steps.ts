import { expect, test } from '@playwright/test'
import { Given, Then, When } from './fixtures'

/**
 * Same `sequence [...]` source as sequence-jump-toolbar.spec.ts — see that file
 * for the annotated line breakdown.
 */
const source = [
  'metadata { title "test" },',
  '',
  'parts ["M": notes],',
  '',
  'sequence ["A", "B", "B"],',
  '',
  'score """',
  '    time=4/4 key=C4 bpm=120 label="A"',
  '    1 2 3 4',
  '',
  '    label="B"',
  "    5 6 7 1'",
  '    """,',
].join('\n')

Given(
  'a sequence-test source {string} is loaded with the disk cache workaround',
  async ({ page }, _sequence: string) => {
    test.setTimeout(75_000)

    await page.addInitScript((src) => {
      localStorage.setItem(
        'poentaro:files:v1',
        JSON.stringify({
          active: 'sequence-test.poentaro',
          userFiles: { 'sequence-test.poentaro': src },
          bin: {},
          fileIds: { 'sequence-test.poentaro': crypto.randomUUID() },
        }),
      )
    }, source)

    await page.goto('/')
    // The play-from-current-measure button only renders once a sequence
    // toolbar entry is selected, so wait on a button that's always present
    // to confirm the app has finished loading.
    await page.waitForSelector('[data-testid="play-measure-button"]', {
      timeout: 15_000,
    })
  },
)

When('I select the first sequence toolbar entry', async ({ page }) => {
  const buttons = page
    .locator('[role="toolbar"]')
    .nth(1)
    .locator('button.section-jump-btn')
  await expect(buttons).toHaveCount(3, { timeout: 15_000 })
  await buttons.nth(0).click()
})

Then(
  'the sequence playback button aria-label says {string}',
  async ({ page }, label: string) => {
    const playBtn = page.getByTestId('play-from-current-measure-button')
    await expect(playBtn).toHaveAttribute('aria-label', label)
  },
)

Then(
  'the play-from-current-measure button becomes enabled once the soundfont loads',
  async ({ page }) => {
    const playBtn = page.getByTestId('play-from-current-measure-button')
    // The button stays disabled until the soundfont (a real ~30 MB asset)
    // finishes loading; wait for that instead of asserting a fixed delay.
    await expect(playBtn).toBeEnabled({ timeout: 30_000 })
  },
)

When('I click the play-from-current-measure button', async ({ page }) => {
  const playBtn = page.getByTestId('play-from-current-measure-button')
  await playBtn.click()
})

Then(
  'the play-from-current-measure button shows the playing state',
  async ({ page }) => {
    const playBtn = page.getByTestId('play-from-current-measure-button')
    await expect(playBtn).toHaveClass(/play-from-measure-btn--playing/, {
      timeout: 5_000,
    })
  },
)

Then(
  'the play-from-current-measure button eventually stops showing the playing state',
  async ({ page }) => {
    const playBtn = page.getByTestId('play-from-current-measure-button')
    // Measure 0 ("1 2 3 4") is short — playback should finish and the button
    // should revert to its normal (non-playing) state on its own. Generous
    // timeout: real-time `<audio>` playback duration is sensitive to CPU
    // throttling/contention in sandboxed/CI runners, so wall-clock completion
    // can lag well past the audio's nominal duration (see FLAKY_TESTS.md).
    await expect(playBtn).not.toHaveClass(/play-from-measure-btn--playing/, {
      timeout: 30_000,
    })
  },
)
