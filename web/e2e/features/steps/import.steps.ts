import fs from 'node:fs'
import { expect } from '@playwright/test'
import { fileSwitcherTrigger, openFileActions } from '../../fileSwitcherHelpers'
import { Given, Then, When } from './fixtures'

const SOURCE = [
  'metadata { title "Test" },',
  '',
  'parts ["Melody" [M]: notes],',
  '',
  'score """',
  '    (bpm=120 key=C4 time=4/4)',
  '    1 2 3 4',
  '    """,',
].join('\n')

let pdfBytes: Buffer | undefined

/** The PDF produced by the last "I export the active file as a PDF" step. */
export function exportedPdfBytes(): Buffer {
  if (!pdfBytes)
    throw new Error('pdfBytes not set — export step must run first')
  return pdfBytes
}

function importButton(page: import('@playwright/test').Page) {
  return page.getByRole('menuitem', { name: /^Import$/ })
}

Given(
  'a local-storage-backed file {string} is seeded for import',
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

Given('the app loads the seeded import test file', async ({ page }) => {
  await page.goto('/')
  await page.waitForSelector('.monaco-editor .view-lines', { timeout: 15_000 })
  await page.waitForSelector('.preview-page', { timeout: 15_000 })
})

When('I export the active file as a PDF', async ({ page }) => {
  const exportButton = page.getByRole('button', {
    name: 'Export',
    exact: true,
  })
  await expect(exportButton).toBeEnabled({ timeout: 30_000 })
  await exportButton.click()
  const pdfItem = page.getByRole('menuitem', { name: 'PDF', exact: true })
  await expect(pdfItem).toBeEnabled({ timeout: 30_000 })
  await pdfItem.click()

  const confirmButton = page.getByTestId('download-rename-confirm')
  await expect(confirmButton).toBeVisible({ timeout: 30_000 })

  const [download] = await Promise.all([
    page.waitForEvent('download'),
    confirmButton.click(),
  ])
  const pdfPath = await download.path()
  expect(pdfPath).toBeTruthy()
  pdfBytes = fs.readFileSync(pdfPath as string)
})

When('I import the exported PDF file', async ({ page }) => {
  if (!pdfBytes)
    throw new Error('pdfBytes not set — export step must run first')
  await openFileActions(page)
  await importButton(page).click()
  await page.setInputFiles('input[type=file]', {
    name: 'test.pdf',
    mimeType: 'application/pdf',
    buffer: pdfBytes,
  })
})

Then(
  'the recovered file opens under a deduped name {string}',
  async ({ page }, name: string) => {
    // "test.poentaro" is already open, so the recovered file lands under a
    // deduped name rather than overwriting it.
    await expect(fileSwitcherTrigger(page)).toContainText(name)
  },
)

Then(
  'the Monaco editor model value equals the original source',
  async ({ page }) => {
    await page.waitForFunction(
      (expected) =>
        window.monaco?.editor.getEditors()[0]?.getModel()?.getValue() ===
        expected,
      SOURCE,
      { timeout: 15_000 },
    )
  },
)

When('I import a plain SVG file with no embedded source', async ({ page }) => {
  await openFileActions(page)
  await importButton(page).click()
  await page.setInputFiles('input[type=file]', {
    name: 'plain.svg',
    mimeType: 'image/svg+xml',
    buffer: Buffer.from(
      '<svg xmlns="http://www.w3.org/2000/svg" width="10" height="10"></svg>',
    ),
  })
})

Then(
  'an import error is shown with message {string}',
  async ({ page }, message: string) => {
    await expect(page.getByText(message)).toBeVisible({ timeout: 10_000 })
  },
)

Then(
  'the error modal message contains {string}',
  async ({ page }, text: string) => {
    await expect(page.getByTestId('error-modal-message')).toContainText(text)
  },
)

Then('the active file is not replaced', async ({ page }) => {
  // Failing import must not crash the app or replace the active file.
  await expect(page.locator('.preview-page')).toHaveCount(1)
})
