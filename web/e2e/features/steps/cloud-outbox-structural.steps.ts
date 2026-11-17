import { expect, type Page } from '@playwright/test'
import createClient from 'openapi-fetch'
import type { paths } from '../../../src/generated/live-share-worker/schema'
import {
  CLOUD_WORKER_ORIGIN,
  matchWorkerRoute,
  seedTokenFor,
} from '../../cloudFileHelpers'
import {
  fileSwitcherTrigger,
  fileTabByExactName,
  nameNewFile,
  openFileActions,
  openFileList,
} from '../../fileSwitcherHelpers'
import {
  currentSeededFileDisplayName,
  currentSignedInEmail,
} from './cloud-account.steps'
import { gotoCloudApp } from './cloud-account-helpers'
import { Given, Then, When } from './fixtures'

const worker = createClient<paths>({ baseUrl: CLOUD_WORKER_ORIGIN })
const WORKER_ROUTES = `${CLOUD_WORKER_ORIGIN}/**`
/** The longest retry delay (Appendix A.2: backoff capped at 60 s). */
const OUTBOX_RETRY_CAP_MS = 65_000
const DELIVERY_TIMEOUT_MS = 25_000

const displayName = (fileName: string) => fileName.replace(/\.poentaro$/, '')

/** Loads the app while the worker is still reachable (so the file list
 * loads), then makes every request to the worker origin fail. */
Given('the cloud is unreachable', async ({ page }) => {
  // A fake clock lets "the cloud is reachable again" jump past the outbox's
  // retry delay instead of waiting it out.
  await page.clock.install()
  await gotoCloudApp(page)
  await page.waitForSelector('.monaco-editor .view-lines', { timeout: 15_000 })
  // The editor opens on the demo file before the cloud file list arrives;
  // cutting the worker off before then aborts that load and the seeded
  // files never appear.
  await expect(
    fileSwitcherTrigger(page).locator('.file-tab-bar-spinner'),
  ).toHaveCount(0, { timeout: 15_000 })
  await page.route(WORKER_ROUTES, (route) =>
    // The share-status lookup the owner hook fires for every newly active
    // cloud file is unrelated to the outbox, yet raises a full-screen
    // "failed to sync" dialog when it fails (an app discrepancy reported
    // with this test); let it through so the dialog does not cover the page.
    matchWorkerRoute('/files/{id}/share/status', route.request().url())
      ? route.continue()
      : route.abort('connectionrefused'),
  )
})

When('the cloud is reachable again', async ({ page }) => {
  await page.unroute(WORKER_ROUTES)
  // The browser's own "back online" signal, which wakes the looper.
  await page.evaluate(() => window.dispatchEvent(new Event('online')))
  await page.clock.fastForward(OUTBOX_RETRY_CAP_MS)
})

async function renameActiveFile(page: Page, newName: string): Promise<void> {
  await openFileList(page)
  const activeTabName = page.locator('.file-tab--active .file-tab-name')
  await activeTabName.dblclick()
  const input = page.locator('.file-tab--active input.file-tab-name')
  await input.fill(newName)
  await input.press('Enter')
  // The rename lands a beat after Enter; later steps (seeding, going back
  // online) must not race it.
  await expect(fileSwitcherTrigger(page)).toContainText(
    newName.split('/').at(-1) ?? newName,
  )
}

// "New" asks for the leaf name and creates and activates the file; a name
// with folders is then moved there by a rename (both apply locally at once
// and are queued).
When('I create a file named {string}', async ({ page }, name: string) => {
  await openFileActions(page)
  await page.locator('.export-menu-item').first().click()
  const leaf = name.split('/').at(-1) ?? name
  await nameNewFile(page, leaf)
  if (leaf !== name) await renameActiveFile(page, name)
})

When(
  'I rename {string} to {string}',
  async ({ page }, from: string, to: string) => {
    expect(from).toBe(currentSeededFileDisplayName())
    await openFileList(page)
    await fileTabByExactName(page, from).click()
    await renameActiveFile(page, to)
  },
)

Then(
  '{string} is listed in the file switcher',
  async ({ page }, fileName: string) => {
    await openFileList(page)
    await expect(fileTabByExactName(page, displayName(fileName))).toHaveCount(1)
  },
)

Then(
  'a file named {string} is listed in the file switcher',
  async ({ page }, fileName: string) => {
    await openFileList(page)
    await expect(fileTabByExactName(page, displayName(fileName))).toHaveCount(
      1,
      { timeout: DELIVERY_TIMEOUT_MS },
    )
  },
)

Then(
  '{string} exists on the server for the signed-in account',
  async ({}, fileName: string) => {
    const identityToken = await seedTokenFor(currentSignedInEmail())
    await expect
      .poll(
        async () => {
          const { data } = await worker.POST('/files/list', {
            body: { identityToken },
          })
          return (
            data?.files.some(
              (file) => file.name === fileName && file.trashedAt === null,
            ) ?? false
          )
        },
        { timeout: DELIVERY_TIMEOUT_MS },
      )
      .toBe(true)
  },
)
