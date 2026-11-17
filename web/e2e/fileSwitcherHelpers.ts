import type { Page } from '@playwright/test'

declare global {
  interface Window {
    monaco?: typeof import('monaco-editor')
  }
}

function activeEditorValue(): string | null {
  const model = window.monaco?.editor.getEditors()[0]?.getModel()
  return model?.getValue() ?? null
}

function editorHasFocus() {
  const active = document.activeElement
  return (
    !!active &&
    (active.classList.contains('native-edit-context') ||
      active.tagName === 'TEXTAREA')
  )
}

/** Clicks the Monaco editor's view-lines and waits for it to actually gain
 * focus before returning. A plain `page.click(...)` can resolve before
 * Monaco has finished wiring up its focus target — a race that's normally
 * lost within milliseconds, but occasionally not at all — which then races
 * immediately-following keyboard shortcuts. Retries the click, and falls
 * back to focusing the editor's input node directly, until focus lands. */
export async function focusEditor(page: Page) {
  // The file browser is a full-screen modal; a scenario that opened it to
  // pick or check a file may have left it open over the editor.
  const browser = page.getByTestId('file-browser')
  if (await browser.isVisible()) {
    await page.keyboard.press('Escape')
    await browser.waitFor({ state: 'hidden' })
  }
  for (let attempt = 0; attempt < 10; attempt++) {
    await page.click('.monaco-editor .view-lines')
    if (await page.evaluate(editorHasFocus)) return
    await page.waitForTimeout(150)
    if (await page.evaluate(editorHasFocus)) return
    await page.evaluate(() => {
      const node = document.querySelector<HTMLElement>(
        '.monaco-editor .native-edit-context, .monaco-editor textarea',
      )
      node?.focus()
    })
    if (await page.evaluate(editorHasFocus)) return
  }
  throw new Error('focusEditor: editor never gained focus')
}

/** Focuses the editor, moves the cursor to the end of the document, and
 * types `text` there — the pattern nearly every editing test uses. Beyond
 * `focusEditor`'s DOM-focus race, there is a second, narrower race: even
 * once the editor's input node has DOM focus, Monaco can take a further
 * moment to wire up its keybinding/input handling, so a `Control+End` or
 * `type()` fired immediately after focus can be dropped or partially land.
 * This waits for the cursor to actually reach the end (via the real Monaco
 * instance exposed on `window.monaco`, set up in `monacoSetup.ts`) before
 * typing, then verifies the typed text landed by reading the model value
 * back. If it didn't, it restores the model to its pre-attempt value and
 * retries the whole focus → move → type sequence. */
/** Types `text` after the last line of notes, inside the score string. */
export async function typeAtEditorEnd(page: Page, text: string) {
  for (let attempt = 0; attempt < 5; attempt++) {
    await focusEditor(page)

    const before = await page.evaluate(activeEditorValue)
    if (before === null) {
      throw new Error('typeAtEditorEnd: no active Monaco editor model')
    }

    const expected = await page.evaluate((text) => {
      const editor = window.monaco?.editor.getEditors()[0]
      const model = editor?.getModel()
      if (!editor || !model) throw new Error('no active Monaco editor model')
      const lines = model.getLinesContent()
      const isLastNote = (line: string) =>
        line.trim() !== '' && !line.trim().startsWith('"""')
      const lineNumber = lines.findLastIndex(isLastNote) + 1
      const column = model.getLineMaxColumn(lineNumber)
      editor.setPosition({ lineNumber, column })
      const offset = model.getOffsetAt({ lineNumber, column })
      const value = model.getValue()
      return value.slice(0, offset) + text + value.slice(offset)
    }, text)

    await page.keyboard.type(text)

    try {
      await page.waitForFunction(
        (expected) =>
          window.monaco?.editor.getEditors()[0]?.getModel()?.getValue() ===
          expected,
        expected,
        { timeout: 2_000 },
      )
      return
    } catch {
      // The typed text was dropped or only partially landed. Reset the
      // model back to its known-good pre-attempt value (still routed
      // through the app's normal onChange, since it's a real model edit)
      // and retry the whole sequence from a clean state.
      await page.evaluate((before) => {
        window.monaco?.editor.getEditors()[0]?.getModel()?.setValue(before)
      }, before)
    }
  }
  throw new Error(`typeAtEditorEnd: "${text}" never landed after retries`)
}

