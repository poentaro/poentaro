import { expect } from '@playwright/test'
import { Then, When } from './fixtures'

function editorValue() {
  return window.monaco?.editor.getEditors()[0]?.getModel()?.getValue() ?? null
}

When(
  'I quickly type {string} on a new line at the end of the editor',
  async ({ page, focusEditor }, text: string) => {
    await focusEditor()
    await page.evaluate(() => {
      const editor = window.monaco?.editor.getEditors()[0]
      const model = editor?.getModel()
      if (!editor || !model) throw new Error('no active Monaco editor model')
      const lineNumber = model.getLineCount()
      editor.setPosition({
        lineNumber,
        column: model.getLineMaxColumn(lineNumber),
      })
    })
    await page.keyboard.press('Enter')
    // No delay between keystrokes: the bug only shows when a keystroke lands
    // before React has finished handling the previous one.
    await page.keyboard.type(text, { delay: 0 })
  },
)

Then('the editor ends with {string}', async ({ page }, text: string) => {
  await expect
    .poll(() => page.evaluate(editorValue), { timeout: 3_000 })
    .toMatch(new RegExp(`${text}$`))
})
