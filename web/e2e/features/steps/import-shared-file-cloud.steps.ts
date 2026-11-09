import { expect } from '@playwright/test'
import { gotoShareUrl } from '../../shareUrlHelper'
import { Given, Then, When } from './fixtures'

const SHARED_SOURCE = [
  'metadata { title "Shared Score" },',
  '',
  'parts ["Melody": notes],',
  '',
  'score """',
  '    (time=4/4 key=C4 bpm=120)',
  '    1 2 3 4',
  '    """,',
].join('\n')

/** Seeds only the `'cloud'` storage-backend preference, without navigating
 * -- unlike `gotoCloudApp` (`cloud-account-helpers.ts`), this scenario's
 * first real navigation is `gotoShareUrl` below, not a plain `page.goto('/')`. */
Given('the cloud storage backend preference is set', async ({ page }) => {
  await page.addInitScript(() => {
    localStorage.setItem(
      'poentaro:storage-backend:v1',
      JSON.stringify({ backend: 'cloud' }),
    )
  })
})

When(
  'I navigate to the share URL for {string}',
  async ({ page }, filename: string) => {
    await gotoShareUrl(page, filename, SHARED_SOURCE)
    await page.waitForSelector('.preview-page', { timeout: 15_000 })
  },
)

Then('the shared-preview banner is visible', async ({ page }) => {
  await expect(page.locator('.shared-preview-banner')).toBeVisible()
})

Then('the shared-preview banner is gone', async ({ page }) => {
  await expect(page.locator('.shared-preview-banner')).toHaveCount(0)
})
