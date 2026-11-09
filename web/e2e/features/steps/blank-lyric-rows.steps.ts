import type { Locator, Page } from '@playwright/test'
import { expect } from '@playwright/test'
import { tagFieldAttribute, tagSelector } from '../../../src/dataAttributes'
import { Given, Then, When } from './fixtures'

/** `source_part_index` of each declared part, in declaration order. */
const PART_INDEX = { M: 0, V1: 1, V2: 2, C: 3 } as const
type Abbreviation = keyof typeof PART_INDEX

const PART_ABBREVIATION_BY_NAME: Record<string, Abbreviation> = {
  Melody: 'M',
  'Verse 1': 'V1',
  'Verse 2': 'V2',
  Chords: 'C',
}

type State = {
  hideRestingParts: boolean
  melodyNotes: string
  verse1Syllables: string | null
  verse2Syllables: string | null
  chords: string | null
  /** x of each Verse 1 syllable (by note id) recorded right after layout. */
  verse1XBeforeHiding: Record<string, number>
}

function initialState(): State {
  return {
    hideRestingParts: true,
    melodyNotes: '1 2 3 4',
    verse1Syllables: null,
    verse2Syllables: null,
    chords: null,
    verse1XBeforeHiding: {},
  }
}

let state: State = initialState()

Given(
  'parts Melody [M], Verse 1 [V1] = lyrics[M], Verse 2 [V2] = lyrics[M], Chords [C] are declared',
  async () => {
    state = initialState()
  },
)

Given(
  'the blank-lyric-rows score sets hide_resting_parts to {string}',
  async ({}, value: string) => {
    state.hideRestingParts = value === 'yes'
  },
)

Given(
  "measure {int}'s Melody notes are {string}",
  async ({}, _index: number, notes: string) => {
    state.melodyNotes = notes
  },
)

Given(
  "measure {int}'s Verse 1 line has syllables {string}",
  async ({}, _index: number, syllables: string) => {
    state.verse1Syllables = syllables
  },
)

Given(
  "measure {int}'s Verse 2 line has syllables {string}",
  async ({}, _index: number, syllables: string) => {
    state.verse2Syllables = syllables
  },
)

Given(
  "measure {int}'s Chords line has chords {string}",
  async ({}, _index: number, chords: string) => {
    state.chords = chords
  },
)

Given('measure {int} has no Verse 1 line', async ({}, _index: number) => {
  state.verse1Syllables = null
})

Given('measure {int} has no Verse 2 line', async ({}, _index: number) => {
  state.verse2Syllables = null
})

function buildSource(): string {
  const lines: string[] = [
    `metadata { title "blank lyric rows test", hide resting parts: ${state.hideRestingParts ? 'yes' : 'no'} },`,
    '',
    'parts ["Melody" [M]: notes, "Verse 1" [V1]: lyrics [M], "Verse 2" [V2]: lyrics [M], "Chords" [C]: chords],',
    '',
    'score """',
    `[M] ${state.melodyNotes}`,
  ]
  if (state.verse1Syllables !== null)
    lines.push(`[V1] ${state.verse1Syllables}`)
  if (state.verse2Syllables !== null)
    lines.push(`[V2] ${state.verse2Syllables}`)
  if (state.chords !== null) lines.push(`[C] ${state.chords}`)
  return [...lines, '""",'].join('\n')
}

/** Every drawn `lyric` syllable group, keyed by note id -> left x. Only
 * Verse 1 has syllables wherever this is used. (Not filtered by
 * `data-part-index`: that is a rendered index which shifts when the notes
 * part is hidden.) */
async function verse1SyllableXs(page: Page): Promise<Record<string, number>> {
  const syllables = page.locator(tagSelector('lyric'))
  const noteIdAttr = tagFieldAttribute('lyric', 'noteId')
  const count = await syllables.count()
  const xs: Record<string, number> = {}
  for (let i = 0; i < count; i++) {
    const syllable = syllables.nth(i)
    const noteId = await syllable.getAttribute(noteIdAttr)
    const box = await syllable.boundingBox()
    if (noteId !== null && box) xs[noteId] = box.x
  }
  return xs
}

When('the blank-lyric-rows score is laid out', async ({ page }) => {
  const source = buildSource()
  await page.addInitScript((src) => {
    localStorage.setItem(
      'poentaro:files:v1',
      JSON.stringify({
        active: 'blank-lyric-rows-test.poentaro',
        userFiles: { 'blank-lyric-rows-test.poentaro': src },
        bin: {},
        fileIds: {
          'blank-lyric-rows-test.poentaro': 'blank-lyric-rows-test-id-001',
        },
      }),
    )
  }, source)

  await page.goto('/')
  await page.waitForSelector('[data-testid="play-measure-button"]', {
    timeout: 15_000,
  })
  await page.waitForSelector(tagSelector('measure'), { timeout: 10_000 })
  state.verse1XBeforeHiding = await verse1SyllableXs(page)
})

function partPill(page: Page, abbreviation: string): Locator {
  return page.locator('.part-toggle-pill').filter({
    has: page.locator('.part-toggle-abbr', {
      hasText: new RegExp(`^${abbreviation}$`),
    }),
  })
}

