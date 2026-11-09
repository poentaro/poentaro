import { expect } from '@playwright/test'
import { Given, Then, When } from './fixtures'

declare global {
  interface Window {
    monaco?: typeof import('monaco-editor')
  }
}

// Bass has real content in both measure groups, so the trailing all-rest
// Melody line in the second group is safe to drop without emptying the
// group. `[Bass]   5  6 7 1` carries irregular whitespace to exercise the
// collapse-to-single-space cleanup at the same time.
const SOURCE = [
  'parts ["Melody": notes, "Bass": notes],',
  '',
  'score """',
  '    [Melody] 1 2 3 4',
  '    [Bass]   5  6 7 1',
  '',
  '    [Melody] 0 0 0 0',
  '    [Bass] 3 3 3 3',
  '    """,',
].join('\n')

const FORMATTED = [
  'parts ["Melody": notes, "Bass": notes],',
  '',
  'score """',
  '    [Melody] 1 2 3 4',
  '    [Bass] 5 6 7 1',
  '',
  '    [Bass] 3 3 3 3',
  '    """,',
  '',
].join('\n')

async function getEditorValue(page: import('@playwright/test').Page) {
  return page.evaluate(
    () => window.monaco?.editor.getEditors()[0]?.getModel()?.getValue() ?? '',
  )
}

async function getEditorPosition(page: import('@playwright/test').Page) {
  return page.evaluate(() => {
    const pos = window.monaco?.editor.getEditors()[0]?.getPosition()
    return pos ? { lineNumber: pos.lineNumber, column: pos.column } : null
  })
}

async function setEditorPosition(
  page: import('@playwright/test').Page,
  lineNumber: number,
  column: number,
) {
  await page.evaluate(
    ({ lineNumber, column }) => {
      window.monaco?.editor.getEditors()[0]?.setPosition({
        lineNumber,
        column,
      })
    },
    { lineNumber, column },
  )
}

Given('the format-score-toolbar test fixture is loaded', async ({ page }) => {
  await page.addInitScript((src) => {
    localStorage.setItem(
      'poentaro:files:v1',
      JSON.stringify({
        active: 'original.poentaro',
        userFiles: { 'original.poentaro': src },
        bin: {},
        fileIds: { 'original.poentaro': crypto.randomUUID() },
      }),
    )
  }, SOURCE)

  await page.goto('/')
  await page.waitForSelector('.preview-page', { timeout: 15_000 })
})

When(
  'I focus the editor and place the caret at line {int} column {int}',
  async ({ page, focusEditor }, lineNumber: number, column: number) => {
    await focusEditor()
    // Column 5 lands inside "[Mel" on the first score line, whose content and
    // line number are both unaffected by formatting.
    await setEditorPosition(page, lineNumber, column)
  },
)

When('I click the format-score toolbar toggle', async ({ page }) => {
  await page.click('.editor-toolbar-button[aria-label="Format score"]')
})

Then(
  'the editor source is reformatted to the expected output',
  async ({ page }) => {
    await expect.poll(() => getEditorValue(page)).toBe(FORMATTED)
  },
)

Then(
  'the caret remains at line {int} column {int} after formatting',
  async ({ page }, lineNumber: number, column: number) => {
    await expect
      .poll(() => getEditorPosition(page))
      .toEqual({
        lineNumber,
        column,
      })
  },
)

Then(
  'the caret is clamped to a valid position within the formatted source',
  async ({ page }) => {
    const formattedLines = FORMATTED.split('\n')
    await expect
      .poll(async () => {
        const pos = await getEditorPosition(page)
        if (!pos) return null
        const lineContent = formattedLines[pos.lineNumber - 1]
        if (lineContent === undefined) return null
        return pos.column <= lineContent.length + 1
      })
      .toBe(true)
  },
)