/** The file-switcher ("<filename> ▾") trigger button in the header — its
 * text is the currently active file's name, whether that's a user file or
 * one of the read-only demo files (which live under the read-only `Demo/` row
 * of this same dropdown). */
export function fileSwitcherTrigger(page: Page) {
  return page
    .locator('.file-tab-bar .export-menu')
    .first()
    .locator('.preview-export-btn')
}

/** The "Demo" disclosure item inside the file-switcher dropdown — expands
 * to the read-only `demo/*.poentaro` file list nested underneath it. */
export function demoDisclosure(page: Page) {
  return page.getByRole('button', { name: 'Demo', exact: true })
}

/** Opens the file-switcher dropdown (the "<filename> ▾" trigger in the
 * header) so its file list / Bin become visible and interactable. Idempotent
 * — the trigger toggles open/closed, so this is a no-op if already open. */
export async function openFileList(page: Page) {
  const trigger = fileSwitcherTrigger(page)
  if ((await trigger.getAttribute('aria-expanded')) === 'true') return
  await trigger.click()
}

/** Opens the file-switcher dropdown and expands its `Demo/` row
 * (the read-only `demo/*.poentaro` file list). Idempotent — a no-op if the
 * row is already expanded. */
export async function openDemoFileList(page: Page) {
  await openFileList(page)
  const disclosure = demoDisclosure(page)
  if ((await disclosure.getAttribute('aria-expanded')) === 'true') return
  await disclosure.click()
}

/** Expands the "Demo/` row and selects the fragment named `name` (e.g.
 * `02-durations.poentaro`), closing the file-switcher dropdown. */
export async function selectDemoFile(page: Page, name: string) {
  await openDemoFileList(page)
  await page.locator('.file-tab-name', { hasText: name }).click()
}

/** Locates a single file-switcher tab whose *entire* display name matches
 * `name` exactly (anchored regex), not merely contains it as a substring.
 * Needed wherever many scenarios seed files into one shared, real,
 * non-isolated account (the cloud storage backend's e2e coverage, backed
 * by a real local D1 instance) rather than each getting its own mocked
 * in-memory store: a plain `hasText: name` string match can then resolve
 * to more than one tab once another concurrently-seeded file's name
 * happens to contain `name` as a substring (e.g. `"a"` also matches
 * `"auto"`; `"banner"` also matches `"banner-reconnect"`), which Playwright
 * treats as a strict-mode error. */
export function fileTabByExactName(page: Page, name: string) {
  const escaped = name.replace(/[.*+?^${}()|[\]\\]/g, '\\$&')
  return page.locator('.file-tab-name', { hasText: new RegExp(`^${escaped}$`) })
}

/** Opens the "⋯" file-actions dropdown (New / Duplicate / Share / Storage…)
 * in the header. Idempotent — the trigger toggles open/closed, so this is a
 * no-op if already open. */
export async function openFileActions(page: Page) {
  // The file browser is a modal: it makes the rest of the page inert.
  const browser = page.getByTestId('file-browser')
  if (await browser.isVisible()) {
    await page.keyboard.press('Escape')
    await browser.waitFor({ state: 'hidden' })
  }
  const trigger = page.getByRole('button', { name: 'File actions' })
  if ((await trigger.getAttribute('aria-expanded')) === 'true') return
  await trigger.click()
}

/** Opens the Bin modal via the "Bin (N)" item in the "⋯" file-actions
 * dropdown. Idempotent — a no-op if the modal is already open. */
export async function openBin(page: Page) {
  if (await page.locator('[data-testid="bin-modal"]').count()) return
  await openFileActions(page)
  await page.locator('.file-tab-bar-bin-trigger').click()
}

/** Closes the Bin modal via Escape. Unlike the old submenu, this is a real
 * modal — it hides the rest of the page from interaction (and from the
 * accessibility tree) while open, so tests must close it before touching
 * anything else. */
export async function closeBin(page: Page) {
  await page.keyboard.press('Escape')
  await page.locator('[data-testid="bin-modal"]').waitFor({ state: 'hidden' })
}

/** Answers the New file name prompt and confirms it. */
export async function nameNewFile(page: Page, name: string) {
  const prompt = page.getByRole('dialog', { name: 'New file' })
  await prompt.getByRole('textbox').fill(name)
  await prompt.getByRole('button', { name: 'Create' }).click()
}
