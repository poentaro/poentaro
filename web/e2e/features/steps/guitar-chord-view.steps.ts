import { expect } from '@playwright/test'
import { tagFieldAttribute, tagSelector } from '../../../src/dataAttributes'
import { Given, Then } from './fixtures'

const SOURCE = [
  'metadata { title "Guitar View Test" },',
  '',
  'parts ["Soprano" [S]: notes, "Chords" [C]: chords],',
  '',
  'score """',
  '    (bpm=120 key=C4 time=4/4)',
  '    [S] 1 2 3 4',
  '    [C] 1 6m 4 5',
  '    """,',
].join('\n')

const diagrams = '.preview-page'

Given(
  'a chord progression in C major is loaded for the guitar chord view',
  async ({ page }) => {
    await page.addInitScript((src) => {
      localStorage.setItem(
        'poentaro:files:v1',
        JSON.stringify({
          active: 'guitar.poentaro',
          userFiles: { 'guitar.poentaro': src },
          bin: {},
          fileIds: { 'guitar.poentaro': crypto.randomUUID() },
        }),
      )
    }, SOURCE)
    await page.goto('/')
    await page.waitForSelector('.preview-page', { timeout: 15_000 })
  },
)

Then('the preview shows no guitar chord diagrams', async ({ page }) => {
  await expect(
    page.locator(`${diagrams} ${tagSelector('guitar-diagram')}`),
  ).toHaveCount(0)
})

Then(
  "the preview's guitar chord diagrams read {string}",
  async ({ page }, expected: string) => {
    const groups = page.locator(`${diagrams} ${tagSelector('guitar-diagram')}`)
    const expectedFrets = expected.split(', ')
    await expect(groups).toHaveCount(expectedFrets.length)
    const frets = await groups.evaluateAll(
      (elements, attribute) =>
        elements.map((element) => element.getAttribute(attribute)),
      tagFieldAttribute('guitar-diagram', 'frets'),
    )
    expect(frets).toEqual(expectedFrets)
  },
)
