import { expect } from '@playwright/test'
import { tagSelector } from '../../../src/dataAttributes'
import { focusEditor } from '../../fileSwitcherHelpers'
import { stableBoundingBox } from '../../rangeSelectHelpers'
import { Given, Then, When } from './fixtures'

/**
 * `max_measures_per_system = 1` forces each measure onto its own system.
 * Each measure mentions only one of the two declared parts, so the other
 * part is implicitly filled with rests and its (default-suppressed) row
 * doesn't render for that system — leaving exactly one part label per
 * system:
 *
 *   System 0 (measure 0): Melody "1 2" (Harmony omitted, suppressed)
 *   System 1 (measure 1): Harmony "5 6" (Melody omitted, suppressed)
 */
const source = [
  'metadata { title "part label copy-paste across systems test", max measures per system 1 },',
  '',
  'parts ["Melody" [M]: notes, "Harmony" [H]: notes],',
  '',
  'score """',
  '[M] 1 2',
  '',
  '[H] 5 6',
  '""",',
].join('\n')

async function loadFixture(page: import('@playwright/test').Page) {
  await page.addInitScript((source) => {
    localStorage.setItem(
      'poentaro:files:v1',
      JSON.stringify({
        active: 'part-label-copy-paste-test.poentaro',
        userFiles: { 'part-label-copy-paste-test.poentaro': source },
        bin: {},
        fileIds: {
          'part-label-copy-paste-test.poentaro':
            'part-label-copy-paste-test-id-001',
        },
      }),
    )
  }, source)
}

/** Waits for measureSpans to be primed (same priming dance the measure-select
 * specs use) so the SVG has settled before hit-testing. */
async function primeMeasureSpans(page: import('@playwright/test').Page) {
  await focusEditor(page)
  await page.keyboard.press('Control+g')
  await page.keyboard.type('6')
  await page.keyboard.press('Enter')
  await expect(page.locator('button.play-measure-btn')).toHaveText(/Measure/, {
    timeout: 5_000,
  })
  await expect(
    page.locator('.preview-page [data-testid="measure-highlight"]').first(),
  ).toBeVisible({ timeout: 5_000 })
}

function partLabel(
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

async function clickLabel(
  page: import('@playwright/test').Page,
  partIndex: number,
  measureIndexStart: number,
) {
  const label = partLabel(page, partIndex, measureIndexStart)
  await expect(label).toBeVisible({ timeout: 5_000 })
  const box = await stableBoundingBox(label)
  if (!box) throw new Error('Could not get bounding box for the part label.')

  // A plain click (mousedown + mouseup at the same point, no drag) — see
  // `part-label-click-selects-notes.steps.ts`'s "I plain-click the Melody
  // part label" step, which this mirrors.
  await page.mouse.move(box.x + box.width / 2, box.y + box.height / 2)
  await page.mouse.down()
  await page.mouse.up()
}

async function getModelLines(page: import('@playwright/test').Page) {
  return page.evaluate(() => {
    const monacoApi = (
      window as unknown as { monaco?: typeof import('monaco-editor') }
    ).monaco
    return (
      monacoApi?.editor.getEditors()[0]?.getModel()?.getLinesContent() ?? []
    )
  })
}

Given('the part-label copy-paste fixture is loaded', async ({ page }) => {
  await loadFixture(page)
  await page.goto('/')

  await page.waitForSelector('[data-testid="play-measure-button"]', {
    timeout: 15_000,
  })
  await page.waitForSelector(
    tagSelector('part-label', { sourcePartIndex: 0 }),
    {
      timeout: 10_000,
    },
  )
  await primeMeasureSpans(page)
})

When("I click system 0's Melody part label", async ({ page }) => {
  await clickLabel(page, 0, 0)
})

When("I click system 1's Harmony part label", async ({ page }) => {
  await clickLabel(page, 1, 1)
})

When('I press the copy keyboard shortcut', async ({ page }) => {
  // Chromium binds its native copy accelerator to Cmd+C on macOS hosts
  // (Monaco's own "CtrlCmd+C" keybinding follows the same OS convention),
  // not Ctrl+C — `Meta` is Playwright's cross-platform name for that key.
  await page.keyboard.press('ControlOrMeta+c')
})

When('I press the paste keyboard shortcut', async ({ page }) => {
  await page.keyboard.press('ControlOrMeta+v')
})

Then(
  "system 1's Harmony part now contains just the notes {string}",
  async ({ page }, notes: string) => {
    // Line 7 (0-indexed) is measure 1's Harmony line in the fixture above.
    await expect
      .poll(async () => (await getModelLines(page))[7], { timeout: 3_000 })
      .toBe(`[H] ${notes}`)
  },
)

Then(
  "system 0's Melody part still contains just the notes {string}",
  async ({ page }, notes: string) => {
    // Line 5 (0-indexed) is measure 0's Melody line in the fixture above.
    const lines = await getModelLines(page)
    expect(lines[5]).toBe(`[M] ${notes}`)
  },
)
