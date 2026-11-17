import { expect } from '@playwright/test'
import { workerRouteGlob } from '../../cloudFileHelpers'
import {
  fileSwitcherTrigger,
  fileTabByExactName,
  openFileActions,
  openFileList,
  typeAtEditorEnd,
} from '../../fileSwitcherHelpers'
import { gotoCloudApp } from './cloud-account-helpers'
import { installContentSaveTracking } from './cloud-content-save-tracking'
import { Given, Then, When } from './fixtures'

// Mirrors `useStorageBackend.ts`'s `AUTOSAVE_DEBOUNCE_MS`. Not imported
// directly -- that module transitively pulls in `fileStore.ts`'s Vite-only
// `?raw` import, which Playwright's test loader can't resolve.
const AUTOSAVE_DEBOUNCE_MS = 20_000

Given(
  'the first content-save request will be aborted as a network failure',
  async ({ page }) => {
    // Aborting the request (rather than fulfilling with a status) produces
    // the network failure `classifyError` in `cloudBackend.ts` maps to
    // `'offline'`, matching a real offline `fetch`. One-shot: every request
    // after this falls through to the real worker.
    let contentSaveCount = 0
    await page.route(workerRouteGlob('/files/{id}/content'), async (route) => {
      contentSaveCount += 1
      if (contentSaveCount === 1) {
        await route.abort('failed')
        return
      }
      await route.continue()
    })
  },
)

Given(
  'I open and edit {string} with suffix {string} and a fake clock installed',
  async ({ page }, filename: string, suffix: string) => {
    installContentSaveTracking(page)

    // Install the fake clock before navigating -- lets us jump straight
    // past the debounce interval instead of waiting it out for real.
    await page.clock.install()

    await gotoCloudApp(page)

    await openFileList(page)
    const tab = fileTabByExactName(page, filename)
    await tab.waitFor({ timeout: 15_000 })
    await tab.click()
    await expect(fileSwitcherTrigger(page)).toContainText(filename)
    await page.waitForSelector('.monaco-editor .view-lines', {
      timeout: 15_000,
    })
    await page.waitForSelector('.preview-page', { timeout: 15_000 })

    await typeAtEditorEnd(page, suffix)

    // Waits for the edit to actually land in the DOM before the caller
    // jumps the fake clock -- otherwise the clock can advance past the
    // debounce deadline before React's effect has re-run and armed the
    // debounced timer for this edit, so the fast-forward wouldn't trigger a
    // save yet.
    await expect(page.locator('.monaco-editor .view-lines')).toContainText(
      `1 2 3 4${suffix}`,
      { timeout: 10_000 },
    )
  },
)

When(
  'I fast-forward the clock past the autosave debounce interval for the error banner',
  async ({ page }) => {
    await page.clock.fastForward(AUTOSAVE_DEBOUNCE_MS)
  },
)

When(
  'I open the storage settings modal to check the error banner',
  async ({ page }) => {
    await openFileActions(page)
    await page.getByRole('menuitem', { name: 'Storage…' }).click()
    await page.getByTestId('storage-settings-modal').waitFor()
  },
)

Then(
  'the status banner is visible and mentions {string}',
  async ({ page }, text: string) => {
    const banner = page.getByTestId('status-banner')
    await expect(banner).toBeVisible({ timeout: 10_000 })
    await expect(banner).toContainText(text)
  },
)

When('I close the storage settings modal', async ({ page }) => {
  // Close the modal so its overlay stops intercepting clicks on the editor.
  await page.keyboard.press('Escape')
  await expect(page.getByTestId('storage-settings-modal')).toHaveCount(0)
})

When(
  'I append {string} to the editor to trigger a recovering autosave',
  async ({ page }, text: string) => {
    await typeAtEditorEnd(page, text)
  },
)

Then('the editor contains {string}', async ({ page }, text: string) => {
  await expect(page.locator('.monaco-editor .view-lines')).toContainText(text, {
    timeout: 10_000,
  })
})

Then('the status banner is gone', async ({ page }) => {
  // The one-shot failed route has already fired and now falls through to
  // the real worker, so the next autosave succeeds and should clear the
  // banner (`lastError` reset to `null` in `saveContentImpl`'s success
  // path).
  await expect(page.getByTestId('status-banner')).toHaveCount(0)
})
