import { expect } from '@playwright/test'
import { tagSelector } from '../../../src/dataAttributes'
import { Given, Then, When } from './fixtures'

type State = {
  measureCount: number
  melodyNotes: string
  singingMeasures: number[]
  singing: string
}

let state: State = {
  measureCount: 0,
  melodyNotes: '',
  singingMeasures: [],
  singing: '',
}

Given(
  'parts Melody [M], Verse [V] = lyrics[M] are declared for the lyric-row-rest-run score',
  async () => {
    state = {
      measureCount: 0,
      melodyNotes: '',
      singingMeasures: [],
      singing: '',
    }
  },
)

Given(
  'the lyric-row-rest-run score has {int} measures of Melody notes {string}',
  async ({}, count: number, notes: string) => {
    state.measureCount = count
    state.melodyNotes = notes
  },
)

Given(
  'the Verse line is sung {string} in measures {int} and {int} only',
  async ({}, syllables: string, first: number, second: number) => {
    state.singing = syllables
    state.singingMeasures = [first, second]
  },
)

function buildSource(): string {
  const lines = [
    `metadata { title "lyric row rest run test", max measures per system ${state.measureCount} },`,
    '',
    'parts ["Melody" [M]: notes, "Verse" [V]: lyrics [M]],',
    '',
    'score """',
  ]
  for (let i = 0; i < state.measureCount; i++) {
    lines.push(`[M] ${state.melodyNotes}`)
    if (state.singingMeasures.includes(i)) lines.push(`[V] ${state.singing}`)
    lines.push('')
  }
  return [...lines, '""",'].join('\n')
}

When('the lyric-row-rest-run score is laid out', async ({ page }) => {
  const source = buildSource()
  await page.addInitScript((src) => {
    localStorage.setItem(
      'poentaro:files:v1',
      JSON.stringify({
        active: 'lyric-row-rest-run-test.poentaro',
        userFiles: { 'lyric-row-rest-run-test.poentaro': src },
        bin: {},
        fileIds: {
          'lyric-row-rest-run-test.poentaro': 'lyric-row-rest-run-test-id-001',
        },
      }),
    )
  }, source)

  await page.goto('/')
  await page.waitForSelector('[data-testid="play-measure-button"]', {
    timeout: 15_000,
  })
  await page.waitForSelector(tagSelector('measure'), { timeout: 10_000 })
})

/** The rest bar is identified by shape, as in
 * merged-rest-run-column-width.steps.ts: a thick horizontal `<line>`. */
async function restBarBoxes(page: import('@playwright/test').Page) {
  return page.evaluate(() =>
    Array.from(document.querySelectorAll('svg line'))
      .filter(
        (l) =>
          l.getAttribute('y1') === l.getAttribute('y2') &&
          l.getAttribute('x1') !== l.getAttribute('x2') &&
          Number(l.getAttribute('stroke-width')) > 2,
      )
      .map((l) => {
        const r = l.getBoundingClientRect()
        return { x: r.x, y: r.y, width: r.width, height: r.height }
      }),
  )
}

Then(
  'the lyric row shows one rest bar with the count {string}',
  async ({ page }, count: string) => {
    await expect(page.locator(`svg text:text-is("${count}")`)).toHaveCount(1)
    expect(await restBarBoxes(page)).toHaveLength(1)
  },
)

Then('the lyric row shows no rest bar', async ({ page }) => {
  expect(await restBarBoxes(page)).toHaveLength(0)
})

When('the lyric-row-rest-run Verse part is soloed', async ({ page }) => {
  await page
    .locator('.part-toggle-pill')
    .filter({
      has: page.locator('.part-toggle-abbr', { hasText: /^V$/ }),
    })
    .locator('.part-toggle-segment--headphones')
    .click()
})
