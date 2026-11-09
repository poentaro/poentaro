import { expect } from '@playwright/test'
import { rectVariantSelector, tagSelector } from '../../../src/dataAttributes'
import {
  clickAndClickSelect,
  stableBoundingBox,
} from '../../rangeSelectHelpers'
import { Given, Then, When } from './fixtures'

/**
 * Regression fixture for the cross-part `Note ↔ Lyric` click-and-click
 * range gesture when both clicks land in the *same* measure — the
 * `Note ↔ Lyric` counterpart to
 * `note-range-select-crosses-part-within-measure.feature`'s `Note ↔ Note`
 * case. `cross_part` in
 * `crates/poentaro-wasm/src/selection_range/note_lyric.rs` still ranges only
 * by `measure_index`, so two clicks landing in the same measure on
 * different parts (e.g. note 0 of `1 2 3`/"do re mi" on one part, syllable
 * 1 of `4 5 6`/"fa so la" on another) wrongly selects that whole measure's
 * notes and syllables on both sides, regardless of where in the measure
 * either click actually landed.
 *
 * Two parts, three notes and three syllables each, all in measure 0.
 * Anchoring on Melody's first note (position 0) and committing on
 * Harmony's second syllable (position 1) should select positions 0-1 in
 * both parts/rows — Melody's notes 0-1 and syllables "do"/"re", Harmony's
 * notes 0-1 and syllables "fa"/"so" — leaving each part's third note/
 * syllable (position 2) unselected.
 */
const source = [
  'metadata { title "cross part within measure note lyric range test", max measures per system 48 },',
  '',
  'parts [',
  '    "Melody" [M]: notes,',
  '    "Melody lyrics" [Mv1]: lyrics [M],',
  '    "Harmony" [H]: notes,',
  '    "Harmony lyrics" [Hv1]: lyrics [H],',
  '],',
  '',
  'score """',
  '[M] 1 2 3',
  '[Mv1] do re mi',
  '[H] 4 5 6',
  '[Hv1] fa so la',
  '""",',
].join('\n')

// A note renders as two sibling `[data-tag="note"]` groups sharing the same
// `data-part-index`/`data-note-id` — one wrapping the (pointer-events: none)
// playback-cursor rect, one wrapping the click-target rect (see
// `applyPersistedNoteHighlights`'s doc comment in
// `previewRangeHighlights.ts`) — and only the click-target group ever gets
// marked `data-note-range-selected`. Scope to the one that `:has()` the
// click-target rect so the locator resolves to exactly one element.
function noteAt(
  page: import('@playwright/test').Page,
  partIndex: number,
  noteId: number,
) {
  return page.locator(
    `${tagSelector('note', { sourcePartIndex: partIndex, noteId })}:has(${rectVariantSelector('note-click-target')})`,
  )
}

function noteClickTarget(
  page: import('@playwright/test').Page,
  partIndex: number,
  noteId: number,
) {
  return noteAt(page, partIndex, noteId).locator(
    rectVariantSelector('note-click-target'),
  )
}

function lyricAt(
  page: import('@playwright/test').Page,
  partIndex: number,
  noteId: number,
) {
  return page.locator(
    tagSelector('lyric', { sourcePartIndex: partIndex, noteId }),
  )
}

Given(
  'the cross-part-within-measure note-lyric range-selection fixture is loaded and both rows have rendered',
  async ({ page }) => {
    await page.addInitScript((src) => {
      localStorage.setItem(
        'poentaro:files:v1',
        JSON.stringify({
          active: 'cross-part-within-measure-note-lyric-range-test.poentaro',
          userFiles: {
            'cross-part-within-measure-note-lyric-range-test.poentaro': src,
          },
          bin: {},
          fileIds: {
            'cross-part-within-measure-note-lyric-range-test.poentaro':
              'cross-part-within-measure-note-lyric-range-test-id-001',
          },
        }),
      )
    }, source)
    await page.goto('/')

    await page.waitForSelector('[data-testid="play-measure-button"]', {
      timeout: 15_000,
    })
    await page.waitForSelector(tagSelector('measure', { index: 0 }), {
      timeout: 10_000,
    })
    await expect(
      page.locator(rectVariantSelector('note-click-target')),
    ).toHaveCount(6, { timeout: 10_000 })
    await expect(page.locator(tagSelector('lyric'))).toHaveCount(6, {
      timeout: 10_000,
    })
    await page.evaluate(() => document.fonts.ready)
    await page.waitForTimeout(200)
  },
)

When(
  "I click-and-click select Melody's note {int} then Harmony's lyric syllable {int}",
  async ({ page }, noteId: number, lyricNoteId: number) => {
    const noteBox = await stableBoundingBox(noteClickTarget(page, 0, noteId))
    const lyricBox = await stableBoundingBox(lyricAt(page, 3, lyricNoteId))
    if (!noteBox || !lyricBox) {
      throw new Error(
        `Could not get bounding boxes for Melody note ${noteId} and Harmony lyric syllable ${lyricNoteId}.`,
      )
    }

    await clickAndClickSelect(
      page,
      noteBox.x + noteBox.width / 2,
      noteBox.y + noteBox.height / 2,
      lyricBox.x + lyricBox.width / 2,
      lyricBox.y + lyricBox.height / 2,
    )
  },
)

Then(
  "Melody's and Harmony's notes at position {int} and {int} are range-selected",
  async ({ page }, a: number, b: number) => {
    for (const partIndex of [0, 2]) {
      for (const noteId of [a, b]) {
        await expect(noteAt(page, partIndex, noteId)).toHaveAttribute(
          'data-note-range-selected',
          '',
        )
      }
    }
  },
)

Then(
  "Melody's and Harmony's lyric syllables at position {int} and {int} are range-selected",
  async ({ page }, a: number, b: number) => {
    for (const partIndex of [1, 3]) {
      for (const noteId of [a, b]) {
        await expect(lyricAt(page, partIndex, noteId)).toHaveAttribute(
          'data-lyric-range-selected',
          '',
        )
      }
    }
  },
)

Then(
  "Melody's and Harmony's notes at position {int} are not range-selected",
  async ({ page }, noteId: number) => {
    for (const partIndex of [0, 2]) {
      await expect(noteAt(page, partIndex, noteId)).not.toHaveAttribute(
        'data-note-range-selected',
        '',
      )
    }
  },
)

Then(
  "Melody's and Harmony's lyric syllables at position {int} are not range-selected",
  async ({ page }, noteId: number) => {
    for (const partIndex of [1, 3]) {
      await expect(lyricAt(page, partIndex, noteId)).not.toHaveAttribute(
        'data-lyric-range-selected',
        '',
      )
    }
  },
)
