import { expect } from '@playwright/test'
import { Given, Then, When } from './fixtures'

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

const MULTI_PART_SOURCE = [
  'metadata { title "Test" },',
  '',
  'parts ["Melody" [M]: notes, "Harmony" [H]: notes],',
  '',
  'score """',
  '    (bpm=120 key=C4 time=4/4)',
  '    1 2 3 4',
  '    5 6 7 1',
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

async function toggleEye(
  page: import('@playwright/test').Page,
  abbreviation: string,
) {
  await page
    .locator('.part-toggle-pill')
    .filter({
      has: page.locator('.part-toggle-abbr', {
        hasText: new RegExp(`^${abbreviation}$`),
      }),
    })
    .locator('.part-toggle-segment--eye')
    .click()
}

function exportMenuButton(page: import('@playwright/test').Page) {
  return page.getByRole('button', { name: 'Export', exact: true })
}

let lastDownload: import('@playwright/test').Download | undefined
let lastDownloadStats: import('node:fs').Stats | undefined

Given('the single-part MIDI export source is loaded', async ({ page }) => {
  await loadSource(page, SINGLE_PART_SOURCE)
  await page.goto('/')
  await page.waitForSelector('.monaco-editor .view-lines', { timeout: 15_000 })
  await page.waitForSelector('.preview-page', { timeout: 15_000 })
})

Given('the multi-part MIDI export source is loaded', async ({ page }) => {
  await loadSource(page, MULTI_PART_SOURCE)
  await page.goto('/')
  await page.waitForSelector('.monaco-editor .view-lines', { timeout: 15_000 })
  await page.waitForSelector('.preview-page', { timeout: 15_000 })
})

Given(
  'I hide the {string} part via its eye toggle, as seen in export midi',
  async ({ page }, abbreviation: string) => {
    await toggleEye(page, abbreviation)
  },
)

When(
  'I export {string} and capture the download',
  async ({ page }, itemName: string) => {
    const menuButton = exportMenuButton(page)
    await expect(menuButton).toBeEnabled({ timeout: 30_000 })
    await menuButton.click()

    const item = page.getByRole('menuitem', { name: itemName, exact: true })
    await expect(item).toBeEnabled({ timeout: 30_000 })
    await item.click()

    const confirmButton = page.getByTestId('download-rename-confirm')
    await expect(confirmButton).toBeVisible({ timeout: 30_000 })

    const [download] = await Promise.all([
      page.waitForEvent('download'),
      confirmButton.click(),
    ])

    const downloadPath = await download.path()
    expect(downloadPath).toBeTruthy()
    const fs = await import('node:fs')
    lastDownloadStats = fs.statSync(downloadPath as string)
    lastDownload = download
  },
)

Then(
  'the downloaded MIDI file is larger than {int} bytes',
  async ({}, size: number) => {
    expect(lastDownloadStats?.size).toBeGreaterThan(size)
  },
)

Then(
  'the downloaded file is named {string}, as seen in export midi',
  async ({}, filename: string) => {
    expect(lastDownload?.suggestedFilename()).toBe(filename)
  },
)
