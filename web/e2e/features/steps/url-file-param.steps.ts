import { expect } from '@playwright/test'
import { fileSwitcherTrigger, openFileList } from '../../fileSwitcherHelpers'
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

const SOURCE_B = SOURCE_A.replace('File A', 'File B')

Given(
  'local files {string} and {string} are seeded for the URL param test',
  async ({ page }, nameA: string, nameB: string) => {
    await page.addInitScript(
      ({ a, b, fileNameA, fileNameB }) => {
        localStorage.setItem(
          'poentaro:files:v1',
          JSON.stringify({
            active: fileNameA,
            userFiles: { [fileNameA]: a, [fileNameB]: b },
            bin: {},
            fileIds: {
              [fileNameA]: crypto.randomUUID(),
              [fileNameB]: crypto.randomUUID(),
            },
          }),
        )
      },
      { a: SOURCE_A, b: SOURCE_B, fileNameA: nameA, fileNameB: nameB },
    )
  },
)

When('the app loads at the root URL', async ({ page }) => {
  await page.goto('/')
  await page.waitForSelector('.preview-page', { timeout: 15_000 })
})

Then(
  'the URL has the {string} param set to {string}',
  async ({ page }, param: string, value: string) => {
    const escaped = value.replace(/\./g, '\\.')
    await expect(page).toHaveURL(new RegExp(`[?&]${param}=${escaped}(&|$)`))
  },
)

When(
  'I open the file list and select {string}',
  async ({ page }, name: string) => {
    await openFileList(page)
    await page.locator('.file-tab-name', { hasText: name }).click()
  },
)

Then(
  'the file switcher trigger shows {string}',
  async ({ page }, name: string) => {
    await expect(fileSwitcherTrigger(page)).toContainText(name)
  },
)

When(
  'the app loads with the URL param {string}',
  async ({ page }, param: string) => {
    // Stored `active` is a.poentaro, but the URL names b.poentaro — the URL should win.
    await page.goto(`/?${param}`)
    await page.waitForSelector('.preview-page', { timeout: 15_000 })
  },
)

Then(
  'the active tab in the file list is named {string}',
  async ({ page }, name: string) => {
    await openFileList(page)
    await expect(page.locator('.file-tab--active .file-tab-name')).toHaveText(
      name,
    )
  },
)

When(
  'the app loads with the URL param naming {string}',
  async ({ page }, name: string) => {
    await page.goto(`/?file=${encodeURIComponent(name)}`)
    await page.waitForSelector('.preview-page', { timeout: 15_000 })
  },
)

Then(
  "the URL's {string} param still names {string}",
  async ({ page }, param: string, name: string) => {
    // The URL param must keep naming the selected file, not silently revert
    // to whichever file was `active` in storage before the URL was applied.
    const url = new URL(page.url())
    expect(url.searchParams.get(param)).toBe(name)
  },
)
