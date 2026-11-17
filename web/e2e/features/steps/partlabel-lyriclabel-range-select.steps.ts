import { expect } from '@playwright/test'
import { rectVariantSelector, tagSelector } from '../../../src/dataAttributes'
import { clickThenStableClick } from '../../rangeSelectHelpers'
import { Given, Then, When } from './fixtures'

/**
 * Regression fixture for the click-and-click *part-label-anchored* gesture
 * resolving against a *lyric-label* `current` — the last of the five
 * label-mixed combinations `PLAN-clickable-element-id-selection.md`'s
 * Follow-up section left open, now closed (see
 * `resolve_selection_range_response`'s `PartLabel ↔ LyricLabel` arm, backed
 * by `resolve_note_like_lyric_like_range`, in `selection_range.rs`). Not a
 * sixth special case — the same shape as `Note ↔ LyricLabel`/`Lyric ↔
 * PartLabel` with both sides already being labels, so neither side needs a
 * `note_spans`/`lyric_spans` lookup at all.
 *
 * The `PartLabel` side always contributes to `note_cells`; the
 * `LyricLabel` side always contributes to `lyric_cells`, restricted to its
 * own `verse` — proved here by Harmony carrying two verses, only one of
 * which (the label's own) ends up selected. `break` isolates each measure
 * into its own system; measure 2 is a distractor, outside the swept range.
 */
const source = [
  'metadata { title "partlabel lyriclabel range test", max measures per system 48 },',
  '',
  'parts [',
  '    "Melody" [M]: notes,',
  '    "Harmony" [H]: notes,',
  '    "Harmony verse 1" [Hv1]: lyrics [H],',
  '    "Harmony verse 2" [Hv2]: lyrics [H],',
  '],',
  '',
  'score """',
  '[M] 1',
  '[H] 5',
  '[Hv1] do',
  '[Hv2] fa',
  '',
  'break',
  '[M] 2',
  '[H] 6',
  '[Hv1] re',
  '[Hv2] sol',
  '',
  'break',
  '[M] 3',
  '[H] 7',
  '[Hv1] mi',
  '[Hv2] la',
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

function lyricLabelInSystem(
  page: import('@playwright/test').Page,
  partIndex: number,
  measureIndexStart: number,
) {
  return page.locator(
    tagSelector('lyric-label', {
      sourcePartIndex: partIndex,
      measureIndexStart,
    }),
  )
}

Given(
  'the partlabel-lyriclabel range-selection fixture is loaded and both parts have rendered',
  async ({ page }) => {
    await page.addInitScript((src) => {
      localStorage.setItem(
        'poentaro:files:v1',
        JSON.stringify({
          active: 'partlabel-lyriclabel-range-test.poentaro',
          userFiles: { 'partlabel-lyriclabel-range-test.poentaro': src },
          bin: {},
          fileIds: {
            'partlabel-lyriclabel-range-test.poentaro':
              'partlabel-lyriclabel-range-test-id-001',
          },
        }),
      )
    }, source)
    await page.goto('/')

    await page.waitForSelector('[data-testid="play-measure-button"]', {
      timeout: 15_000,
    })
    await expect(noteInPart(page, 0)).toHaveCount(3, { timeout: 10_000 })
    await expect(noteInPart(page, 1)).toHaveCount(3, { timeout: 10_000 })
    await page.waitForSelector(
      tagSelector('part-label', { sourcePartIndex: 0, measureIndexStart: 0 }),
      { timeout: 10_000 },
    )
    await page.waitForSelector(
      tagSelector('lyric-label', {
        sourcePartIndex: 2,
        measureIndexStart: 1,
      }),
      { timeout: 10_000 },
    )
    await page.evaluate(() => document.fonts.ready)
    await page.waitForTimeout(200)
  },
)

When(
  "I click-and-click select Melody's label in system 0 then Harmony's verse label in system 1",
  async ({ page }) => {
    const fromLabel = partLabelInSystem(page, 0, 0)
    const toLabel = lyricLabelInSystem(page, 2, 1)
    await expect(fromLabel).toBeVisible({ timeout: 5_000 })
    await expect(toLabel).toBeVisible({ timeout: 5_000 })

    await clickThenStableClick(page, fromLabel, toLabel)
  },
)

Then(
  '{int} notes are range-selected in total, as seen in partlabel lyriclabel range select',
  async ({ page }, count: number) => {
    await expect(
      page.locator(`${tagSelector('note')}[data-note-range-selected]`),
    ).toHaveCount(count)
  },
)

Then(
  '{int} syllables are range-selected in total, as seen in partlabel lyriclabel range select',
  async ({ page }, count: number) => {
    // Only Harmony's first lyric part (the label's own) lies in the swept
    // rows; its second lyric part sits below the label.
    const selected = page.locator(
      `${tagSelector('lyric')}[data-lyric-range-selected]`,
    )
    await expect(selected).toHaveCount(count)
    await expect(
      page.locator(
        `${tagSelector('lyric', { sourcePartIndex: 3 })}[data-lyric-range-selected]`,
      ),
    ).toHaveCount(0)
  },
)

Then(
  'no note in measure 2 is range-selected, as seen in partlabel lyriclabel range select',
  async ({ page }) => {
    for (const partIndex of [0, 1]) {
      await expect(noteInPart(page, partIndex).nth(2)).not.toHaveAttribute(
        'data-note-range-selected',
        '',
      )
    }
  },
)
