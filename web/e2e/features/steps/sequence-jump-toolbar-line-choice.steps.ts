import { expect } from '@playwright/test'
import { Given } from './fixtures'

/** The second `Verse` sings the second lyric line (`WITH [w] 2`). */
const source = [
  'metadata { title "test" },',
  '',
  'parts ["Melody" [M]: notes, "Words" [w]: lyrics [M]],',
  '',
  'sequence [Verse, Verse WITH [w] 2],',
  '',
  'score """',
  '    time=4/4 key=C4 bpm=120 label="Verse"',
  '    [M] 1 2 3 4',
  '    [w] 一 二 三 四',
  '    [w] 五 六 七 八',
  '    """,',
].join('\n')

Given(
  'a score whose sequence is {string} is loaded',
  async ({ page }, _sequence: string) => {
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
    await expect(
      page
        .locator('[role="toolbar"]')
        .nth(1)
        .locator('button.section-jump-btn'),
    ).toHaveCount(2, { timeout: 15_000 })
  },
)
