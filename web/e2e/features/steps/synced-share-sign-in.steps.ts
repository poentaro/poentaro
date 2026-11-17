import { expect } from '@playwright/test'
import { routeAppCall } from '../../appCalls'
import { Given, Then, When } from './fixtures'
import {
  openSyncedTab,
  waitForSeededSyncedFile,
} from './synced-share-button.steps'
import { syncedShareButtonState } from './synced-share-button-state'

// Covers signing in from the Synced Share modal's prompt
// (synced-share-sign-in.feature) -- the "Synced as <email>" identity row and
// the full-screen session-failure dialog. Kept in its own steps file (not
// synced-share-button.steps.ts) since these steps mock the worker's
// answers directly rather than reusing a pre-seeded session. The account
// dialog itself is driven by `account-auth.steps.ts`'s steps.

Then('the sign-in prompt is shown', async ({ page }) => {
  await expect(page.getByTestId('share-modal-signin-state')).toBeVisible()
})

When('the owner clicks "Sign in" in the prompt', async ({ page }) => {
  await page.getByTestId('share-modal-sign-in').click()
})

Then(
  'the identity row shows signed in as {string}',
  async ({ page }, email: string) => {
    await expect(page.getByTestId('share-modal-identity')).toContainText(
      `Signed in as ${email}`,
    )
  },
)

Then(
  'the synced share identity row reads {string}',
  async ({ page }, text: string) => {
    await expect(page.getByTestId('share-modal-identity')).toContainText(text)
  },
)

// Reject only the start call; the initial sharing-status lookup still reaches
// the worker, so the failure is triggered by the owner's Sync action.
Given(
  'the Synced Share worker rejects the next start-share request with a verification failure',
  async ({ page }) => {
    await routeAppCall(page, 'start sharing file _', (route) =>
      route.fulfill({ status: 401, body: 'Your session has ended.' }),
    )
  },
)

Then(
  'the synced share error dialog is shown with a {string} link',
  async ({ page }, linkText: string) => {
    await expect(page.getByTestId('synced-share-error-dialog')).toBeVisible()
    await expect(
      page.getByTestId('synced-share-error-dialog-issue-link'),
    ).toHaveText(linkText)
  },
)

Then(
  'the synced share error dialog shows the reason {string}',
  async ({ page }, reason: string) => {
    await expect(
      page.getByTestId('synced-share-error-dialog-details'),
    ).toContainText(reason)
  },
)

When('the owner dismisses the synced share error dialog', async ({ page }) => {
  await page.getByTestId('synced-share-error-dialog-close').click()
})

Then('the synced share error dialog is gone', async ({ page }) => {
  await expect(page.getByTestId('synced-share-error-dialog')).toHaveCount(0)
})

// The error dialog closes the share modal as a side effect of opening --
// two independent Radix `Dialog.Root`s open at once fight over the focus
// trap, and the error dialog wins (see `openSyncedTab`'s doc comment in
// `synced-share-button.steps.ts`) -- so scenarios that need to inspect the
// modal's state afterwards must explicitly re-open it first.
When('the owner reopens the share modal', async ({ page }) => {
  await openSyncedTab(page)
})

// Signing in switches a cloud-preferring page onto the cloud backend, but
// the page is left on whatever file it was showing while signed out --
// reopen it on the seeded cloud file, the way a returning owner would.
When(
  'the owner reopens the app on the seeded synced file',
  async ({ page }) => {
    const name = syncedShareButtonState.ownerFileName
    if (!name) throw new Error('no synced cloud file was seeded')
    await page.goto(`/?file=${encodeURIComponent(name)}`)
    await waitForSeededSyncedFile(page)
  },
)
