import { expect, test } from '@playwright/test'
import { Given, Then, When } from './fixtures'

/** `A` sings line 1 the first time and line 2 (`w.2`) the second time. Play
 * Selection on the second `A` must ask for that occurrence (entry 1, with
 * the sequence respected), not the written measure. */
const source = [
  'metadata { title "test" },',
  '',
  'parts ["Melody" [M]: notes, "Words" [w]: lyrics [M]],',
  '',
  'sequence ["A", "A" WITH [w] 2],',
  '',
  'score """',
  '    time=4/4 key=C4 bpm=120 label="A"',
  '    [M] 1 2 3 4',
  '    [w] 一 二 三 四',
  '    [w] 五 六 七 八',
  '    """,',
].join('\n')

interface RecordedPlaybackRequest {
  respectSequence?: boolean
  sequenceEntryStartIndex?: number
  sequenceEntryEndIndex?: number
}

declare global {
  interface Window {
    __recordedPlaybackRequests: RecordedPlaybackRequest[]
  }
}

Given(
  'an {string} sequence score is loaded with playback requests recorded',
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
      window.__recordedPlaybackRequests = []
      const originalPostMessage = Worker.prototype.postMessage
      Worker.prototype.postMessage = function (
        this: Worker,
        message: unknown,
        ...rest: unknown[]
      ) {
        const request = message as { type?: string } & RecordedPlaybackRequest
        if (request?.type === 'generateMeasureRangeAudio') {
          window.__recordedPlaybackRequests.push({
            respectSequence: request.respectSequence,
            sequenceEntryStartIndex: request.sequenceEntryStartIndex,
            sequenceEntryEndIndex: request.sequenceEntryEndIndex,
          })
        }
        // biome-ignore lint/suspicious/noExplicitAny: forwarding the original overloads verbatim
        return (originalPostMessage as any).call(this, message, ...rest)
      }
    }, source)
    await page.goto('/')
    // Play All is enabled once the soundfont has loaded.
    await expect(page.getByTestId('play-all-button')).toBeEnabled({
      timeout: 30_000,
    })
  },
)

When('I select sequence toolbar entry {int}', async ({ page }, n: number) => {
  const entries = page
    .locator('[role="toolbar"]')
    .nth(1)
    .locator('button.section-jump-btn')
  await expect(entries).toHaveCount(2, { timeout: 15_000 })
  await entries.nth(n - 1).click()
})

When('I click Play Selection', async ({ page }) => {
  await page.getByTestId('play-measure-button').click()
})

Then(
  'the last playback request plays sequence entry {int} as the sequence plays it',
  async ({ page }, entry: number) => {
    await expect
      .poll(
        () => page.evaluate(() => window.__recordedPlaybackRequests.at(-1)),
        { timeout: 15_000 },
      )
      .toEqual({
        respectSequence: true,
        sequenceEntryStartIndex: entry,
        sequenceEntryEndIndex: entry,
      })
  },
)
