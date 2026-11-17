import { expect, type Page } from '@playwright/test'
import { workerRouteGlob } from '../../cloudFileHelpers'
import { fileTabByExactName, openFileList } from '../../fileSwitcherHelpers'
import { attemptEditAndForceSave } from './account-sign-in.steps'
import { Given, Then, When } from './fixtures'

// Covers the header's persistent account chip (account-chip.feature) --
// see HANDOFF-account-chip.md. The chip's own signed-out/signed-in/syncing
// states and its profile popover; sign-out through the popover is also
// exercised by account-auth.feature, account-sign-in.feature and
// synced-share-sign-in.feature, whose "click the account chip" /
// "click \"Sign out\" in the profile popover" steps live here too, shared
// across all three feature files via playwright-bdd's global step registry
// (same pattern as `openSyncedTab` being shared from
// synced-share-button.steps.ts).

When('the app loads for the account chip flow', async ({ page }) => {
  await page.goto('/')
})

Then(
  'the header shows a {string} button and no account chip',
  async ({ page }, label: string) => {
    await expect(
      page.getByRole('button', { name: label, exact: true }),
    ).toBeVisible()
    await expect(page.getByTestId('account-chip')).toHaveCount(0)
  },
)

When('I click {string} in the header', async ({ page }, label: string) => {
  await page.getByRole('button', { name: label, exact: true }).click()
})

Then(
  'the header shows the account chip labeled {string}',
  async ({ page }, label: string) => {
    // Signing in waits on the worker's password check before the chip
    // re-renders -- allow longer than Playwright's default 5s.
    await expect(page.getByTestId('account-chip')).toBeVisible({
      timeout: 10_000,
    })
    await expect(page.getByTestId('account-chip')).toContainText(label)
  },
)

Then(
  'the header no longer shows a {string} button',
  async ({ page }, label: string) => {
    await expect(
      page.getByRole('button', { name: label, exact: true }),
    ).toHaveCount(0)
  },
)

async function clickAccountChip(page: Page) {
  await page.getByTestId('account-chip').click()
}

When('I click the account chip', async ({ page }) => {
  await clickAccountChip(page)
})

When('the owner clicks the account chip', async ({ page }) => {
  await clickAccountChip(page)
})

// The account chip lives in the header, behind `ShareModal`'s Radix
// `Dialog.Overlay` whenever that modal is left open (e.g. right after
// clicking "Sync") -- the overlay blocks pointer events on everything
// beneath it, so a scenario that syncs first must close the modal before
// it can click the chip. Mirrors the `Escape`-to-close approach
// `account-sign-in.steps.ts`'s "I close the storage settings modal and
// attempt to edit and force-save" step already uses for the same reason
// with `StorageSettingsModal`.
async function closeShareModal(page: Page) {
  await page.keyboard.press('Escape')
  await expect(page.getByTestId('share-modal')).toHaveCount(0)
}

When('I close the share modal', async ({ page }) => {
  await closeShareModal(page)
})

When('the owner closes the share modal', async ({ page }) => {
  await closeShareModal(page)
})

Then('the profile popover shows {string}', async ({ page }, text: string) => {
  await expect(page.getByTestId('account-popover')).toBeVisible()
  await expect(page.getByTestId('account-popover')).toContainText(text)
})

Then(
  'the header account chip shows the syncing indicator',
  async ({ page }) => {
    await expect(
      page.getByTestId('account-chip-syncing-indicator'),
    ).toBeVisible()
  },
)

async function clickPopoverButton(page: Page, label: string) {
  await page
    .getByTestId('account-popover')
    .getByRole('button', { name: label })
    .click()
}

When(
  'I click {string} in the profile popover',
  async ({ page }, label: string) => {
    await clickPopoverButton(page, label)
  },
)

When(
  'the owner clicks {string} in the profile popover',
  async ({ page }, label: string) => {
    await clickPopoverButton(page, label)
  },
)

Then(
  'the storage settings modal shows the sign-in prompt',
  async ({ page }) => {
    await expect(page.getByTestId('account-sign-in')).toBeVisible()
  },
)

Then(
  'the storage settings modal hides the sign-in prompt',
  async ({ page }) => {
    await expect(page.getByTestId('account-sign-in')).toHaveCount(0)
  },
)

Given(
  'the stored storage-backend preference is {string}',
  async ({ page }, backend: string) => {
    await page.addInitScript((backend: string) => {
      localStorage.setItem(
        'poentaro:storage-backend:v1',
        JSON.stringify({ backend }),
      )
    }, backend)
  },
)

Then(
  'the stored storage-backend preference is set to local',
  async ({ page }) => {
    const preference = await page.evaluate(() =>
      localStorage.getItem('poentaro:storage-backend:v1'),
    )
    expect(JSON.parse(preference ?? '{}')).toEqual({ backend: 'local' })
  },
)

When('I select the {string} tab', async ({ page }, name: string) => {
  const tab = fileTabByExactName(page, name)
  await tab.click()
  await page.waitForSelector('.monaco-editor .view-lines', { timeout: 15_000 })
})

// Registered before navigating, same reasoning as
// account-sign-in.steps.ts's "the app loads the cloud-backed file list for
// disconnect" -- needs to be in place for the whole scenario, including the
// post-sign-out edit attempt at the very end (see "no content request is
// sent to the worker after signing out" below). Kept as its own module-level
// array rather than reusing account-sign-in.steps.ts's, since each
// `.steps.ts` file's own scenario controls when it gets reset (mirrors that
// file's own doc comment on why this is module-level, not a fixture).
let contentSaveRequests: string[] = []

When(
  'the app loads the cloud-backed file list for the account chip flow',
  async ({ page }) => {
    contentSaveRequests = []
    await page.route(workerRouteGlob('/files/{id}/content'), async (route) => {
      contentSaveRequests.push(route.request().url())
      await route.continue()
    })
    await page.goto('/')
    await openFileList(page)
    const tab = fileTabByExactName(page, 'song')
    await tab.waitFor({ timeout: 15_000 })
  },
)

When('I edit and force-save', async ({ page }) => {
  await attemptEditAndForceSave(page)
})

Then('no content request is sent to the worker after signing out', async () => {
  // Give any (incorrect) autosave/force-save to the cloud backend a chance
  // to fire; the route handler above records it if it does.
  await new Promise((resolve) => setTimeout(resolve, 500))
  expect(contentSaveRequests).toHaveLength(0)
})
