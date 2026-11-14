import { expect } from '@playwright/test'
import { routeAppCall } from '../../appCalls'
import {
  fileSwitcherTrigger,
  fileTabByExactName,
  openFileList,
} from '../../fileSwitcherHelpers'
import { gotoCloudApp } from './cloud-account-helpers'
import { Given, Then, When } from './fixtures'

Given(
  'the cloud file-list request is delayed by 5 seconds for the file switcher',
  async ({ page }) => {
    // Delays the `files/list` call `backend.load()` makes so the
    // spinner has a window to be observed before the listing resolves.
    await routeAppCall(page, 'files/list', async (route) => {
      await new Promise((resolve) => setTimeout(resolve, 5000))
      await route.fallback()
    })
  },
)

When(
  'the app loads with the editor ready while cloud files load',
  async ({ page }) => {
    await gotoCloudApp(page)
    await page.waitForSelector('.monaco-editor .view-lines', {
      timeout: 15_000,
    })
  },
)

Then('the file switcher trigger shows a loading spinner', async ({ page }) => {
  const trigger = fileSwitcherTrigger(page)
  await expect(trigger.locator('.file-tab-bar-spinner')).toBeVisible({
    timeout: 5_000,
  })
})

Then(
  'opening the file list shows the hint {string}',
  async ({ page }, hint: string) => {
    await openFileList(page)
    await expect(page.locator('.file-browser-hint')).toHaveText(hint)
  },
)

When('the cloud file-list request resolves', async ({ page }) => {
  const trigger = fileSwitcherTrigger(page)
  await expect(trigger.locator('.file-tab-bar-spinner')).toHaveCount(0, {
    timeout: 15_000,
  })
})

Then(
  'the file switcher trigger spinner is gone and the caret is visible',
  async ({ page }) => {
    const trigger = fileSwitcherTrigger(page)
    await expect(trigger.locator('.export-menu-caret')).toBeVisible()
  },
)

Then('the file list shows {string}', async ({ page }, name: string) => {
  await openFileList(page)
  await expect(fileTabByExactName(page, name)).toBeVisible()
})
