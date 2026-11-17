import { expect, type Page } from '@playwright/test'
import { tagSelector } from '../../../src/dataAttributes'
import { stableBoundingBox } from '../../rangeSelectHelpers'
import { Given, Then, When } from './fixtures'

const SOURCE = [
  '## the song',
  'metadata { title "Song" },',
  '',
  '## who sings',
  'parts ["Melody" [M]: notes, "Melody lyrics" [Mv1]: lyrics [M]],',
  '',
  'score """',
  '    // first verse',
  '    bpm=120 key=C4 time=4/4',
  '    [M] 1 1 5 5',
  '    [Mv1] twin- kle twin- kle',
  '',
  '    [M] 6 6 5 0',
  '    [Mv1] lit- tle star',
  '    """,',
  '',
].join('\n')

const COMMENTS = ['# the song', '# who sings', '// first verse']

function editorValue(page: Page) {
  return page.evaluate(
    () => window.monaco?.editor.getEditors()[0]?.getModel()?.getValue() ?? '',
  )
}

function diagnosticLines(page: Page) {
  return page.evaluate(() => {
    const model = window.monaco?.editor.getEditors()[0]?.getModel()
    if (!model) return []
    return window.monaco?.editor
      .getModelMarkers({ resource: model.uri })
      .map((marker) => marker.startLineNumber)
  })
}

Given('the anyx document fixture is loaded', async ({ page }) => {
  await page.addInitScript((source) => {
    localStorage.setItem(
      'poentaro:files:v1',
      JSON.stringify({
        active: 'test.poentaro',
        userFiles: { 'test.poentaro': source },
        bin: {},
        fileIds: { 'test.poentaro': crypto.randomUUID() },
      }),
    )
  }, SOURCE)
  await page.goto('/')
  await page.waitForSelector('.monaco-editor .view-lines', { timeout: 30_000 })
  await page.waitForSelector(tagSelector('measure', { index: 1 }), {
    timeout: 15_000,
  })
})

Then(
  "the preview shows the anyx document's {int} measures",
  async ({ page }, count: number) => {
    await expect(page.locator(tagSelector('measure')).first()).toBeVisible()
    await expect(
      page.locator(tagSelector('measure', { index: count - 1 })).first(),
    ).toBeVisible()
    await expect(
      page.locator(tagSelector('measure', { index: count })),
    ).toHaveCount(0)
  },
)

Then('the editor has no anyx diagnostics', async ({ page }) => {
  await page.waitForTimeout(1_000)
  expect(await diagnosticLines(page)).toEqual([])
})

When(
  'I select the whole of line {int} in the anyx document',
  async ({ page }, line: number) => {
    await page.evaluate((lineNumber) => {
      const editor = window.monaco?.editor.getEditors()[0]
      const maxColumn = editor?.getModel()?.getLineMaxColumn(lineNumber) ?? 1
      editor?.focus()
      editor?.setSelection({
        startLineNumber: lineNumber,
        startColumn: 1,
        endLineNumber: lineNumber,
        endColumn: maxColumn,
      })
    }, line)
  },
)

Then(
  '{int} notes are range-selected in the anyx document',
  async ({ page }, count: number) => {
    await expect(
      page.locator(`${tagSelector('note')}[data-note-range-selected]`),
    ).toHaveCount(count)
  },
)

When(
  'I Cmd\\/Ctrl-click measure {int} in the anyx document',
  async ({ page }, measure: number) => {
    const box = await stableBoundingBox(
      page.locator(tagSelector('measure', { index: measure })).first(),
    )
    if (!box) throw new Error(`Measure ${measure} has no bounding box.`)
    await page.keyboard.down('ControlOrMeta')
    await page.mouse.click(box.x + box.width / 2, box.y + box.height / 2)
    await page.keyboard.up('ControlOrMeta')
  },
)

Then(
  'the editor selects {string} in the anyx document',
  async ({ page }, text: string) => {
    await expect
      .poll(() =>
        page.evaluate(() => {
          const editor = window.monaco?.editor.getEditors()[0]
          const model = editor?.getModel()
          const selections = editor?.getSelections() ?? []
          return selections
            .map((selection) => model?.getValueInRange(selection))
            .join(' ')
        }),
      )
      .toBe(text)
  },
)

Then('the anyx document still has its comments', async ({ page }) => {
  const source = await editorValue(page)
  for (const comment of COMMENTS) expect(source).toContain(comment)
})

When("I close the anyx document's edit parts modal", async ({ page }) => {
  await page.keyboard.press('Escape')
  await page.getByTestId('edit-parts-modal').waitFor({ state: 'hidden' })
})

When(
  'I replace {string} with {string} in the anyx document',
  async ({ page }, before: string, after: string) => {
    await page.evaluate(
      ({ before, after }) => {
        const model = window.monaco?.editor.getEditors()[0]?.getModel()
        model?.setValue(model.getValue().replace(before, after))
      },
      { before, after },
    )
  },
)

Then(
  'the editor has an anyx diagnostic on line {int}',
  async ({ page }, line: number) => {
    await expect.poll(() => diagnosticLines(page)).toContain(line)
  },
)

Then(
  'the preview still shows a page for the anyx document',
  async ({ page }) => {
    await expect(page.locator('.preview-page').first()).toBeVisible()
  },
)
