import { expect } from '@playwright/test'
import { fileSwitcherTrigger, openFileList } from '../../fileSwitcherHelpers'
import { Given, Then, When } from './fixtures'

const SOURCE = [
  'metadata { title "Rename Test" },',
  '',
  'parts ["Melody" [M]: notes],',
  '',
  'score """',
  '    (bpm=120 key=C4 time=4/4)',
  '    1 2 3 4',
  '    """,',
].join('\n')

Given(
  'a local-storage-backed file {string} is seeded for renaming',
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

When('the app loads the seeded rename test file', async ({ page }) => {
  await page.goto('/')

  // Wait for the initial SVG preview to appear.
  await page.waitForSelector('.preview-page', { timeout: 15_000 })
})

When(
  'I double-click the active tab name to enter rename mode',
  async ({ page }) => {
    // Double-click the active tab to enter rename mode.
    await openFileList(page)
    const tabName = page.locator('.file-tab--active .file-tab-name')
    await tabName.dblclick()
  },
)

When(
  "I fill the active tab's rename input with {string} and press Enter",
  async ({ page }, newName: string) => {
    // Clear and type a new name, then confirm.
    const input = page.locator('.file-tab--active input.file-tab-name')
    await input.fill(newName)
    await input.press('Enter')
  },
)

Then(
  'the active tab shows the name {string}',
  async ({ page }, name: string) => {
    // The tab should show the new name.
    await expect(page.locator('.file-tab--active .file-tab-name')).toHaveText(
      name,
    )
  },
)

Then(
  'the file switcher trigger shows the renamed file {string}',
  async ({ page }, name: string) => {
    await expect(fileSwitcherTrigger(page)).toContainText(name)
  },
)

Then(
  'the SVG preview is still visible without any manual edits',
  async ({ page }) => {
    // The SVG preview should still be visible without any manual edits.
    await expect(page.locator('.preview-page').first()).toBeVisible({
      timeout: 5_000,
    })
  },
)
