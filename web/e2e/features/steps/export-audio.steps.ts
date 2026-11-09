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

let firstSrc: string | null = null
let secondSrc: string | null = null
let lastDownload: import('@playwright/test').Download | undefined
let lastDownloadStats: import('node:fs').Stats | undefined

Given(
  'the export test timeout is extended to {int} seconds',
  async ({}, seconds: number) => {
    test.setTimeout(seconds * 1_000)
  },
)

Given('the single-part export source is loaded', async ({ page }) => {
  await loadSource(page, SINGLE_PART_SOURCE)
  await page.goto('/')
  await page.waitForSelector('.monaco-editor .view-lines', { timeout: 15_000 })
  await page.waitForSelector('.preview-page', { timeout: 15_000 })
})

Given('the multi-part export source is loaded', async ({ page }) => {
  await loadSource(page, MULTI_PART_SOURCE)
  await page.goto('/')
  await page.waitForSelector('.monaco-editor .view-lines', { timeout: 15_000 })
  await page.waitForSelector('.preview-page', { timeout: 15_000 })
})

Given(
  'I hide the {string} part via its eye toggle',
  async ({ page }, abbreviation: string) => {
    await toggleEye(page, abbreviation)
  },
)

When(
  'I open the export menu and choose {string}',
  async ({ page }, itemName: string) => {
    const menuButton = exportMenuButton(page)
    await expect(menuButton).toBeEnabled({ timeout: 30_000 })
    await menuButton.click()

    const item = page.getByRole('menuitem', { name: itemName, exact: true })
    await expect(item).toBeEnabled({ timeout: 30_000 })
    await item.click()
  },
)

When('I open the export menu', async ({ page }) => {
  const menuButton = exportMenuButton(page)
  await menuButton.click()
})

When(
  'I open the export menu and choose {string} and capture the download',
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
  'I type {string} at the end of the editor, as seen in export audio',
  async ({ typeAtEditorEnd }, text: string) => {
    // Editing the source in place (no file switch) must not clear the
    // existing audio or reset the item back to "WAV" — wavUrl is only cleared
    // on `activeFile` change, not on `source` change.
    await typeAtEditorEnd(text)
  },
)

Then('the inline audio player is visible with a blob src', async ({ page }) => {
  const audioPlayer = page.locator('.preview-audio-player')
  await expect(audioPlayer).toBeVisible({ timeout: 15_000 })

  firstSrc = await audioPlayer.getAttribute('src')
  expect(firstSrc).toMatch(/^blob:/)
})

Then(
  'the inline audio player has decoded playable audio with positive duration',
  async ({ page }) => {
    const audioPlayer = page.locator('.preview-audio-player')

    // A silently empty/corrupt WAV would still produce a `blob:` src but have
    // zero duration — assert the browser actually decoded playable audio.
    const firstDuration = await audioPlayer.evaluate(
      (el: HTMLAudioElement) =>
        new Promise<number>((resolve) => {
          if (el.readyState >= 1 && Number.isFinite(el.duration)) {
            resolve(el.duration)
            return
          }
          el.addEventListener('loadedmetadata', () => resolve(el.duration), {
            once: true,
          })
        }),
    )
    expect(firstDuration).toBeGreaterThan(0)
  },
)

Then(
  "the inline audio player's blob src changes to a new blob url",
  async ({ page }) => {
    const audioPlayer = page.locator('.preview-audio-player')

    // The previous blob URL is revoked and a new one set (usePoentaroWorker.ts's
    // setNextWavUrl) — assert the src actually changed rather than being reused.
    await expect
      .poll(() => audioPlayer.getAttribute('src'), { timeout: 15_000 })
      .not.toBe(firstSrc)
    secondSrc = await audioPlayer.getAttribute('src')
    expect(secondSrc).toMatch(/^blob:/)
  },
)

Then(
  'the inline audio player keeps showing the regenerated audio',
  async ({ page }) => {
    const audioPlayer = page.locator('.preview-audio-player')
    await expect(audioPlayer).toBeVisible()
    await expect(audioPlayer).toHaveAttribute('src', secondSrc as string)
  },
)

Then('only one inline audio player is visible', async ({ page }) => {
  await expect(page.locator('.preview-audio-player')).toHaveCount(1)
})

Then(
  'the export menu shows a {string} item',
  async ({ page }, itemName: string) => {
    await expect(
      page.getByRole('menuitem', { name: itemName, exact: true }),
    ).toBeVisible()
  },
)

Then(
  'the downloaded zip file is larger than {int} bytes',
  async ({}, size: number) => {
    expect(lastDownloadStats?.size).toBeGreaterThan(size)
  },
)

Then('the downloaded file is named {string}', async ({}, filename: string) => {
  expect(lastDownload?.suggestedFilename()).toBe(filename)
})

When(
  'I wait for the download to finish, as seen in export audio',
  async ({ page }) => {
    const confirmButton = page.getByTestId('download-rename-confirm')
    await expect(confirmButton).toBeVisible({ timeout: 30_000 })
    await Promise.all([
      page.waitForEvent('download', { timeout: 30_000 }),
      confirmButton.click(),
    ])
  },
)
