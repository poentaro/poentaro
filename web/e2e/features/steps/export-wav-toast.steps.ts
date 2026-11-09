import { expect } from '@playwright/test'
import { Given, Then, test, When } from './fixtures'

const SINGLE_PART_SOURCE = [
  'metadata { title "Test" },',
  '',
  'parts ["Melody" [M]: notes],',
  '',
  'score """',
  '    (bpm=120 key=C4 time=4/4)',
  '    1 2 3 4',
  '    """,',
].join('\n')

async function loadSource(
  page: import('@playwright/test').Page,
  source: string,
) {
  await page.addInitScript((src) => {
    localStorage.setItem(
      'poentaro:files:v1',
      JSON.stringify({
        active: 'test.poentaro',
        userFiles: { 'test.poentaro': src },
        bin: {},
        fileIds: { 'test.poentaro': crypto.randomUUID() },
      }),
    )
  }, source)
}

function exportMenuButton(page: import('@playwright/test').Page) {
  return page.getByRole('button', { name: 'Export', exact: true })
}

Given(
  'the export test timeout is extended to {int} seconds, as seen in export wav toast',
  async ({}, seconds: number) => {
    test.setTimeout(seconds * 1_000)
  },
)

Given('the single-part WAV toast export source is loaded', async ({ page }) => {
  await loadSource(page, SINGLE_PART_SOURCE)
  await page.goto('/')
  await page.waitForSelector('.monaco-editor .view-lines', { timeout: 15_000 })
  await page.waitForSelector('.preview-page', { timeout: 15_000 })
})

When(
  'I open the export menu and choose {string}, as seen in export wav toast',
  async ({ page }, itemName: string) => {
    const menuButton = exportMenuButton(page)
    await expect(menuButton).toBeEnabled({ timeout: 30_000 })
    await menuButton.click()

    const item = page.getByRole('menuitem', { name: itemName, exact: true })
    await expect(item).toBeEnabled({ timeout: 30_000 })
    await item.click()
  },
)

Then(
  'the export menu closes immediately after choosing {string}',
  async ({ page }, _itemName: string) => {
    // Clicking the menu item closes the dropdown immediately (ExportMenuButton
    // sets `open` to false on select), so the toast is the only place a user
    // can observe that export is still running.
    await expect(page.getByRole('menu')).not.toBeVisible()
  },
)

Then(
  'the export toast is visible with a spinner and says {string}',
  async ({ page }, label: string) => {
    const toast = page.getByTestId('export-audio-toast')
    await expect(toast).toBeVisible({ timeout: 5_000 })
    await expect(toast.locator('.file-tab-bar-spinner')).toBeVisible()
    await expect(toast).toHaveText(label)
  },
)

Then('the inline audio player eventually becomes visible', async ({ page }) => {
  const audioPlayer = page.locator('.preview-audio-player')
  await expect(audioPlayer).toBeVisible({ timeout: 15_000 })
})

Then(
  'the export toast goes away once generation finishes',
  async ({ page }) => {
    // Once generation finishes the toast should go away rather than linger.
    const toast = page.getByTestId('export-audio-toast')
    await expect(toast).not.toBeVisible({ timeout: 5_000 })
  },
)
