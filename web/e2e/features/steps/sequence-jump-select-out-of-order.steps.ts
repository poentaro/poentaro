import { expect } from '@playwright/test'
import { waitForAssetLoadingBannerGone } from '../../assetLoadingHelpers'
import { Given, Then, When } from './fixtures'

const SOURCE = [
  'metadata { title "test" },',
  '',
  'parts ["M": notes],',
  '',
  'sequence ["C", "A"],',
  '',
  'score """',
  '    time=4/4 key=C4 bpm=120 label="A"',
  '    1 2 3 4',
  '',
  '    label="B"',
  "    5 6 7 1'",
  '',
  '    label="C"',
  "    1' 7 6 5",
  '    """,',
].join('\n')

async function getEditorSelections(page: import('@playwright/test').Page) {
  return page.evaluate(() => {
    const monacoApi = (
      window as unknown as { monaco: typeof import('monaco-editor') }
    ).monaco
    const selections = monacoApi.editor.getEditors()[0]?.getSelections() ?? []
    return selections.map((s) => ({
      startLineNumber: s.startLineNumber,
      endLineNumber: s.endLineNumber,
    }))
  })
}

// Both toolbars render `button.section-jump-btn` inside their own
// `[role="toolbar"]`; SequenceJumpToolbar mounts second.
function sequenceToolbarButtons(page: import('@playwright/test').Page) {
  return page
    .locator('[role="toolbar"]')
    .nth(1)
    .locator('button.section-jump-btn')
}

Given(
  'a sequence chain {string} over sections {string}, {string}, {string} is seeded for out-of-order selection',
  async (
    { page },
    _chain: string,
    _sectionA: string,
    _sectionB: string,
    _sectionC: string,
  ) => {
    await page.addInitScript((src) => {
      localStorage.setItem(
        'poentaro:files:v1',
        JSON.stringify({
          active: 'sequence-out-of-order-test.poentaro',
          userFiles: { 'sequence-out-of-order-test.poentaro': src },
          bin: {},
          fileIds: {
            'sequence-out-of-order-test.poentaro': crypto.randomUUID(),
          },
        }),
      )
    }, SOURCE)
  },
)

When(
  'the app loads with the out-of-order sequence toolbar ready',
  async ({ page }) => {
    await page.goto('/')
    await expect(sequenceToolbarButtons(page)).toHaveCount(2, {
      timeout: 15_000,
    })
    await waitForAssetLoadingBannerGone(page)
  },
)

When(
  'I drag from the {string} sequence entry to the {string} sequence entry',
  async ({ page }, _from: string, _to: string) => {
    const buttons = sequenceToolbarButtons(page)

    // Drag from the "C" button (index 0, chain-first) to the "A" button
    // (index 1, chain-last) — chain order disagrees with document order here.
    await buttons.nth(0).hover()
    await page.mouse.down()
    await buttons.nth(1).hover()
    await page.mouse.up()
  },
)

Then(
  'the selected measure range shows {string}',
  async ({ page }, range: string) => {
    await expect(page.getByTestId('selected-measure-range')).toHaveText(range, {
      timeout: 3_000,
    })
  },
)

Then(
  'the editor selects exactly {string} and {string} as disjoint ranges, excluding {string}',
  async ({ page }, _first: string, _second: string, _excluded: string) => {
    await expect
      .poll(() => getEditorSelections(page), { timeout: 3_000 })
      .toEqual([
        { startLineNumber: 14, endLineNumber: 15 }, // "C" only
        { startLineNumber: 8, endLineNumber: 9 }, // "A" only
      ])
  },
)
