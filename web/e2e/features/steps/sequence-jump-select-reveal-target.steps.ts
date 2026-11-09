import { expect } from '@playwright/test'
import { waitForAssetLoadingBannerGone } from '../../assetLoadingHelpers'
import { Given, Then, When } from './fixtures'

// Padded with filler measures so "Intro" (lines 11-12) and "C" (near the
// bottom) can't both land in the same viewport.
const fillerMeasures = Array.from(
  { length: 60 },
  (_, i) => `label="Filler${i}"\n1 2 3 4`,
).join('\n\n')

const SOURCE = [
  'metadata { title "test" },',
  '',
  'parts ["M": notes],',
  '',
  'sequence [Intro, "C", Intro],',
  '',
  'score """',
  'time=4/4 key=C4 bpm=120 label="Intro"',
  '1 2 3 4',
  '',
  fillerMeasures,
  '',
  'label="C"',
  "1' 7 6 5",
  '""",',
].join('\n')

function sequenceToolbarButtons(page: import('@playwright/test').Page) {
  return page
    .locator('[role="toolbar"]')
    .nth(1)
    .locator('button.section-jump-btn')
}

Given(
  'a sequence chain {string} padded with filler measures is seeded for editor reveal target',
  async ({ page }, _chain: string) => {
    await page.addInitScript((src) => {
      localStorage.setItem(
        'poentaro:files:v1',
        JSON.stringify({
          active: 'sequence-reveal-target-test.poentaro',
          userFiles: { 'sequence-reveal-target-test.poentaro': src },
          bin: {},
          fileIds: {
            'sequence-reveal-target-test.poentaro': crypto.randomUUID(),
          },
        }),
      )
    }, SOURCE)
  },
)

When(
  'the app loads with the editor-reveal sequence toolbar ready',
  async ({ page }) => {
    await page.goto('/')
    await expect(sequenceToolbarButtons(page)).toHaveCount(3, {
      timeout: 15_000,
    })
    await waitForAssetLoadingBannerGone(page)
  },
)

When(
  'I drag from the {string} sequence entry to the repeated {string} sequence entry, as seen in editor reveal target',
  async ({ page }, _from: string, _to: string) => {
    const buttons = sequenceToolbarButtons(page)
    // Chain-order indices: 0 = Intro (first), 1 = C, 2 = Intro (repeat,
    // resolving to the same written lines as index 0). Drag from "C" (1) to
    // the repeated "Intro" (2) — the drag ends on the far-away entry.
    await buttons.nth(1).hover()
    await page.mouse.down()
    await buttons.nth(2).hover()
    await page.mouse.up()
  },
)

Then("the editor scrolls to reveal Intro's written lines", async ({ page }) => {
  await expect
    .poll(
      () =>
        page.evaluate(() => {
          const monacoApi = (
            window as unknown as { monaco: typeof import('monaco-editor') }
          ).monaco
          const ed = monacoApi.editor.getEditors()[0]
          return ed
            ?.getVisibleRanges()
            .some((r) => r.startLineNumber <= 12 && r.endLineNumber >= 11)
        }),
      { timeout: 3_000 },
    )
    .toBe(true)
})
