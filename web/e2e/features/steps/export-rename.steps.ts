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

function renameInput(page: import('@playwright/test').Page) {
  return page.getByTestId('download-rename-input')
}

function renameConfirmButton(page: import('@playwright/test').Page) {
  return page.getByTestId('download-rename-confirm')
}

function renameCancelButton(page: import('@playwright/test').Page) {
  return page.getByTestId('download-rename-cancel')
}

function renameErrorText(page: import('@playwright/test').Page) {
  return page.getByTestId('download-rename-error')
}

let lastDownload: import('@playwright/test').Download | undefined
let downloadCount = 0
/** Registered right before a confirm click/Enter press — so a download
 * that fires immediately isn't missed — and resolved lazily by "the
 * downloaded file is named" step. A confirm that's rejected by validation
 * (no download at all) never gets awaited, so it doesn't need to resolve. */
let pendingDownloadPromise: Promise<
  import('@playwright/test').Download
> | null = null
/** Last blob src the "click Download" step observed on the inline audio
 * player — see that step for why it waits for the src to change before
 * clicking rather than just for generation to finish. */
let lastKnownAudioSrc: string | null = null

Given(
  'the single-part PDF export source is loaded, as seen in export rename',
  async ({ page }) => {
    await loadSource(page, SINGLE_PART_SOURCE)
    await page.goto('/')
    await page.waitForSelector('.monaco-editor .view-lines', {
      timeout: 15_000,
    })
    await page.waitForSelector('.preview-page', { timeout: 15_000 })
  },
)

Given(
  'the export test timeout is extended to {int} seconds, as seen in export rename',
  async ({}, seconds: number) => {
    test.setTimeout(seconds * 1_000)
  },
)

Given(
  'the single-part export source is loaded, as seen in export rename',
  async ({ page }) => {
    await loadSource(page, SINGLE_PART_SOURCE)
    await page.goto('/')
    await page.waitForSelector('.monaco-editor .view-lines', {
      timeout: 15_000,
    })
    await page.waitForSelector('.preview-page', { timeout: 15_000 })
  },
)

When(
  'I export {string} and open the rename modal, as seen in export rename',
  async ({ page }, itemName: string) => {
    const menuButton = exportMenuButton(page)
    await expect(menuButton).toBeEnabled({ timeout: 30_000 })
    await menuButton.click()

    const item = page.getByRole('menuitem', { name: itemName, exact: true })
    await expect(item).toBeEnabled({ timeout: 30_000 })
    await item.click()

    await expect(renameInput(page)).toBeVisible({ timeout: 30_000 })
  },
)

When(
  'I open the export menu and choose {string}, as seen in export rename',
  async ({ page }, itemName: string) => {
    const menuButton = exportMenuButton(page)
    await expect(menuButton).toBeEnabled({ timeout: 30_000 })
    await menuButton.click()

    const item = page.getByRole('menuitem', { name: itemName, exact: true })
    await expect(item).toBeEnabled({ timeout: 30_000 })
    await item.click()
  },
)

When("I click the inline audio player's Download button", async ({ page }) => {
  const audioPlayer = page.locator('.preview-audio-player')
  await expect(audioPlayer).toBeVisible({ timeout: 15_000 })
  // Wait for the src to actually change before clicking Download — while
  // switching formats (e.g. WAV to MP3), `.preview-audio--generating`
  // reflects `wavUrl ? audioGenerating : mp3Exporting` (see Preview.tsx),
  // which stays false throughout an MP3 export as long as the *previous*
  // WAV url is still set; the src only flips once the new blob is ready,
  // so that's what the download button's filename actually reads at
  // click time.
  await expect
    .poll(() => audioPlayer.getAttribute('src'), { timeout: 30_000 })
    .not.toBe(lastKnownAudioSrc)
  lastKnownAudioSrc = await audioPlayer.getAttribute('src')

  const downloadButton = page.getByTestId('preview-audio-download-button')
  await expect(downloadButton).toBeVisible({ timeout: 15_000 })
  await downloadButton.click()

  await expect(renameInput(page)).toBeVisible({ timeout: 30_000 })
})

Then(
  'the rename modal shows the input pre-filled with {string}',
  async ({ page }, filename: string) => {
    await expect(renameInput(page)).toHaveValue(filename)
  },
)

When(
  'I clear the rename input and type {string}',
  async ({ page }, filename: string) => {
    const input = renameInput(page)
    await input.fill('')
    await input.fill(filename)
  },
)

When("I click the modal's {string} button", async ({ page }, label: string) => {
  const confirmButton = renameConfirmButton(page)
  // Contains rather than exact match: the button also carries an "Enter"
  // key-hint badge alongside the label text.
  await expect(confirmButton).toContainText(label)

  // Registered before the click so a download that fires immediately isn't
  // missed — but not awaited here, since an invalid name rejects the
  // submission inline and never fires one (see "rejected inline" scenario).
  // Swallow the rejection here (the context closing at test end otherwise
  // surfaces as an unhandled rejection) — "the downloaded file is named"
  // re-awaits the same promise and reports any real failure there.
  pendingDownloadPromise = page.waitForEvent('download')
  pendingDownloadPromise.catch(() => {})
  await confirmButton.click()
})

When("I click the modal's Cancel button", async ({ page }) => {
  await renameCancelButton(page).click()
})

When('I press Enter in the rename input', async ({ page }) => {
  const input = renameInput(page)
  pendingDownloadPromise = page.waitForEvent('download')
  pendingDownloadPromise.catch(() => {})
  await input.press('Enter')
})

Then('the rename modal is closed', async ({ page }) => {
  await expect(renameInput(page)).not.toBeVisible()
})

Then('the rename modal shows an inline error', async ({ page }) => {
  await expect(renameErrorText(page)).toBeVisible()
})

Then('no download fires, as seen in export rename', async ({ page }) => {
  downloadCount = 0
  const onDownload = () => {
    downloadCount += 1
  }
  page.on('download', onDownload)
  // Give any in-flight download a moment to fire (it shouldn't).
  await page.waitForTimeout(500)
  page.off('download', onDownload)
  expect(downloadCount).toBe(0)
})

Then(
  'the downloaded file is named {string}, as seen in export rename',
  async ({}, filename: string) => {
    if (!pendingDownloadPromise) {
      throw new Error(
        'No download was awaited — expected a prior confirm/Enter step to register one.',
      )
    }
    lastDownload = await pendingDownloadPromise
    expect(lastDownload.suggestedFilename()).toBe(filename)
  },
)

Then(
  'the inline audio player still plays after the rename modal closes',
  async ({ page }) => {
    await expect(renameInput(page)).not.toBeVisible()

    const audioPlayer = page.locator('.preview-audio-player')
    await expect(audioPlayer).toBeVisible()
    const src = await audioPlayer.getAttribute('src')
    expect(src).toMatch(/^blob:/)

    const duration = await audioPlayer.evaluate(
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
    expect(duration).toBeGreaterThan(0)
  },
)
