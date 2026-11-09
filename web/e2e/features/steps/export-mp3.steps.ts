import { expect } from '@playwright/test'
import { Given, Then, When } from './fixtures'

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

function exportMenuButton(page: import('@playwright/test').Page) {
  return page.getByRole('button', { name: 'Export', exact: true })
}

let lastDownload: import('@playwright/test').Download | undefined
let lastDownloadStats: import('node:fs').Stats | undefined

Given('the multi-part MP3 export source is loaded', async ({ page }) => {
  await loadSource(page, MULTI_PART_SOURCE)
  await page.goto('/')
  await page.waitForSelector('.monaco-editor .view-lines', { timeout: 15_000 })
  await page.waitForSelector('.preview-page', { timeout: 15_000 })
})

When(
  'I export {string} and capture the download, as seen in export mp3',
  async ({ page }, itemName: string) => {
    const menuButton = exportMenuButton(page)
    await expect(menuButton).toBeEnabled({ timeout: 30_000 })
    await menuButton.click()

    const item = page.getByRole('menuitem', { name: itemName, exact: true })
    await expect(item).toBeEnabled({ timeout: 30_000 })

    // The MP3 encoder runs in a worker and its wasm panics never reach the
    // UI (no try/catch posts an error message back), so a crash there would
    // otherwise just hang the "download" wait for the full timeout. Race a
    // page error against it so a wasm panic (e.g. rusty_mp3 calling
    // `Instant::now()`, unsupported on wasm32-unknown-unknown) fails fast
    // with a clear message instead.
    const pageErrorPromise = page
      .waitForEvent('pageerror', { timeout: 30_000 })
      .then((error) => {
        throw error
      })
    // Swallow rejections from whichever race loser isn't awaited, so it
    // doesn't surface as an unhandled rejection after this step returns.
    pageErrorPromise.catch(() => {})

    await item.click()

    const confirmButton = page.getByTestId('download-rename-confirm')
    await Promise.race([
      expect(confirmButton).toBeVisible({ timeout: 30_000 }),
      pageErrorPromise,
    ])

    // Register the download listener *before* clicking — the modal can
    // resolve fast enough that the download fires before a `waitForEvent`
    // registered after the click ever starts listening, which would hang
    // forever waiting for an event that already happened.
    const [download] = await Promise.race([
      Promise.all([page.waitForEvent('download'), confirmButton.click()]),
      pageErrorPromise,
    ])

    const downloadPath = await download.path()
    expect(downloadPath).toBeTruthy()
    const fs = await import('node:fs')
    lastDownloadStats = fs.statSync(downloadPath as string)
    lastDownload = download
  },
)

Then(
  'the downloaded MP3 file is larger than {int} bytes',
  async ({}, size: number) => {
    expect(lastDownloadStats?.size).toBeGreaterThan(size)
  },
)

Then(
  'the downloaded file is named {string}, as seen in export mp3',
  async ({}, filename: string) => {
    expect(lastDownload?.suggestedFilename()).toBe(filename)
  },
)

When(
  'I wait for the download to finish, as seen in export mp3',
  async ({ page }) => {
    // Same wasm-panic protection as the download-capturing step above: race
    // the download against a page error so a wasm crash fails fast instead
    // of hanging until the timeout.
    const pageErrorPromise = page
      .waitForEvent('pageerror', { timeout: 30_000 })
      .then((error) => {
        throw error
      })
    pageErrorPromise.catch(() => {})

    const confirmButton = page.getByTestId('download-rename-confirm')
    await Promise.race([
      expect(confirmButton).toBeVisible({ timeout: 30_000 }),
      pageErrorPromise,
    ])

    await Promise.race([
      Promise.all([page.waitForEvent('download'), confirmButton.click()]),
      pageErrorPromise,
    ])
  },
)