async function toggleSolo(page: Page, abbreviation: Abbreviation) {
  await partPill(page, abbreviation)
    .locator('.part-toggle-segment--headphones')
    .click()
}

const SOLO_FIRST_TWO_THEN_LAST =
  /^(Melody|Verse 1|Verse 2|Chords), (Melody|Verse 1|Verse 2|Chords) and (Melody|Verse 1|Verse 2|Chords) are soloed$/

When(
  SOLO_FIRST_TWO_THEN_LAST,
  async ({ page }, a: string, b: string, c: string) => {
    for (const name of [a, b, c]) {
      await toggleSolo(page, PART_ABBREVIATION_BY_NAME[name] as Abbreviation)
    }
  },
)

When('Verse 1 and Verse 2 are soloed', async ({ page }) => {
  await toggleSolo(page, 'V1')
  await toggleSolo(page, 'V2')
})

When('the Melody part is hidden', async ({ page }) => {
  await partPill(page, 'M').locator('.part-toggle-segment--eye').click()
})

/** The rows drawn for measure `index`, top to bottom, as the label text
 * printed beside each row (the part's abbreviation). A row's label is a
 * transparent click-target `<g>` (tag `part-label` for a notes/chords row,
 * `lyric-label` for a lyric row) whose `data-part-index` is a rendered
 * index that shifts when parts are hidden, so rows are identified by the
 * printed `<text>` that sits inside that click-target's box instead. */
async function rowLabels(page: Page, index: number): Promise<string[]> {
  return page.evaluate(
    ({ index, selectors }) => {
      const labelTexts = Array.from(document.querySelectorAll('svg text'))
        .map((el) => ({
          text: (el.textContent ?? '').trim(),
          box: el.getBoundingClientRect(),
        }))
        .filter((t) => t.text.length > 0)
      return selectors
        .flatMap((selector) =>
          Array.from(document.querySelectorAll(selector.css)).map((el) => ({
            el,
            selector,
          })),
        )
        .filter(({ el, selector }) => {
          const start = Number(el.getAttribute(selector.startAttr))
          const end = Number(el.getAttribute(selector.endAttr))
          return start <= index && index <= end
        })
        .map(({ el }) => el.getBoundingClientRect())
        .sort((a, b) => a.y - b.y)
        .map((rect) => {
          const printed = labelTexts.find((t) => {
            const cx = t.box.x + t.box.width / 2
            const cy = t.box.y + t.box.height / 2
            return (
              cx >= rect.x &&
              cx <= rect.x + rect.width &&
              cy >= rect.y &&
              cy <= rect.y + rect.height
            )
          })
          return printed?.text ?? '?'
        })
    },
    {
      index,
      selectors: (['part-label', 'lyric-label'] as const).map((tag) => ({
        css: tagSelector(tag),
        startAttr: tagFieldAttribute(tag, 'measureIndexStart'),
        endAttr: tagFieldAttribute(tag, 'measureIndexEnd'),
      })),
    },
  )
}

Then(
  /^measure (\d+) has rows (.+)$/,
  async ({ page }, index: string, rowsStr: string) => {
    const expected = rowsStr.split(',').map((s) => s.trim().replace(/"/g, ''))
    await expect.poll(() => rowLabels(page, Number(index))).toEqual(expected)
  },
)

Then(
  'the Verse 1 row in measure {int} shows no syllables',
  async ({ page }, index: number) => {
    expect(await rowLabels(page, index)).toContain('V1')
    // No part is hidden or soloed here, so rendered index == source index.
    await expect(
      page.locator(tagSelector('lyric', { sourcePartIndex: PART_INDEX.V1 })),
    ).toHaveCount(0)
  },
)

Then(
  'the Verse 1 syllables sit at the same columns as before hiding',
  async ({ page }) => {
    const before = state.verse1XBeforeHiding
    expect(Object.keys(before).length).toBeGreaterThan(0)
    // Hiding Melody removes its row; poll until the preview re-renders.
    await expect(page.locator(tagSelector('note'))).toHaveCount(0)
    // Column rods come from every visible row, so hiding the Melody row
    // shifts the columns by ~1px (slack is shared out differently). The
    // lyrics still sit at the notes' columns: same note ids, same
    // left-to-right order, each within TOLERANCE_PX of where it was.
    const TOLERANCE_PX = 2
    const idsLeftToRight = (xs: Record<string, number>) =>
      Object.entries(xs)
        .sort(([, a], [, b]) => a - b)
        .map(([noteId]) => noteId)
    await expect
      .poll(async () => {
        const after = await verse1SyllableXs(page)
        return (
          idsLeftToRight(after).join() === idsLeftToRight(before).join() &&
          Object.entries(before).every(
            ([noteId, x]) =>
              Math.abs((after[noteId] ?? Number.NaN) - x) < TOLERANCE_PX,
          )
        )
      })
      .toBe(true)
  },
)

Then(
  'the row under Melody is labelled {string}',
  async ({ page }, label: string) => {
    const labels = await rowLabels(page, 0)
    const melodyAt = labels.indexOf('M')
    expect(melodyAt).toBeGreaterThanOrEqual(0)
    expect(labels[melodyAt + 1]).toBe(label)
  },
)
