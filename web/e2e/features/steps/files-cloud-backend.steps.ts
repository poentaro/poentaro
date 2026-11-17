import { expect } from '@playwright/test'
import {
  fileSwitcherTrigger,
  fileTabByExactName,
  nameNewFile,
  openBin,
  openFileActions,
  openFileList,
} from '../../fileSwitcherHelpers'
import { currentSeededFileDisplayName } from './cloud-account.steps'
import { gotoCloudApp } from './cloud-account-helpers'
import { Then, When } from './fixtures'

// Create/rename/delete/duplicate coverage for the cloud storage backend.
// See also `files-cloud-backend-restore-collision.steps.ts` (the
// restore-collision scenario) and `files-cloud-backend-autosave.steps.ts`
// (autosave) -- split out of this file to stay under this repo's 400-line
// cap.

const newButton = ({ page }: { page: import('@playwright/test').Page }) =>
  page.locator('.export-menu-item').first()
const duplicateButton = ({ page }: { page: import('@playwright/test').Page }) =>
  page.getByRole('menuitem', { name: 'Duplicate' })

When('the app loads the cloud backend with no files', async ({ page }) => {
  await gotoCloudApp(page)
  await page.waitForSelector('.monaco-editor .view-lines', { timeout: 15_000 })
})

When(
  'I click the {string} button to create a file',
  async ({ page }, label) => {
    expect(label).toBe('New')
    await openFileActions(page)
    await newButton({ page }).click()
    await nameNewFile(page, 'untitled')
  },
)

Then('the active tab becomes {string}', async ({ page }, name: string) => {
  await expect(fileSwitcherTrigger(page)).toContainText(name)
})

// Named distinctly from `share.steps.ts`'s plain "I reload the page" (which
// must NOT reopen the file list -- that scenario's file switcher is hidden
// entirely) -- an identical step text with two different implementations
// would be an ambiguous step definition at collection time.
When('I reload the page and reopen the file list', async ({ page }) => {
  await page.reload()
  // Let Monaco finish its initial focus before opening the modal; a later
  // editor focus otherwise dismisses the freshly opened file browser.
  await page.waitForSelector('.monaco-editor .view-lines', { timeout: 15_000 })
  await openFileList(page)
})

Then(
  'the file list shows {string} after reload',
  async ({ page }, name: string) => {
    await fileTabByExactName(page, name).waitFor({ timeout: 15_000 })
  },
)

// -- Rename ------------------------------------------------------------

When('the app loads the cloud-backed file list', async ({ page }) => {
  await gotoCloudApp(page)
  await openFileList(page)
})

/** Selects the file this scenario's own `Given` just seeded to make it
 * active -- a freshly loaded cloud backend always starts on the read-only
 * demo file (see `cloudBackend.ts`'s `load()`), which can't be
 * renamed/duplicated/deleted. Selected by its exact tracked name, not
 * "whichever tab is first": the shared `e2e-test-user` account is real and
 * persistent across this whole suite run, so other scenarios' files can
 * already be sitting in the list too (see `currentSeededFileDisplayName`'s
 * doc comment). Then renames it inline via the file list's dblclick-to-edit
 * name field (`FileTabName.tsx`). */
When(
  'I rename the active file to {string}',
  async ({ page }, newName: string) => {
    await openFileList(page)
    const tab = fileTabByExactName(page, currentSeededFileDisplayName())
    await tab.click()
    await page.waitForSelector('.monaco-editor .view-lines', {
      timeout: 15_000,
    })
    await openFileList(page)
    const activeTabName = page.locator('.file-tab--active .file-tab-name')
    await activeTabName.dblclick()
    const input = page.locator('.file-tab--active input.file-tab-name')
    await input.fill(newName)
    await input.press('Enter')
  },
)

Then('the file list does not show {string}', async ({ page }, name: string) => {
  await openFileList(page)
  await expect(fileTabByExactName(page, name)).toHaveCount(0)
})

// -- Delete --------------------------------------------------------------

When(
  'I choose Delete from the file actions menu of the active file',
  async ({ page }) => {
    await openFileList(page)
    const tab = fileTabByExactName(page, currentSeededFileDisplayName())
    await tab.click()
    await page.waitForSelector('.monaco-editor .view-lines', {
      timeout: 15_000,
    })
    await openFileActions(page)
    await page.getByRole('menuitem', { name: 'Delete' }).click()
  },
)

const deleteFileDialog = (page: import('@playwright/test').Page) =>
  page.getByRole('dialog', { name: /^Delete file/ })

Then(
  'the delete file dialog is titled {string}',
  async ({ page }, title: string) => {
    await expect(deleteFileDialog(page)).toContainText(title)
  },
)

When('I confirm the delete file dialog', async ({ page }) => {
  await deleteFileDialog(page)
    .getByRole('button', { name: 'Move to bin' })
    .click()
})

When('I cancel the delete file dialog', async ({ page }) => {
  await deleteFileDialog(page).getByRole('button', { name: 'Cancel' }).click()
  await expect(deleteFileDialog(page)).toHaveCount(0)
})

Then('the file list still shows {string}', async ({ page }, name: string) => {
  await openFileList(page)
  await expect(fileTabByExactName(page, name)).toHaveCount(1)
})

Then('the bin does not show {string}', async ({ page }, name: string) => {
  // The "Bin (N)" menu item only exists while the bin holds something.
  await openFileActions(page)
  if ((await page.locator('.file-tab-bar-bin-trigger').count()) === 0) return
  await openBin(page)
  await expect(binItemNamed(page, name)).toHaveCount(0)
})

Then(
  'the file list no longer shows {string}',
  async ({ page }, name: string) => {
    await openFileList(page)
    await expect(fileTabByExactName(page, name)).toHaveCount(0)
  },
)

// Scoped to the one matching bin item, not a bare `.file-tab-bar-bin-name`
// locator -- the bin lists every trashed file for the account, and the
// shared `e2e-test-user` account can already hold other scenarios' trashed
// files too (see `currentSeededFileDisplayName`'s doc comment), so an
// unscoped locator can resolve to more than one element.
function binItemNamed(page: import('@playwright/test').Page, name: string) {
  return page.locator('.file-tab-bar-bin-name', {
    hasText: new RegExp(`^${name}$`),
  })
}

Then('the bin shows {string}', async ({ page }, name: string) => {
  await openBin(page)
  await expect(binItemNamed(page, name)).toBeVisible()
})

Then('the bin shows {string} after reload', async ({ page }, name: string) => {
  await openBin(page)
  await expect(binItemNamed(page, name)).toBeVisible()
})

// -- Duplicate -------------------------------------------------------------

When('I duplicate the active file', async ({ page }) => {
  await openFileList(page)
  const tab = fileTabByExactName(page, currentSeededFileDisplayName())
  await tab.click()
  await page.waitForSelector('.monaco-editor .view-lines', { timeout: 15_000 })
  await openFileActions(page)
  await duplicateButton({ page }).click()
})

Then(
  'the editor for {string} contains {string}',
  async ({ page }, name: string, text: string) => {
    await openFileList(page)
    await page
      .locator('.file-tab-name', { hasText: new RegExp(`^${name}$`) })
      .click()
    await page.waitForSelector('.monaco-editor .view-lines', {
      timeout: 15_000,
    })
    await expect(page.locator('.monaco-editor .view-lines')).toContainText(text)
  },
)
