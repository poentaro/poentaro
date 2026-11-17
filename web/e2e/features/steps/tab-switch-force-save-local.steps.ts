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

let storedFileABeforeSwitch: string | undefined

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

Given(
  'local files {string} and {string} are seeded for tab-switch force-save',
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

Given(
  'the clock is installed and never advanced before the tab switch',
  async ({ page }) => {
    // Install the fake clock and never advance it — `localBackend`'s
    // `saveContent` is a no-op, so the switch must not rely on the autosave
    // debounce timer ever firing to have the latest edit in storage.
    await page.clock.install()
  },
)

When('the app loads the tab-switch force-save test files', async ({ page }) => {
  await page.goto('/')

  await page.waitForSelector('.monaco-editor .view-lines', { timeout: 15_000 })
  await page.waitForSelector('.preview-page', { timeout: 15_000 })
})

When(
  'I type {string} at the end of the editor before switching tabs',
  async ({ typeAtEditorEnd }, text: string) => {
    await typeAtEditorEnd(text)
  },
)

Then(
  'the stored file {string} contains {string}',
  async ({ page }, name: string, expected: string) => {
    await expect.poll(getStoredFile.bind(null, page, name)).toContain(expected)
    storedFileABeforeSwitch = await getStoredFile(page, name)
  },
)

When('I switch the active tab to {string}', async ({ page }, name: string) => {
  await openFileList(page)
  await page.locator('.file-tab-name', { hasText: name }).click()
})

Then(
  'the {string} tab becomes the current tab',
  async ({ page }, name: string) => {
    await openFileList(page)
    await expect(
      page.locator('.file-tab-name', { hasText: name }),
    ).toHaveAttribute('aria-current', 'true')
  },
)

Then(
  'the editor view-lines contain {string}',
  async ({ page }, text: string) => {
    await expect(page.locator('.monaco-editor .view-lines')).toContainText(text)
  },
)

Then(
  'the stored file {string} is unchanged from before the tab switch',
  async ({ page }, name: string) => {
    expect(await getStoredFile(page, name)).toBe(storedFileABeforeSwitch)
  },
)
