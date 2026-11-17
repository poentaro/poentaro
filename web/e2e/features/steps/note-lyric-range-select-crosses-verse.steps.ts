import { expect } from '@playwright/test'
import { rectVariantSelector, tagSelector } from '../../../src/dataAttributes'
import {
  clickAndClickSelect,
  stableBoundingBox,
} from '../../rangeSelectHelpers'
import { Given, Then, When } from './fixtures'

/**
 * Regression fixture for the same-part `Note ↔ Lyric` click-and-click range
 * gesture when the second click lands on a syllable in a verse *other than*
 * verse 0 — reported live: `[a] 1 2 3 4` with two lyric lines, clicking note
 * "1" then the second verse's third syllable ("g") selected notes 1-3 and
 * only the second verse's syllables, silently dropping the first verse's
 * syllables it visually swept over on the way there.
 *
 * `note_lyric::same_part` (see `selection_range/note_lyric.rs`) used to
 * restrict `lyric_cells` to the `Lyric` endpoint's own verse alone. But a
 * `Note` endpoint has no verse of its own — the note row renders above every
 * verse row — so a sweep from it down to verse `V` always visually crosses
 * verses `0` through `V`, not just row `V` in isolation. This fixture has
 * two verse rows on one measure of four notes, so verse 0's syllables at the
 * swept note-id range must show up as selected too, while each verse's
 * fourth syllable (note id 3, outside the swept range) stays unselected as a
 * distractor proving the fix ranges by `note_id` alongside `verse`, not just
 * unioning every verse wholesale.
 */
const source = [
  'metadata { title "note lyric cross verse range test", max measures per system 48 },',
  '',
  'parts ["Melody" [M]: notes, "Melody verse 1" [Mv1]: lyrics [M], "Melody verse 2" [Mv2]: lyrics [M]],',
  '',
  'score """',
  '[M] 1 2 3 4',
  '[Mv1] a b c d',
  '[Mv2] e f g h',
  '""",',
].join('\n')

function noteClickTarget(
  page: import('@playwright/test').Page,
  noteId: number,
) {
  return page
    .locator(
      `${tagSelector('note', { sourcePartIndex: 0, noteId })}:has(${rectVariantSelector('note-click-target')})`,
    )
    .locator(rectVariantSelector('note-click-target'))
}

function noteAt(page: import('@playwright/test').Page, noteId: number) {
  return page.locator(
    `${tagSelector('note', { sourcePartIndex: 0, noteId })}:has(${rectVariantSelector('note-click-target')})`,
  )
}

function lyricAt(
  page: import('@playwright/test').Page,
  verse: number,
  noteId: number,
) {
  return page.locator(
    tagSelector('lyric', { sourcePartIndex: verse + 1, noteId }),
  )
}

Given(
  'the note-lyric cross-verse range-selection fixture is loaded and both verses have rendered',
  async ({ page }) => {
    await page.addInitScript((src) => {
      localStorage.setItem(
        'poentaro:files:v1',
        JSON.stringify({
          active: 'note-lyric-cross-verse-range-test.poentaro',
          userFiles: { 'note-lyric-cross-verse-range-test.poentaro': src },
          bin: {},
          fileIds: {
            'note-lyric-cross-verse-range-test.poentaro':
              'note-lyric-cross-verse-range-test-id-001',
          },
        }),
      )
    }, source)
    await page.goto('/')

    await page.waitForSelector('[data-testid="play-measure-button"]', {
      timeout: 15_000,
    })
    await expect(
      page.locator(rectVariantSelector('note-click-target')),
    ).toHaveCount(4, { timeout: 10_000 })
    await expect(page.locator(tagSelector('lyric'))).toHaveCount(8, {
      timeout: 10_000,
    })
    await page.evaluate(() => document.fonts.ready)
    await page.waitForTimeout(200)
  },
)

When(
  "I click-and-click select Melody's note {int} then verse {int}'s syllable {int}",
  async ({ page }, noteId: number, verse: number, lyricNoteId: number) => {
    const noteBox = await stableBoundingBox(noteClickTarget(page, noteId))
    const lyricBox = await stableBoundingBox(lyricAt(page, verse, lyricNoteId))
    if (!noteBox || !lyricBox) {
      throw new Error(
        `Could not get bounding boxes for Melody note ${noteId} and verse ${verse}'s syllable ${lyricNoteId}.`,
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
  "Melody's notes {int} through {int} are range-selected",
  async ({ page }, start: number, end: number) => {
    for (let noteId = start; noteId <= end; noteId++) {
      await expect(noteAt(page, noteId)).toHaveAttribute(
        'data-note-range-selected',
        '',
      )
    }
  },
)

Then(
  "verse {int}'s syllables {int} through {int} are range-selected",
  async ({ page }, verse: number, start: number, end: number) => {
    for (let noteId = start; noteId <= end; noteId++) {
      await expect(lyricAt(page, verse, noteId)).toHaveAttribute(
        'data-lyric-range-selected',
        '',
      )
    }
  },
)

Then(
  "verse {int}'s syllable {int} is not range-selected",
  async ({ page }, verse: number, noteId: number) => {
    await expect(lyricAt(page, verse, noteId)).not.toHaveAttribute(
      'data-lyric-range-selected',
      '',
    )
  },
)
