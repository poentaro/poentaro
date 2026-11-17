import { expect, type Page } from '@playwright/test'
import { Given, Then, When } from './fixtures'
import {
  CHORD_BOUND_LYRICS_SOURCE,
  EXPECTED_SOURCES,
  HELD_NOTE_MELODY_SOURCE,
  OVER_BRACKETED_CHORD_LYRICS_SOURCE,
  OVERFLOWING_MELODY_SOURCE,
  SOURCE,
  UNWRITTEN_LYRICS_SOURCE,
} from './part-stream-editor-sources'

declare global {
  interface Window {
    monaco?: typeof import('monaco-editor')
  }
}

const MENU_BUTTON = '.editor-toolbar-button[aria-label="Edit part as stream"]'
const FOCUS_EDITOR = '[data-testid="part-focus-editor"]'

async function loadSource(page: Page, source: string) {
  await page.addInitScript((src) => {
    localStorage.setItem(
      'poentaro:files:v1',
      JSON.stringify({
        active: 'stream.poentaro',
        userFiles: { 'stream.poentaro': src },
        bin: {},
        fileIds: { 'stream.poentaro': crypto.randomUUID() },
      }),
    )
  }, source)
  await page.goto('/')
  await page.waitForSelector('.editor-surface .monaco-editor .view-lines', {
    timeout: 30_000,
  })
}

/** Reads the main source editor, or the part focus editor's stream. */
async function editorValue(
  page: Page,
  which: 'main' | 'focus',
): Promise<string | null> {
  return page.evaluate(
    ([side, focusSelector]) => {
      const container = document.querySelector(
        side === 'main' ? '.editor-surface' : focusSelector,
      )
      const editor = window.monaco?.editor
        .getEditors()
        .find((candidate) =>
          container?.contains(candidate.getContainerDomNode()),
        )
      return editor?.getModel()?.getValue() ?? null
    },
    [which, FOCUS_EDITOR] as const,
  )
}

/** Puts the part focus editor's caret at a 1-based line and column. */
async function placeFocusCursor(
  page: Page,
  position: 'start' | 'end' | { line: number; column: number },
) {
  await page.evaluate(
    ([where, focusSelector]) => {
      const container = document.querySelector(focusSelector)
      const editor = window.monaco?.editor
        .getEditors()
        .find((candidate) =>
          container?.contains(candidate.getContainerDomNode()),
        )
      const model = editor?.getModel()
      if (!editor || !model) throw new Error('no part focus editor')
      const target =
        where === 'start'
          ? { lineNumber: 1, column: 1 }
          : where === 'end'
            ? model.getFullModelRange().getEndPosition()
            : { lineNumber: where.line, column: where.column }
      editor.setPosition(target)
      editor.focus()
    },
    [position, FOCUS_EDITOR] as const,
  )
}

function focusLines(page: Page) {
  return editorValue(page, 'focus').then((text) => text?.split('\n') ?? [])
}

function header(page: Page, measureIndex: number) {
  return page.locator(
    `${FOCUS_EDITOR} [data-testid="part-focus-header"][data-measure-index="${measureIndex}"]`,
  )
}

function ghosts(page: Page, measureIndex?: number) {
  const index =
    measureIndex === undefined ? '' : `[data-measure-index="${measureIndex}"]`
  return page.locator(
    `${FOCUS_EDITOR} [data-testid="part-focus-ghost"]${index} .part-focus-ghost-line`,
  )
}

function belowGhosts(page: Page, measureIndex: number) {
  return page.locator(
    `${FOCUS_EDITOR} [data-testid="part-focus-ghost"][data-measure-index="${measureIndex}"][data-position="below"] .part-focus-ghost-line`,
  )
}

function menuItem(page: Page, abbreviation: string) {
  return page.locator(
    `[data-testid="part-focus-part-item"][data-abbreviation="${abbreviation}"]`,
  )
}

function partPill(page: Page, abbreviation: string) {
  return page.locator('.part-toggle-pill').filter({
    has: page.locator('.part-toggle-abbr', {
      hasText: new RegExp(`^${abbreviation}$`),
    }),
  })
}

Given('the part stream editor fixture is loaded', async ({ page }) => {
  await loadSource(page, SOURCE)
})

