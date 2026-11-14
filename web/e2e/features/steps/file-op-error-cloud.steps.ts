import { expect } from '@playwright/test'
import { routeAppCall } from '../../appCalls'
import {
  fileTabByExactName,
  nameNewFile,
  openFileActions,
  openFileList,
} from '../../fileSwitcherHelpers'
import { gotoCloudApp } from './cloud-account-helpers'
import { Given, Then, When } from './fixtures'

const newButton = ({ page }: { page: import('@playwright/test').Page }) =>
  page.locator('.export-menu-item').first()

Given(
  'the first file-create request will fail with a 500 error',
  async ({ page }) => {
    // Registered before navigating: fails the first `files/create` call
    // with a 500, then lets every call after through (including a retried
    // create), so the real worker behaves normally once this one-shot
    // failure has fired.
    let createCount = 0
    await routeAppCall(page, 'files/create _ at _ content _', async (route) => {
      createCount += 1
      if (createCount === 1) {
        await route.fulfill({ status: 500, body: 'Internal Server Error' })
        return
      }
      await route.fallback()
    })
  },
)

When(
  'the app loads the cloud-backed file list for a failing create',
  async ({ page }) => {
    await gotoCloudApp(page)
    await openFileList(page)
    const existingTab = fileTabByExactName(page, 'existing')
    await existingTab.waitFor({ timeout: 15_000 })
  },
)

When(
  'I click the {string} button to create a file that will fail',
  async ({ page }, label: string) => {
    expect(label).toBe('New')
    await openFileActions(page)
    await newButton({ page }).click()
    await nameNewFile(page, 'untitled')
  },
)

Then(
  'the new-file button spinner clears and its label resets to {string}',
  async ({ page }, label: string) => {
    // `finally`'s `setPending(false)` must run on the error path too, not
    // just on success -- otherwise the "New" button would be stuck
    // spinning. The dropdown closes once `onCreate` settles (success or
    // failure), so reopen it to observe the reset state.
    await openFileActions(page)
    await expect(
      newButton({ page }).locator('.file-tab-bar-spinner'),
    ).toHaveCount(0)
    await expect(newButton({ page })).toHaveText(label)
  },
)

Then('no error modal is shown', async ({ page }) => {
  await expect(page.getByTestId('error-modal')).toHaveCount(0)
})

Then('an {string} tab exists', async ({ page }, name: string) => {
  // The create is applied locally at once and delivered by the outbox later.
  await openFileList(page)
  await expect(fileTabByExactName(page, name)).toHaveCount(1)
})
