import { expect, test } from '@playwright/test'
import { Given, Then } from './fixtures'

/**
 * `B` is written after `A` but played first, and the sequence ends on `B`
 * again — so resolving Play All by written-measure occurrence (first
 * occurrence of measure 1 → last occurrence of measure 2) would skip the
 * leading `B` entirely. Play All must instead name the whole `sequence [...]`
 * entry range.
 */
const source = [
  'metadata { title "test" },',
  '',
  'parts ["M": notes],',
  '',
  'sequence ["B", "A", "B"],',
  '',
  'score """',
  '    time=4/4 key=C4 bpm=120 label="A"',
  '    1 2 3 4',
  '',
  '    label="B"',
  "    5 6 7 1'",
  '    """,',
].join('\n')

interface RecordedWorkerRequest {
  type: string
  sequenceEntryStartIndex?: number
  sequenceEntryEndIndex?: number
}

declare global {
  interface Window {
    __recordedWorkerRequests: RecordedWorkerRequest[]
  }
}

Given(
  'a {string} sequence score is loaded with worker requests recorded',
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
      window.__recordedWorkerRequests = []
      const originalPostMessage = Worker.prototype.postMessage
      Worker.prototype.postMessage = function (
        this: Worker,
        message: unknown,
        ...rest: unknown[]
      ) {
        if (
          typeof message === 'object' &&
          message !== null &&
          'type' in message
        ) {
          const { type, sequenceEntryStartIndex, sequenceEntryEndIndex } =
            message as RecordedWorkerRequest
          window.__recordedWorkerRequests.push({
            type,
            sequenceEntryStartIndex,
            sequenceEntryEndIndex,
          })
        }
        // biome-ignore lint/suspicious/noExplicitAny: forwarding the original overloads verbatim
        return (originalPostMessage as any).call(this, message, ...rest)
      }
    }, source)

    await page.goto('/')
  },
)

Then(
  'the Play All audio request spans sequence entries {int} through {int}',
  async ({ page }, start: number, end: number) => {
    await expect
      .poll(
        () =>
          page.evaluate(() =>
            window.__recordedWorkerRequests
              .filter((r) => r.type === 'generateMeasureRangeAudio')
              .at(-1),
          ),
        { timeout: 15_000 },
      )
      .toEqual({
        type: 'generateMeasureRangeAudio',
        sequenceEntryStartIndex: start,
        sequenceEntryEndIndex: end,
      })
  },
)
