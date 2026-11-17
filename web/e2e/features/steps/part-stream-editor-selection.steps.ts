import { expect, type Page } from '@playwright/test'
import { rectVariantSelector, tagSelector } from '../../../src/dataAttributes'
import { Then, When } from './fixtures'

/**
 * Selection sync between the part focus editor and the preview, on the
 * `part stream editor fixture`. Notes are picked by their place among the
 * part's rendered notes, in reading order.
 */

const FOCUS_EDITOR = '[data-testid="part-focus-editor"]'

// The rendered notes' `data-part-index` of the fixture's note parts.
const PART_INDICES: Record<string, number> = { M: 0, B: 3 }

function partIndex(abbreviation: string): number {
  const index = PART_INDICES[abbreviation]
  if (index === undefined) throw new Error(`unknown part ${abbreviation}`)
  return index
}

/** Selects `text` on a 1-based line of the part focus editor. */
async function selectFocusText(page: Page, line: number, text: string) {
  await page.evaluate(
    ([lineNumber, needle, focusSelector]) => {
      const container = document.querySelector(focusSelector)
      const editor = window.monaco?.editor
        .getEditors()
        .find((candidate) =>
          container?.contains(candidate.getContainerDomNode()),
        )
      const model = editor?.getModel()
      if (!editor || !model) throw new Error('no part focus editor')
      const column = model.getLineContent(lineNumber).indexOf(needle) + 1
      if (column === 0) throw new Error(`"${needle}" not on line ${lineNumber}`)
      editor.focus()
      editor.setSelection({
        startLineNumber: lineNumber,
        startColumn: column,
        endLineNumber: lineNumber,
        endColumn: column + needle.length,
      })
    },
    [line, text, FOCUS_EDITOR] as const,
  )
}

/** The part focus editor's selections, as the text each one covers (a
 * caret reads as `""`) and the 1-based line it starts on. */
async function focusSelections(page: Page) {
  return page.evaluate((focusSelector) => {
    const container = document.querySelector(focusSelector)
    const editor = window.monaco?.editor
      .getEditors()
      .find((candidate) => container?.contains(candidate.getContainerDomNode()))
    const model = editor?.getModel()
    if (!editor || !model) return []
    return (editor.getSelections() ?? []).map((selection) => ({
      line: selection.startLineNumber,
      column: selection.startColumn,
      text: model.getValueInRange(selection),
    }))
  }, FOCUS_EDITOR)
}

async function clickNthNote(page: Page, abbreviation: string, nth: number) {
  // A note renders as two `note` groups; only one wraps the click target.
  const clickTarget = rectVariantSelector('note-click-target')
  const target = page
    .locator(
      `${tagSelector('note', { sourcePartIndex: partIndex(abbreviation) })}:has(${clickTarget})`,
    )
    .nth(nth - 1)
    .locator(clickTarget)
  await expect(target).toBeVisible({ timeout: 5_000 })
  await target.click()
  await page.mouse.move(0, 0)
}

When(
  'I select {string} on part focus line {int}',
  async ({ page }, text: string, line: number) => {
    await selectFocusText(page, line, text)
  },
)

When(
  'I click note {int} of part {string} in the preview',
  async ({ page }, nth: number, abbreviation: string) => {
    await clickNthNote(page, abbreviation, nth)
  },
)

Then(
  'the preview range-selects {int} notes of part {string}',
  async ({ page }, count: number, abbreviation: string) => {
    const selected = page.locator(
      `${tagSelector('note', { sourcePartIndex: partIndex(abbreviation) })}[data-note-range-selected]`,
    )
    await expect(selected).toHaveCount(count)
  },
)

Then(
  'the preview range-selects {int} lyrics',
  async ({ page }, count: number) => {
    const selected = page.locator(
      `${tagSelector('lyric')}[data-lyric-range-selected]`,
    )
    await expect(selected).toHaveCount(count)
  },
)

Then(
  'the part focus editor selects {string} on line {int}',
  async ({ page }, text: string, line: number) => {
    await expect
      .poll(() => focusSelections(page))
      .toEqual([expect.objectContaining({ line, text })])
  },
)

Then(
  'the part focus caret is at the start of line {int}',
  async ({ page }, line: number) => {
    await expect
      .poll(() => focusSelections(page))
      .toEqual([{ line, column: 1, text: '' }])
  },
)
