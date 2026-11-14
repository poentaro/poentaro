import { expect } from '@playwright/test'
import { routeAppCall } from '../../appCalls'
import { openFileActions } from '../../fileSwitcherHelpers'
import { Given, Then, When } from './fixtures'

Given(
  'the cloud file-list request is delayed by 1 second when switching backend',
  async ({ page }) => {
    // Delays the `files/list` call `backend.load()` makes so the
    // spinner has a window to be observed before the listing resolves.
    await routeAppCall(page, 'files/list', async (route) => {
      await new Promise((resolve) => setTimeout(resolve, 1000))
      await route.fallback()
    })
  },
)

When(
  'the app loads on the local backend with the editor ready',
  async ({ page }) => {
    await page.goto('/')
    await page.waitForSelector('.monaco-editor .view-lines', {
      timeout: 15_000,
    })
  },
)

When(
  'I open the storage settings modal to switch backend',
  async ({ page }) => {
    await openFileActions(page)
    await page.getByRole('menuitem', { name: 'Storage…' }).click()
    await page.getByTestId('storage-settings-modal').waitFor()
  },
)

Then('the cloud loading spinner is visible', async ({ page }) => {
  const spinner = page.getByTestId('cloud-loading-spinner')
  await expect(spinner).toBeVisible({ timeout: 5_000 })
})

Then(
  'the cloud loading spinner disappears once loading finishes',
  async ({ page }) => {
    const spinner = page.getByTestId('cloud-loading-spinner')
    await expect(spinner).toHaveCount(0, { timeout: 15_000 })
  },
)