Given('the chord-bound lyrics fixture is loaded instead', async ({ page }) => {
  await loadSource(page, CHORD_BOUND_LYRICS_SOURCE)
})

Given('the overflowing melody fixture is loaded instead', async ({ page }) => {
  await loadSource(page, OVERFLOWING_MELODY_SOURCE)
})

Given('the held note melody fixture is loaded instead', async ({ page }) => {
  await loadSource(page, HELD_NOTE_MELODY_SOURCE)
})

Given('the unwritten lyrics fixture is loaded instead', async ({ page }) => {
  await loadSource(page, UNWRITTEN_LYRICS_SOURCE)
})

Given(
  'the over-bracketed chord lyrics fixture is loaded instead',
  async ({ page }) => {
    await loadSource(page, OVER_BRACKETED_CHORD_LYRICS_SOURCE)
  },
)

When('I open the Edit part as stream menu', async ({ page }) => {
  await page.click(MENU_BUTTON)
  await expect(page.getByTestId('part-focus-part-item').first()).toBeVisible()
})

When(
  'I choose part {string} from the Edit part as stream menu',
  async ({ page }, abbreviation: string) => {
    await page.click(MENU_BUTTON)
    await menuItem(page, abbreviation).click()
    await expect
      .poll(() => editorValue(page, 'focus'), { timeout: 15_000 })
      .not.toBeNull()
  },
)

Then(
  'the part focus editor is shown in place of the main editor',
  async ({ page }) => {
    const focusBox = await page.locator(FOCUS_EDITOR).boundingBox()
    const mainBox = await page.locator('.editor-main').boundingBox()
    expect(focusBox).toEqual(mainBox)
  },
)

Then('the part focus header says {string}', async ({ page }, text: string) => {
  await expect(page.getByTestId('part-focus-title')).toHaveText(text)
})

Then('the part focus stream is:', async ({ page }, text: string) => {
  await expect.poll(() => editorValue(page, 'focus')).toBe(text)
})

Then(
  'part focus line {int} is {string}',
  async ({ page }, line: number, text: string) => {
    await expect.poll(async () => (await focusLines(page))[line - 1]).toBe(text)
  },
)

Then(
  'measure {int} shows the header row {string}',
  async ({ page }, measure: number, text: string) => {
    await expect(header(page, measure - 1)).toHaveText(text)
  },
)

Then(
  'measure {int} shows the ghost lines below {string} and {string}',
  async ({ page }, measure: number, first: string, second: string) => {
    await expect(belowGhosts(page, measure - 1)).toHaveText([first, second])
  },
)

Then(
  'measure {int} shows the ghost lines below {string}',
  async ({ page }, measure: number, text: string) => {
    await expect(belowGhosts(page, measure - 1)).toHaveText([text])
  },
)

Then(
  'measure {int} shows no ghost line for part {string}',
  async ({ page }, measure: number, abbreviation: string) => {
    await expect(ghosts(page, measure - 1).first()).toBeVisible()
    await expect(
      ghosts(page, measure - 1).filter({ hasText: `[${abbreviation}]` }),
    ).toHaveCount(0)
  },
)

Then(
  'no measure shows a ghost line for part {string}',
  async ({ page }, abbreviation: string) => {
    await expect(ghosts(page).first()).toBeVisible()
    await expect(
      ghosts(page).filter({ hasText: `[${abbreviation}]` }),
    ).toHaveCount(0)
  },
)

Then('the ghost lines cannot be selected or edited', async ({ page }) => {
  const ghost = page.locator(`${FOCUS_EDITOR} [data-testid="part-focus-ghost"]`)
  await expect(ghost.first()).toHaveCSS('pointer-events', 'none')
  await expect(ghost.first()).toHaveCSS('user-select', 'none')
  await expect.poll(() => editorValue(page, 'focus')).not.toContain('[W]')
})

When(
  'I hide part {string} with its toggle',
  async ({ page }, abbreviation: string) => {
    await partPill(page, abbreviation)
      .locator('.part-toggle-segment--eye')
      .click()
  },
)

When(
  'I solo part {string} with its toggle',
  async ({ page }, abbreviation: string) => {
    await partPill(page, abbreviation)
      .locator('.part-toggle-segment--headphones')
      .click()
  },
)

