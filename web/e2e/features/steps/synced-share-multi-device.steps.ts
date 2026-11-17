import { type BrowserContext, expect, type Page } from '@playwright/test'
import { seedCloudFile } from '../../cloudFileHelpers'
import { seedSignedInSession } from './cloud-account-helpers'
import { AfterScenario, Given, Then, When } from './fixtures'
import {
  openSyncedTab,
  waitForSeededSyncedFile,
} from './synced-share-button.steps'
import {
  SYNCED_FILE_BASE_NAME,
  SYNCED_SOURCE,
  syncedShareButtonState,
} from './synced-share-button-state'

// Covers one account signed in on several devices at once
// (synced-share-sign-in.feature's multi-device scenario). Each device is
// its own isolated browser context -- its own localStorage, so its own
// session, the way two real browsers would be.

interface Device {
  context: BrowserContext
  page: Page
}

const devices = new Map<string, Device>()
let signedInEmail: string | undefined

AfterScenario(async () => {
  await Promise.all([...devices.values()].map(({ context }) => context.close()))
  devices.clear()
  signedInEmail = undefined
})

function deviceNamed(name: string): Device {
  const device = devices.get(name)
  if (!device) throw new Error(`device ${JSON.stringify(name)} was not opened`)
  return device
}

Given(
  'the file store is seeded with the synced score for {string}',
  async ({}, email: string) => {
    const name = `${SYNCED_FILE_BASE_NAME}-${crypto.randomUUID().slice(0, 8)}.poentaro`
    const file = await seedCloudFile(email, name, SYNCED_SOURCE)
    syncedShareButtonState.ownerFileName = name
    syncedShareButtonState.ownerFileId = file.id
  },
)

Given(
  'the owner signs in as {string} on device {string}',
  async ({ browser }, email: string, name: string) => {
    const context = await browser.newContext()
    await context.grantPermissions(['clipboard-read', 'clipboard-write'])
    await context.addInitScript(() => {
      localStorage.setItem(
        'poentaro:storage-backend:v1',
        JSON.stringify({ backend: 'cloud' }),
      )
    })
    await seedSignedInSession(context, email)
    const page = await context.newPage()
    devices.set(name, { context, page })
    signedInEmail = email

    await page.goto('/')
    await expect(page.getByTestId('account-chip')).toContainText(email)
  },
)

/** Opens the app on the scenario's seeded file, the way a returning owner
 * would, and waits for it to be the file on screen. */
async function openSeededFile(page: Page): Promise<void> {
  const fileName = syncedShareButtonState.ownerFileName
  if (!fileName) throw new Error('no synced cloud file was seeded')
  await page.goto(`/?file=${encodeURIComponent(fileName)}`)
  await waitForSeededSyncedFile(page)
}

Given(
  'the owner starts syncing the seeded file on device {string}',
  async ({}, name: string) => {
    const { page } = deviceNamed(name)
    await openSeededFile(page)
    await openSyncedTab(page)
    await page.getByTestId('share-modal-start-sync').click()
    await expect(page.getByTestId('share-modal-stop-sync')).toBeVisible()
  },
)

When('the owner signs out on device {string}', async ({}, name: string) => {
  const { page } = deviceNamed(name)
  // Close the share modal first: its overlay blocks the header's account
  // chip (see `closeShareModal` in `account-chip.steps.ts`).
  await page.keyboard.press('Escape')
  await expect(page.getByTestId('share-modal')).toHaveCount(0)
  await page.getByTestId('account-chip').click()
  // Sign-out tells the worker without awaiting it -- wait for it here, so
  // later steps on other devices can't race ahead of it.
  await Promise.all([
    page.waitForResponse((response) =>
      response.url().endsWith('/auth/sign-out'),
    ),
    page
      .getByTestId('account-popover')
      .getByRole('button', { name: 'Sign out' })
      .click(),
  ])
})

When(
  'the owner reopens the seeded file on device {string}',
  async ({}, name: string) => {
    const fileName = syncedShareButtonState.ownerFileName
    if (!fileName) throw new Error('no synced cloud file was seeded')
    await deviceNamed(name).page.goto(`/?file=${encodeURIComponent(fileName)}`)
  },
)

// A rejected session surfaces as either the Synced Share error dialog (its
// share-status lookup is rejected) or a signed-out share modal (the cloud
// backend dropped the rejected sign-in first) -- checking for the live
// share and the identity row catches both.
Then(
  'device {string} is still signed in and syncing the seeded file',
  async ({}, name: string) => {
    const { page } = deviceNamed(name)
    await waitForSeededSyncedFile(page)
    await openSyncedTab(page)
    await expect(page.getByTestId('share-modal-stop-sync')).toBeVisible()
    await expect(page.getByTestId('share-modal-identity')).toContainText(
      `${signedInEmail}`,
    )
    await expect(page.getByTestId('synced-share-error-dialog')).toHaveCount(0)
  },
)
