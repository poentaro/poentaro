import { expect } from '@playwright/test'
import { openFileList } from '../../fileSwitcherHelpers'
import { Given, Then, When } from './fixtures'

const SOURCE_A = [
  'metadata { title "File A" },',
  '',
  'parts ["Melody" [M]: notes],',
  '',
  'score """',
  '    (bpm=120 key=C4 time=4/4)',
  '    1 2 3 4',
  '    """,',
].join('\n')

const EDITED_A = SOURCE_A.replace('1 2 3 4', '1 2 3 4 5')

const SOURCE_B = [
  'metadata { title "File B" },',
  '',
  'parts ["Melody" [M]: notes],',
  '',
  'score """',
  '    (bpm=120 key=C4 time=4/4)',
  '    5 6 7 1',
  '    """,',
].join('\n')

async function getStoredFile(
  page: import('@playwright/test').Page,
  name: string,
) {
  return page.evaluate((fileName) => {
    const raw = localStorage.getItem('poentaro:files:v1')
    if (!raw) return ''
    const store = JSON.parse(raw) as {
      active: string
      userFiles: Record<string, string>
    }
    return store.userFiles[fileName] ?? ''
  }, name)
}

// `.monaco-editor .view-lines` can transiently render two overlapping
// snapshots of the content while the virtualized scroller animates, so
// asserting on its text right after an undo/redo keystroke is flaky. Read the
// live model value of the active editor instance instead, via the `monaco`
// global that `@monaco-editor/react`'s loader exposes on `window` (each open
// file has its own model, keyed by `path`, so this must read the model
// attached to the visible editor rather than an arbitrary one).
async function getEditorValue(page: import('@playwright/test').Page) {
  return page.evaluate(() => {
    const monacoApi = (
      window as unknown as { monaco: typeof import('monaco-editor') }
    ).monaco
    const model = monacoApi.editor.getEditors()[0]?.getModel()
    return model?.getValue() ?? ''
  })
}

// Monaco groups keystrokes into undo stops at word boundaries, so whether
// typing " 5" lands as one or two undo stops is a timing detail, not
// something worth pinning down here. Press the given key until the model
// reaches `expected` (or give up after a few tries, letting the final
// assertion report the mismatch).
async function pressUntilValue(
  page: import('@playwright/test').Page,
  key: string,
  expected: string,
) {
  for (let i = 0; i < 5; i++) {
    if ((await getEditorValue(page)) === expected) return
    await page.keyboard.press(key)
  }
}

Given(
  'local files {string} and {string} are seeded for undo-redo across file switch',
  async ({ page }, nameA: string, nameB: string) => {
    await page.addInitScript(
      ({ sourceA, sourceB, fileNameA, fileNameB }) => {
        localStorage.setItem(
          'poentaro:files:v1',
          JSON.stringify({
            active: fileNameA,
            userFiles: { [fileNameA]: sourceA, [fileNameB]: sourceB },
            bin: {},
            fileIds: {
              [fileNameA]: crypto.randomUUID(),
              [fileNameB]: crypto.randomUUID(),
            },
          }),
        )
      },
      {
        sourceA: SOURCE_A,
        sourceB: SOURCE_B,
        fileNameA: nameA,
        fileNameB: nameB,
      },
    )
  },
)

When('the app loads the undo-redo file-switch test files', async ({ page }) => {
  await page.goto('/')

  await page.waitForSelector('.monaco-editor .view-lines', { timeout: 15_000 })
  await page.waitForSelector('.preview-page', { timeout: 15_000 })
})

When(
  'I type {string} at the end of the editor to edit file A',
  async ({ typeAtEditorEnd }, text: string) => {
    await typeAtEditorEnd(text)
  },
)

Then(
  'the stored file {string} contains {string}, as seen in undo redo across file switch',
  async ({ page }, name: string, expected: string) => {
    await expect.poll(getStoredFile.bind(null, page, name)).toContain(expected)
  },
)

When(
  'I switch the active tab to {string} without saving',
  async ({ page }, name: string) => {
    // Switch to file B without explicitly saving.
    await openFileList(page)
    await page.locator('.file-tab-name', { hasText: name }).click()
  },
)

Then('the {string} tab is the active tab', async ({ page }, name: string) => {
  await openFileList(page)
  await expect(
    page.locator('.file-tab-name', { hasText: name }),
  ).toHaveAttribute('aria-current', 'true')
  // The browser is a full-screen modal; close it so later steps can reach
  // the editor.
  await page.keyboard.press('Escape')
  await expect(page.getByTestId('file-browser')).toBeHidden()
})

Then(
  "the editor view-lines show file B's content {string}",
  async ({ page }, text: string) => {
    await expect(page.locator('.monaco-editor .view-lines')).toContainText(text)
  },
)

When(
  'I switch the active tab back to {string}',
  async ({ page }, name: string) => {
    // Switch back to file A: the edit must still be there.
    await openFileList(page)
    await page.locator('.file-tab-name', { hasText: name }).click()
  },
)

Then(
  "the editor view-lines show file A's edited content {string}",
  async ({ page }, text: string) => {
    await expect(page.locator('.monaco-editor .view-lines')).toContainText(text)
  },
)

When(
  "I focus the editor and press undo until file A's original content is restored",
  async ({ page, focusEditor }) => {
    // Undo, after the round trip through B, must undo A's own edit -
    // not some artifact of the tab switch (e.g. reverting to B's content).
    // Monaco's built-in undo/redo keybindings resolve to the `Control`
    // chord regardless of host OS (unlike the app's own Cmd/Ctrl+S handler),
    // so these are not platform-conditional like other specs' `Meta+...`.
    await focusEditor()
    await pressUntilValue(page, 'Control+z', SOURCE_A)
  },
)

Then(
  "the editor model value equals file A's original source",
  async ({ page }) => {
    await expect.poll(getEditorValue.bind(null, page)).toBe(SOURCE_A)
  },
)

Then(
  "the stored file {string} equals file A's original source",
  async ({ page }, name: string) => {
    await expect.poll(getStoredFile.bind(null, page, name)).toBe(SOURCE_A)
  },
)

Then(
  "the stored file {string} is untouched by A's undo",
  async ({ page }, name: string) => {
    expect(await getStoredFile(page, name)).toBe(SOURCE_B)
  },
)

When("I press redo until file A's edit is restored", async ({ page }) => {
  // Redo must restore A's edit, again without touching B.
  await pressUntilValue(page, 'Control+y', EDITED_A)
})

Then(
  "the editor model value equals file A's edited source",
  async ({ page }) => {
    await expect.poll(getEditorValue.bind(null, page)).toBe(EDITED_A)
  },
)

Then(
  "the stored file {string} equals file A's edited source",
  async ({ page }, name: string) => {
    await expect.poll(getStoredFile.bind(null, page, name)).toBe(EDITED_A)
  },
)

Then(
  "the stored file {string} is untouched by A's redo",
  async ({ page }, name: string) => {
    expect(await getStoredFile(page, name)).toBe(SOURCE_B)
  },
)
