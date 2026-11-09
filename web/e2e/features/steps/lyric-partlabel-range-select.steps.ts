import { expect } from '@playwright/test'
import { rectVariantSelector, tagSelector } from '../../../src/dataAttributes'
import { clickThenStableClick } from '../../rangeSelectHelpers'
import { Given, Then, When } from './fixtures'

/**
 * Regression fixture for the click-and-click *lyric-anchored* gesture
 * resolving against a *part-label* `current` — one of the five label-mixed
 * combinations `PLAN-clickable-element-id-selection.md`'s Follow-up section
 * left open, now closed (see `resolve_selection_range_response`'s
 * `Lyric ↔ PartLabel` arm, backed by `resolve_note_like_lyric_like_range`,
 * in `selection_range.rs`).
 *
 * The `PartLabel` side always contributes to `note_cells` (already a
 * `[start, end]` span, no lookup needed); the `Lyric` side always
 * contributes to `lyric_cells`, restricted to its own `verse` — proved here
 * by Melody carrying two verses, only one of which (the syllable's own) ends
 * up selected. Unlike `Note ↔ PartLabel`, this is never gated by
 * part-match: the `Lyric` endpoint is a real syllable, not a duplicate of
 * the part-contributing side. `break` isolates each measure into its own
 * system; measure 2 is a distractor, outside the swept range.
 */
const source = [
  'metadata { title "lyric partlabel range test", max measures per system 48 },',
  '',
  'parts [',
  '    "Melody" [M]: notes,',
  '    "Melody verse 1" [Mv1]: lyrics [M],',
  '    "Melody verse 2" [Mv2]: lyrics [M],',
  '    "Harmony" [H]: notes,',
  '],',
  '',
  'score """',
  '[M] 1',
  '[Mv1] do',
  '[Mv2] fa',
  '[H] 5',
  '',
  'break',
  '[M] 2',
  '[Mv1] re',
  '[Mv2] sol',
  '[H] 6',
  '',
  'break',
  '[M] 3',
  '[Mv1] mi',
  '[Mv2] la',
  '[H] 7',
  '""",',
].join('\n')

// A note group carries a sibling `Tag::Note` group for its
// (pointer-events: none) playback-cursor rect alongside its own
// click-target rect, so a bare `[data-tag="note"]` query double-counts each
// note — filter to the click-target rect's own group, mirroring
// `note-range-select-crosses-part.steps.ts`'s `noteGroup` convention.
function noteInPart(page: import('@playwright/test').Page, partIndex: number) {
  return page
    .locator(tagSelector('note', { sourcePartIndex: partIndex }))
    .filter({
      has: page.locator(rectVariantSelector('note-click-target')),
    })
}

function lyricInPart(page: import('@playwright/test').Page, partIndex: number) {
  return page.locator(tagSelector('lyric', { sourcePartIndex: partIndex }))
}

function partLabelInSystem(
  page: import('@playwright/test').Page,
  partIndex: number,
  measureIndexStart: number,
) {
  return page.locator(
    tagSelector('part-label', {
      sourcePartIndex: partIndex,
      measureIndexStart,
    }),
  )
}

Given(
  'the lyric-partlabel range-selection fixture is loaded and both parts have rendered',
  async ({ page }) => {
    await page.addInitScript((src) => {
      localStorage.setItem(
        'poentaro:files:v1',
        JSON.stringify({
          active: 'lyric-partlabel-range-test.poentaro',
          userFiles: { 'lyric-partlabel-range-test.poentaro': src },
          bin: {},
          fileIds: {
            'lyric-partlabel-range-test.poentaro':
              'lyric-partlabel-range-test-id-001',
          },
        }),
      )
    }, source)
    await page.goto('/')

    await page.waitForSelector('[data-testid="play-measure-button"]', {
      timeout: 15_000,
    })
    await expect(noteInPart(page, 0)).toHaveCount(3, { timeout: 10_000 })
    await expect(noteInPart(page, 3)).toHaveCount(3, { timeout: 10_000 })
    await expect(lyricInPart(page, 1)).toHaveCount(3, { timeout: 10_000 })
    await expect(lyricInPart(page, 2)).toHaveCount(3, { timeout: 10_000 })
    await page.waitForSelector(
      tagSelector('part-label', { sourcePartIndex: 3, measureIndexStart: 1 }),
      { timeout: 10_000 },
    )
    await page.evaluate(() => document.fonts.ready)
    await page.waitForTimeout(200)
  },
)

When(
  "I click-and-click select Melody's verse-0 syllable in measure 0 then Harmony's label in system 1",
  async ({ page }) => {
    const fromLyric = lyricInPart(page, 1).nth(0)
    const toLabel = partLabelInSystem(page, 3, 1)
    await expect(fromLyric).toBeVisible({ timeout: 5_000 })
    await expect(toLabel).toBeVisible({ timeout: 5_000 })

    await clickThenStableClick(page, fromLyric, toLabel)
  },
)

Then(
  '{int} notes are range-selected in total, as seen in lyric partlabel range select',
  async ({ page }, count: number) => {
    await expect(
      page.locator(`${tagSelector('note')}[data-note-range-selected]`),
    ).toHaveCount(count)
  },
)

Then(
  '{int} syllables are range-selected in total, as seen in lyric partlabel range select',
  async ({ page }, count: number) => {
    // The sweep covers the rows from Melody's verse 1 down to Harmony's
    // label, so both of Melody's lyric parts are selected in the swept
    // measures.
    const selected = page.locator(
      `${tagSelector('lyric')}[data-lyric-range-selected]`,
    )
    await expect(selected).toHaveCount(count)
    await expect(
      page.locator(
        `${tagSelector('lyric', { sourcePartIndex: 2 })}[data-lyric-range-selected]`,
      ),
    ).toHaveCount(2)
  },
)

Then(
  'no note in measure 2 is range-selected, as seen in lyric partlabel range select',
  async ({ page }) => {
    for (const partIndex of [0, 3]) {
      await expect(noteInPart(page, partIndex).nth(2)).not.toHaveAttribute(
        'data-note-range-selected',
        '',
      )
    }
  },
)
