import { expect } from '@playwright/test'
import { Given, Then } from './fixtures'

const SOURCE = [
  'metadata { title "ABC View Test" },',
  '',
  'parts ["Soprano" [S]: notes, "Chords" [C]: chords],',
  '',
  'score """',
  '    (bpm=120 key=D4 time=4/4)',
  '    [S] 1 2 3 4',
  '    [C] 1 - 6m/3 -',
  '',
  "    [S] 5 6 7 1'",
  '    [C] 4M7 - 5b -',
  '    """,',
].join('\n')

Given(
  'a score in D major is loaded for the ABC notation view',
  async ({ page }) => {
    await page.addInitScript((src) => {
      localStorage.setItem(
        'poentaro:files:v1',
        JSON.stringify({
          active: 'abc.poentaro',
          userFiles: { 'abc.poentaro': src },
          bin: {},
          fileIds: { 'abc.poentaro': crypto.randomUUID() },
        }),
      )
    }, SOURCE)
    await page.goto('/')
    await page.waitForSelector('.preview-page', { timeout: 15_000 })
  },
)

Then(
  "the preview's note heads read {string}",
  async ({ page }, expected: string) => {
    await expect(
      page.locator('.preview-page text[data-variant="note-head"]'),
    ).toHaveText(expected.split(', '))
  },
)

Then(
  "the preview's chord symbols read {string}",
  async ({ page }, expected: string) => {
    await expect(
      page.locator('.preview-page text[data-variant="chord-symbol"]'),
    ).toHaveText(expected.split(', '))
  },
)
