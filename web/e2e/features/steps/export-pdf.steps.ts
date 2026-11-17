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
let downloadCount = 0

Given('the single-part PDF export source is loaded', async ({ page }) => {
  await loadSource(page, SINGLE_PART_SOURCE)
  await page.goto('/')
  await page.waitForSelector('.monaco-editor .view-lines', { timeout: 15_000 })
  await page.waitForSelector('.preview-page', { timeout: 15_000 })
})

Given('the multi-part PDF export source is loaded', async ({ page }) => {
  await loadSource(page, MULTI_PART_SOURCE)
  await page.goto('/')
  await page.waitForSelector('.monaco-editor .view-lines', { timeout: 15_000 })
  await page.waitForSelector('.preview-page', { timeout: 15_000 })
})

Given(
  'I hide the {string} part via its eye toggle, as seen in export pdf',
  async ({ page }, abbreviation: string) => {
    await toggleEye(page, abbreviation)
  },
)

When(
  'I export {string} and capture the download, as seen in export pdf',
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

When(
  'I open the export menu and rapid double-click the {string} item, capturing the download',
  async ({ page }, itemName: string) => {
    const menuButton = exportMenuButton(page)
    await expect(menuButton).toBeEnabled({ timeout: 30_000 })
    await menuButton.click()

    const item = page.getByRole('menuitem', { name: itemName, exact: true })
    await expect(item).toBeEnabled({ timeout: 30_000 })

    downloadCount = 0
    page.on('download', () => {
      downloadCount += 1
    })

    // Dispatch two click events back-to-back in a single browser task so both
    // reach the handler before React can re-render the menu as closed,
    // exercising the `pdfExporting`/`splitPdfExporting` re-entrancy guard in
    // `exportPdf` (usePoentaroWorker.ts) rather than relying on real user timing.
    // The dedup happens before `requestDownload` is ever called twice, so
    // only a single rename modal opens.
    await item.evaluate((el: HTMLElement) => {
      el.click()
      el.click()
    })

    const confirmButton = page.getByTestId('download-rename-confirm')
    await expect(confirmButton).toBeVisible({ timeout: 30_000 })

    const [download] = await Promise.all([
      page.waitForEvent('download', { timeout: 30_000 }),
      confirmButton.click(),
    ])
    lastDownload = download

    // Give a stray second export (if the guard were broken) a chance to fire
    // before asserting only one download ever happened.
    await new Promise((resolve) => setTimeout(resolve, 500))
  },
)

Then(
  'the downloaded PDF file is larger than {int} bytes',
  async ({}, size: number) => {
    expect(lastDownloadStats?.size).toBeGreaterThan(size)
  },
)

Then(
  'the downloaded file is named {string}, as seen in export pdf',
  async ({}, filename: string) => {
    expect(lastDownload?.suggestedFilename()).toBe(filename)
  },
)

/** The file names in a ZIP's central directory (each entry's header is
 * `PK\x01\x02`, with the name length at byte 28 and the name at byte 46). */
function zipEntryNames(zip: Buffer): string[] {
  const names: string[] = []
  for (
    let at = zip.indexOf('PK\x01\x02');
    at !== -1;
    at = zip.indexOf('PK\x01\x02', at + 46)
  ) {
    const nameLength = zip.readUInt16LE(at + 28)
    names.push(zip.toString('utf8', at + 46, at + 46 + nameLength))
  }
  return names
}

Then('the downloaded zip holds only {string}', async ({}, filename: string) => {
  const fs = await import('node:fs')
  const zip = fs.readFileSync((await lastDownload?.path()) as string)
  expect(zipEntryNames(zip)).toEqual([filename])
})

Then('only a single PDF download ever fires', async () => {
  expect(downloadCount).toBe(1)
})
