import { expect } from '@playwright/test'
import { Given, Then, When } from './fixtures'

type State = { chords: string; lyrics: string }

let state: State = { chords: '', lyrics: '' }

Given(
  'a chord-lyrics score with chords {string} and lyrics {string}',
  async ({}, chords: string, lyrics: string) => {
    state = { chords, lyrics }
  },
)

function buildSource(): string {
  return [
    'metadata { title "chord lyrics test" },',
    '',
    'parts ["c": chords, "l": lyrics [c]],',
    '',
    'score """',
    '    time=4/4 key=C4 bpm=120',
    `    [c] ${state.chords}`,
    `    [l] ${state.lyrics}`,
    '    """,',
  ].join('\n')
}

When('the chord-lyrics score is rendered', async ({ page }) => {
  await page.addInitScript((src) => {
    localStorage.setItem(
      'poentaro:files:v1',
      JSON.stringify({
        active: 'chord-lyrics-test.poentaro',
        userFiles: { 'chord-lyrics-test.poentaro': src },
        bin: {},
        fileIds: { 'chord-lyrics-test.poentaro': 'chord-lyrics-test-id-001' },
      }),
    )
  }, buildSource())
  await page.goto('/')
  await page.waitForSelector('[data-testid="play-measure-button"]', {
    timeout: 15_000,
  })
  // Debounce + render worker round-trip.
  await page.waitForTimeout(2_000)
})

Then('the chord-lyrics score has no error', async ({ page }) => {
  await expect(page.locator('[data-testid="error-highlight"]')).toHaveCount(0)
})

Then('the chord-lyrics score has an error', async ({ page }) => {
  await expect(
    page.locator('.preview-page [data-testid="error-highlight"]').first(),
  ).toBeVisible({ timeout: 5_000 })
})

async function previewText(page: import('@playwright/test').Page) {
  return page
    .locator('.preview-page')
    .first()
    .evaluate((el) => el.textContent)
}

Then(
  'the chord-lyrics preview shows {string}',
  async ({ page }, text: string) => {
    expect(await previewText(page)).toContain(text)
  },
)

Then(
  'the chord-lyrics preview does not show {string}',
  async ({ page }, text: string) => {
    expect(await previewText(page)).not.toContain(text)
  },
)

async function expectChordAbove(
  page: import('@playwright/test').Page,
  chord: string,
  lyric: string,
) {
  const boxes = await page.evaluate(
    ({ chord, lyric }) => {
      const texts = Array.from(
        document.querySelectorAll('.preview-page svg text'),
      )
      const box = (el: Element | undefined) => {
        if (!el) return null
        const r = el.getBoundingClientRect()
        return { x: r.x, y: r.y, width: r.width, height: r.height }
      }
      return {
        chord: box(texts.find((t) => t.textContent?.trim() === chord)),
        lyric: box(texts.find((t) => t.textContent?.includes(lyric))),
      }
    },
    { chord, lyric },
  )
  expect(boxes.chord, `chord ${chord} not drawn`).not.toBeNull()
  expect(boxes.lyric, `lyric ${lyric} not drawn`).not.toBeNull()
  if (!boxes.chord || !boxes.lyric) return
  const chordCenter = boxes.chord.x + boxes.chord.width / 2
  expect(chordCenter).toBeGreaterThanOrEqual(boxes.lyric.x - 2)
  expect(chordCenter).toBeLessThanOrEqual(boxes.lyric.x + boxes.lyric.width + 2)
  expect(boxes.chord.y).toBeLessThan(boxes.lyric.y)
}

Then(
  'the chord {string} is drawn above the bracket {string}',
  async ({ page }, chord: string, bracket: string) =>
    expectChordAbove(page, chord, bracket),
)

Then(
  'the chord {string} is drawn above the lyric {string}',
  async ({ page }, chord: string, lyric: string) =>
    expectChordAbove(page, chord, lyric),
)
