import { expect } from '@playwright/test'
import { Given, Then } from './fixtures'

const OUT_OF_RANGE_SOURCE = [
  'metadata { title "Test" },',
  '',
  'parts ["Melody" [M]: notes { volume: 150 %, octave: + 5 }],',
  '',
  'score """',
  '    (bpm=120 key=C4 time=4/4)',
  '    [M] 1 1 5 5',
  '    """,',
].join('\n')

Given(
  'the edit-parts-modal test fixture with an out-of-range volume and octave offset is loaded',
  async ({ page }) => {
    await page.addInitScript((source) => {
      localStorage.setItem(
        'poentaro:files:v1',
        JSON.stringify({
          active: 'test.poentaro',
          userFiles: { 'test.poentaro': source },
          bin: {},
          fileIds: { 'test.poentaro': crypto.randomUUID() },
        }),
      )
    }, OUT_OF_RANGE_SOURCE)
    await page.goto('/')
  },
)

Then(
  'the editor shows the diagnostic {string}',
  async ({ page }, message: string) => {
    await expect(
      page.locator('.editor-error-zone-message', { hasText: message }),
    ).toBeVisible({ timeout: 30_000 })
  },
)
