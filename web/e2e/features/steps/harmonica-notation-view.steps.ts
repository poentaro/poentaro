import { expect } from '@playwright/test'
import { variantSelector } from '../../../src/dataAttributes'
import { Given, Then } from './fixtures'

// The last measure's `5,,` sits below the harmonica's lowest hole (middle
// C), and its `4#` (F♯) is played with the slide button pressed in.
const SOURCE = [
  'metadata { title "Harmonica View Test" },',
  '',
  'parts ["Melody" [M]: notes],',
  '',
  'score """',
  '    (bpm=120 key=C4 time=4/4)',
  '    [M] 1 2 3 4',
  '',
  "    [M] 5 6 7 1'",
  '',
  "    [M] 5,, 4# 3' 1''",
  '    """,',
].join('\n')

Given(
  'a score in C major is loaded for the harmonica notation view',
  async ({ page }) => {
    await page.addInitScript((src) => {
      localStorage.setItem(
        'poentaro:files:v1',
        JSON.stringify({
          active: 'harmonica.poentaro',
          userFiles: { 'harmonica.poentaro': src },
          bin: {},
          fileIds: { 'harmonica.poentaro': crypto.randomUUID() },
        }),
      )
    }, SOURCE)
    await page.goto('/')
    await page.waitForSelector('.preview-page', { timeout: 15_000 })
  },
)

Then(
  'the preview flags the unplayable note heads {string}',
  async ({ page }, expected: string) => {
    await expect(
      page.locator('.preview-page text[data-variant="unplayable-note-head"]'),
    ).toHaveText(expected.split(' '))
  },
)

/** Each slide arrow is drawn as three strokes: a shaft and its head's two. */
const STROKES_PER_SLIDE_ARROW = 3

Then(
  'the preview draws {int} slide arrow(s)',
  async ({ page }, expected: number) => {
    await expect(
      page.locator(`.preview-page line${variantSelector('harmonica-slide')}`),
    ).toHaveCount(expected * STROKES_PER_SLIDE_ARROW)
  },
)
