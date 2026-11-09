import { expect } from '@playwright/test'
import { Given, Then, When } from './fixtures'

const SOURCE = [
  'metadata { title "Autosave Test" },',
  '',
  'parts ["Melody" [M]: notes],',
  '',
  'score """',
  '    (bpm=120 key=C4 time=4/4)',
  '    1 2 3 4',
  '    """,',
].join('\n')

async function getStoredSource(page: import('@playwright/test').Page) {
  return page.evaluate(() => {
    const raw = localStorage.getItem('poentaro:files:v1')
    if (!raw) return ''
    const store = JSON.parse(raw) as {
      active: string
      userFiles: Record<string, string>
    }
    return store.userFiles[store.active] ?? ''
  })
}

Given(
  'a local-storage-backed file {string} is seeded',
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

Given('the clock is installed and never advanced', async ({ page }) => {
  // Install the fake clock (and never advance it). `localBackend`'s
  // `saveContent` is a no-op — `useLocalStorage` already writes synchronously
  // on every keystroke — so proving the edit lands in `localStorage` without
  // ever fast-forwarding the clock demonstrates local persistence has no
  // dependency on the autosave debounce timer at all.
  await page.clock.install()
})

When('the app loads the local-storage-backed file', async ({ page }) => {
  await page.goto('/')

  await page.waitForSelector('.monaco-editor .view-lines', { timeout: 15_000 })
  await page.waitForSelector('.preview-page', { timeout: 15_000 })
})

When(
  'I type {string} at the end of the editor',
  async ({ typeAtEditorEnd }, text: string) => {
    await typeAtEditorEnd(text)
  },
)

Then(
  'the stored source for the active file contains {string}',
  async ({ page }, expected: string) => {
    await expect.poll(getStoredSource.bind(null, page)).toContain(expected)
  },
)

Then('no save-status badge appears', async ({ page }) => {
  // No debounced save is even scheduled for the local backend, so the
  // save-status badge (only meaningful for GitHub) never appears.
  await expect(page.getByTestId('save-status-badge')).toHaveCount(0)
})
