import { expect } from '@playwright/test'
import { fileSwitcherTrigger, openFileList } from '../../fileSwitcherHelpers'
import { Given, Then, When } from './fixtures'

const SOURCE = [
  'metadata { title "Extension Test" },',
  '',
  'parts ["Melody" [M]: notes],',
  '',
  'score """',
  '    (bpm=120 key=C4 time=4/4)',
  '    1 2 3 4',
  '    """,',
].join('\n')

Given(
  'a local-storage-backed file {string} is seeded to test extension hiding',
  async ({ page }, name: string) => {
    await page.addInitScript(
      ({ src, fileName }) => {
        localStorage.setItem(
          'poentaro:files:v1',
          JSON.stringify({
            active: fileName,
            userFiles: { [fileName]: src },
            bin: {},
            fileIds: { [fileName]: crypto.randomUUID() },
          }),
        )
      },
      { src: SOURCE, fileName: name },
    )
  },
)

When(
  'the app loads the seeded extension-hiding test file',
  async ({ page }) => {
    await page.goto('/')
    await page.waitForSelector('.preview-page', { timeout: 15_000 })
  },
)

Then(
  'the file switcher trigger shows the extension-less name {string}',
  async ({ page }, name: string) => {
    await expect(fileSwitcherTrigger(page)).toHaveText(name)
  },
)

Then(
  'the active file tab shows the extension-less name {string}',
  async ({ page }, name: string) => {
    await openFileList(page)
    await expect(page.locator('.file-tab--active .file-tab-name')).toHaveText(
      name,
    )
  },
)

When(
  'I double-click the active tab name to enter rename mode, as seen in extension hiding',
  async ({ page }) => {
    const tabName = page.locator('.file-tab--active .file-tab-name')
    await tabName.dblclick()
  },
)

Then(
  'the rename input starts with the extension-less value {string}',
  async ({ page }, value: string) => {
    const input = page.locator('.file-tab--active input.file-tab-name')
    await expect(input).toHaveValue(value)
  },
)

When(
  'I fill the rename input with {string} and press Enter, as seen in extension hiding',
  async ({ page }, name: string) => {
    const input = page.locator('.file-tab--active input.file-tab-name')
    await input.fill(name)
    await input.press('Enter')
  },
)
