import { expect, type Page } from '@playwright/test'
import { workerRouteGlob } from '../../cloudFileHelpers'
import {
  fileTabByExactName,
  focusEditor,
  openFileActions,
  openFileList,
} from '../../fileSwitcherHelpers'
import { gotoCloudApp } from './cloud-account-helpers'
import { Then, When } from './fixtures'

When('the app loads for the account sign-in flow', async ({ page }) => {
  await page.goto('/')
})

When('I open the storage settings modal for sign-in', async ({ page }) => {
  await openFileActions(page)
  await page.getByRole('menuitem', { name: 'Storage…' }).click()
  await page.getByTestId('storage-settings-modal').waitFor()
})

When('I click "Sign in" in the storage settings modal', async ({ page }) => {
  await page
    .getByTestId('storage-settings-modal')
    .getByRole('button', { name: 'Sign in', exact: true })
    .click()
})

Then(
  'the storage settings modal shows connected as {string}',
  async ({ page }, email: string) => {
    await expect(page.getByTestId('account-connected')).toBeVisible({
      timeout: 10_000,
    })
    await expect(page.getByTestId('account-connected')).toContainText(email)
  },
)

When(
  'I select the {string} storage option',
  async ({ page }, label: string) => {
    await page.getByRole('button', { name: label }).click()
  },
)

Then(
  'the stored storage-backend preference is set to cloud',
  async ({ page }) => {
    const preference = await page.evaluate(() =>
      localStorage.getItem('poentaro:storage-backend:v1'),
    )
    expect(JSON.parse(preference ?? '{}')).toEqual({ backend: 'cloud' })
  },
)

let contentSaveRequests: string[] = []

When(
  'the app loads the cloud-backed file list for disconnect',
  async ({ page }) => {
    contentSaveRequests = []
    // Registered before navigating so it's in place for the whole scenario,
    // including the post-disconnect edit attempt at the very end -- see
    // "no content request is sent to the worker after disconnecting" below.
    await page.route(workerRouteGlob('/files/{id}/content'), async (route) => {
      contentSaveRequests.push(route.request().url())
      await route.continue()
    })
    await gotoCloudApp(page)
    await openFileList(page)
    const tab = fileTabByExactName(page, 'song')
    await tab.waitFor({ timeout: 15_000 })
  },
)

When(
  'I select the {string} tab before disconnecting',
  async ({ page }, name: string) => {
    const tab = fileTabByExactName(page, name)
    await tab.click()
    await page.waitForSelector('.monaco-editor .view-lines', {
      timeout: 15_000,
    })
  },
)

When('I click the {string} button', async ({ page }, label: string) => {
  await page.getByRole('button', { name: label }).click()
})

Then('the stored account auth is cleared', async ({ page }) => {
  const storedAuth = await page.evaluate(() =>
    localStorage.getItem('poentaro:account-auth:v2'),
  )
  // `accountAuth.ts`'s state is a `usehooks-ts` `useLocalStorage` hook --
  // `setAccountAuth(null)` writes the JSON-stringified value `"null"`
  // rather than removing the key outright, so the cleared state reads back
  // as the *string* `"null"`, not an absent/`null` `localStorage.getItem`.
  expect(JSON.parse(storedAuth ?? '"unset"')).toBeNull()
})

Then('the app no longer shows as connected', async ({ page }) => {
  await expect(page.getByTestId('account-connected')).toHaveCount(0)
})

Then(
  'the {string} storage option is checked',
  async ({ page }, label: string) => {
    await expect(page.getByRole('button', { name: label })).toHaveAttribute(
      'aria-pressed',
      'true',
    )
  },
)

// Disconnecting switches the active file to the read-only reference/demo
// file (`isReadOnlyFile` in `fileStore.ts`), so this attempted edit is a
// no-op in the editor itself -- the assertion afterwards only cares that
// force-saving doesn't hit the worker, regardless of whether the keystrokes
// changed anything. Exported so `account-chip.steps.ts` can reuse it for its
// own "signing out while cloud storage is active" scenario (that one signs
// out via the header chip directly, with no storage-settings modal to close
// first) -- mirrors this file's own `openSyncedTab` cross-import pattern in
// `synced-share-sign-in.steps.ts`.
export async function attemptEditAndForceSave(page: Page) {
  await focusEditor(page)
  await page.keyboard.press('Control+End')
  await page.keyboard.type(' edited')
  await page.keyboard.press('ControlOrMeta+s')
}

When(
  'I close the storage settings modal and attempt to edit and force-save',
  async ({ page }) => {
    await page.keyboard.press('Escape')
    await attemptEditAndForceSave(page)
  },
)

When('I attempt to edit and force-save', async ({ page }) => {
  await attemptEditAndForceSave(page)
})

Then(
  'no content request is sent to the worker after disconnecting',
  async () => {
    // Give any (incorrect) autosave/force-save to the cloud backend a
    // chance to fire; the route handler above records it if it does.
    await new Promise((resolve) => setTimeout(resolve, 500))
    expect(contentSaveRequests).toHaveLength(0)
  },
)
