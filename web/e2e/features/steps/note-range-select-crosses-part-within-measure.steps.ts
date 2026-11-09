import { expect } from '@playwright/test'
import {
  ancestorTagXpath,
  rectVariantSelector,
  tagSelector,
} from '../../../src/dataAttributes'
import { Given, Then } from './fixtures'

/**
 * Regression fixture for the cross-part `Note ↔ Note` click-and-click
 * range gesture when both clicks land in the *same* measure (see
 * `cross_part` in `crates/poentaro-wasm/src/selection_range/note_note.rs`):
 * ranging by `measure_index` alone made two same-measure clicks on
 * different parts select the whole measure on both sides regardless of
 * where in the measure either click landed. Fixed by adding a second axis,
 * each note's own position within its `(part, measure)` group
 * (`note_position_in_measure`), so the range stops at that position on
 * both sides instead of sweeping the full measure.
 *
 * Two parts, three notes each, all in measure 0 — anchoring on Melody's
 * first note (index 0, position 0) and committing on Harmony's second note
 * (index 4, position 1) should select positions 0-1 in both parts (indices
 * 0, 1, 3, 4), leaving each part's third note (indices 2, 5) unselected.
 */
const crossPartWithinMeasureSource = [
  'metadata { title "cross part within measure range test" },',
  '',
  'parts ["Melody" [M]: notes, "Harmony" [H]: notes],',
  '',
  'score """',
  '[M] 1 2 3',
  '[H] 4 5 6',
  '""",',
].join('\n')

function noteRects(page: import('@playwright/test').Page) {
  return page.locator(rectVariantSelector('note-click-target'))
}

// `data-note-id` restarts at 0 within each part, so it can't disambiguate
// across parts on its own — walk up from the nth click-target rect (in
// render order) to its own `[data-tag="note"]` group instead, the same
// "index" ordering `noteRects(page).nth(...)` already uses to pick the
// click point (mirrors `note-range-select-crosses-part.steps.ts`).
function noteGroup(page: import('@playwright/test').Page, index: number) {
  return noteRects(page).nth(index).locator(ancestorTagXpath('note'))
}

Given(
  'the cross-part-within-measure range-selection fixture is loaded and note click targets have rendered',
  async ({ page }) => {
    await page.addInitScript((source) => {
      localStorage.setItem(
        'poentaro:files:v1',
        JSON.stringify({
          active: 'cross-part-within-measure-range-test.poentaro',
          userFiles: {
            'cross-part-within-measure-range-test.poentaro': source,
          },
          bin: {},
          fileIds: {
            'cross-part-within-measure-range-test.poentaro':
              'cross-part-within-measure-range-test-id-001',
          },
        }),
      )
    }, crossPartWithinMeasureSource)
    await page.goto('/')

    await page.waitForSelector('[data-testid="play-measure-button"]', {
      timeout: 15_000,
    })
    await page.waitForSelector(tagSelector('measure', { index: 0 }), {
      timeout: 10_000,
    })
    await expect(noteRects(page)).toHaveCount(6, { timeout: 10_000 })
  },
)

// Reuses the "I click-and-click select the note at index {int} then the
// note at index {int}" step defined in
// `note-range-select-crosses-system.steps.ts` and the "the notes at index
// {int}, {int}, {int} and {int} are all range-selected" Then step defined
// in `note-range-select-crosses-part.steps.ts` — playwright-bdd resolves
// step text against one project-wide registry, so redefining either
// identical pattern here would collide with it.

Then(
  'the notes at index {int} and {int} are not range-selected',
  async ({ page }, a: number, b: number) => {
    for (const noteId of [a, b]) {
      await expect(noteGroup(page, noteId)).not.toHaveAttribute(
        'data-note-range-selected',
        '',
      )
    }
  },
)
