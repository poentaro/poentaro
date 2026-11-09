import { expect } from '@playwright/test'
import {
  ancestorTagXpath,
  rectVariantSelector,
  tagSelector,
} from '../../../src/dataAttributes'
import { Given, Then } from './fixtures'

/**
 * Regression fixture for a bug introduced by the `note_position_in_measure`
 * axis added to the cross-part `Note ↔ Note` arm (see `cross_part` in
 * `crates/poentaro-wasm/src/selection_range/note_note.rs`): that axis's
 * `[position_start, position_end]` bound was applied to *every* measure in
 * the swept range uniformly, not just the boundary measures. So when the
 * anchor and current land in *different* measures, a rest ahead of the
 * later click's note (which pushes that note's own position further into
 * its measure) wrongly truncated the *earlier* measure too, even though
 * the earlier measure should be selected in full — same-line-selection
 * semantics, where an interior/earlier line is taken whole and only the
 * boundary line nearest the far click is partial.
 *
 * Two parts, two measures. Measure 0: each part's first note (position 0)
 * is a real note, followed by three rests (positions 1-3) — a whole
 * measure. Measure 1: each part opens with a rest (position 0), then a
 * real note (position 1), then two more rests (positions 2-3) — so the
 * "position" axis no longer lines up with "the first real note" once
 * rests are involved.
 *
 * Anchoring on Melody's measure-0 note (index 0, position 0) and
 * committing on Harmony's measure-1 note (index 13, position 1) must
 * select measure 0 in full (indices 0-7, all four positions of both
 * parts) and only positions 0-1 of measure 1 (indices 8, 9, 12, 13),
 * leaving measure 1's positions 2-3 (indices 10, 11, 14, 15) unselected.
 */
const crossMeasureRestSource = [
  'metadata { title "cross measure rest range test" },',
  '',
  'parts ["Melody" [M]: notes, "Harmony" [H]: notes],',
  '',
  'score """',
  '[M] 1 0 0 0',
  '[H] 2 0 0 0',
  '',
  '[M] 0 3 0 0',
  '[H] 0 4 0 0',
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
  'the cross-measure-rest range-selection fixture is loaded and note click targets have rendered',
  async ({ page }) => {
    await page.addInitScript((source) => {
      localStorage.setItem(
        'poentaro:files:v1',
        JSON.stringify({
          active: 'cross-measure-rest-range-test.poentaro',
          userFiles: {
            'cross-measure-rest-range-test.poentaro': source,
          },
          bin: {},
          fileIds: {
            'cross-measure-rest-range-test.poentaro':
              'cross-measure-rest-range-test-id-001',
          },
        }),
      )
    }, crossMeasureRestSource)
    await page.goto('/')

    await page.waitForSelector('[data-testid="play-measure-button"]', {
      timeout: 15_000,
    })
    await page.waitForSelector(tagSelector('measure', { index: 1 }), {
      timeout: 10_000,
    })
    await expect(noteRects(page)).toHaveCount(16, { timeout: 10_000 })
  },
)

// Reuses the "I click-and-click select the note at index {int} then the
// note at index {int}" step defined in
// `note-range-select-crosses-system.steps.ts` — playwright-bdd resolves
// step text against one project-wide registry, so redefining the identical
// pattern here would collide with it.

Then(
  'the notes at indexes {string} are all range-selected',
  async ({ page }, indexes: string) => {
    for (const noteId of indexes.split(',').map(Number)) {
      await expect(noteGroup(page, noteId)).toHaveAttribute(
        'data-note-range-selected',
        '',
      )
    }
  },
)

Then(
  'the notes at indexes {string} are not range-selected',
  async ({ page }, indexes: string) => {
    for (const noteId of indexes.split(',').map(Number)) {
      await expect(noteGroup(page, noteId)).not.toHaveAttribute(
        'data-note-range-selected',
        '',
      )
    }
  },
)