When(
  'I place the cursor at the start of part focus line {int}',
  async ({ page }, line: number) => {
    await placeFocusCursor(page, { line, column: 1 })
  },
)

When(
  'I place the cursor after {string} on part focus line {int}',
  async ({ page }, text: string, line: number) => {
    const lineText = (await focusLines(page))[line - 1] ?? ''
    const index = lineText.indexOf(text)
    if (index < 0) throw new Error(`"${text}" is not on line ${line}`)
    await placeFocusCursor(page, { line, column: index + text.length + 1 })
  },
)

When(
  'I place the cursor at the end of the part focus stream',
  async ({ page }) => {
    await placeFocusCursor(page, 'end')
  },
)

When(
  'I type {string} in the part focus editor',
  async ({ page }, text: string) => {
    await page.keyboard.type(text)
  },
)

When('I press Delete in the part focus editor', async ({ page }) => {
  await page.keyboard.press('Delete')
})

When('I press Enter in the part focus editor', async ({ page }) => {
  await page.keyboard.press('Enter')
})

When('I press undo in the part focus editor', async ({ page }) => {
  // Monaco's undo keybinding is the `Control` chord on every host OS.
  await page.keyboard.press('Control+z')
})

When('I press Escape in the part focus editor', async ({ page }) => {
  await placeFocusCursor(page, 'start')
  await page.keyboard.press('Escape')
})

Then(
  'part focus line {int} shows the header row {string} and no ghost lines',
  async ({ page }, line: number, text: string) => {
    await expect(header(page, line - 1)).toHaveText(text)
    await expect(ghosts(page, line - 1)).toHaveCount(0)
  },
)

Then('the part focus status says {string}', async ({ page }, text: string) => {
  await expect(page.getByTestId('part-focus-status')).toHaveText(text)
})

Then('the part focus editor has an error marker', async ({ page }) => {
  await expect
    .poll(() =>
      page.evaluate((focusSelector) => {
        const container = document.querySelector(focusSelector)
        const monaco = window.monaco
        const model = monaco?.editor
          .getEditors()
          .find((candidate) =>
            container?.contains(candidate.getContainerDomNode()),
          )
          ?.getModel()
        if (!monaco || !model) return 0
        return monaco.editor
          .getModelMarkers({ resource: model.uri })
          .filter((marker) => marker.severity === monaco.MarkerSeverity.Error)
          .length
      }, FOCUS_EDITOR),
    )
    .toBeGreaterThan(0)
})

Then('the part focus Apply button is disabled', async ({ page }) => {
  await expect(
    page.getByRole('button', { name: 'Apply', exact: true }),
  ).toBeDisabled()
})

When('I apply the part focus editor', async ({ page }) => {
  await page.getByRole('button', { name: 'Apply', exact: true }).click()
})

When('I click Cancel in the part focus editor', async ({ page }) => {
  await page.getByRole('button', { name: 'Cancel', exact: true }).click()
})

Then('the main editor is shown', async ({ page }) => {
  await expect(page.locator(FOCUS_EDITOR)).toHaveCount(0)
  await expect(page.locator('.editor-surface .monaco-editor')).toBeVisible()
})

Then(
  'the main editor source is the {} source',
  async ({ page }, name: string) => {
    await expect
      .poll(() => editorValue(page, 'main'))
      .toBe(EXPECTED_SOURCES[name])
  },
)

When('I press undo in the main editor', async ({ page, focusEditor }) => {
  await focusEditor()
  await page.keyboard.press('Control+z')
})

Then(
  'the menu item for part {string} is disabled',
  async ({ page }, abbreviation: string) => {
    await expect(menuItem(page, abbreviation)).toHaveAttribute(
      'data-disabled',
      '',
    )
  },
)

Then(
  'the menu item for part {string} is enabled',
  async ({ page }, abbreviation: string) => {
    await expect(menuItem(page, abbreviation)).not.toHaveAttribute(
      'data-disabled',
    )
  },
)

Then(
  'the menu item for part {string} explains {string}',
  async ({ page }, abbreviation: string, reason: string) => {
    await expect(menuItem(page, abbreviation)).toContainText(reason)
  },
)
