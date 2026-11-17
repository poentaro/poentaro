import type { Browser, Page } from '@playwright/test'
import { expect } from '@playwright/test'
import { seedCloudFile, workerRouteGlob } from '../../cloudFileHelpers'
import { DEFAULT_E2E_EMAIL } from '../../e2eAccounts.ts'
import {
  fileSwitcherTrigger,
  fileTabByExactName,
  openFileList,
} from '../../fileSwitcherHelpers'
import { gotoCloudApp, seedSignedInSession } from './cloud-account-helpers'
import { AfterScenario, BeforeScenario, Given, Then, When } from './fixtures'
import { openSyncedTab } from './synced-share-button.steps'
import { syncedShareButtonState as sharedState } from './synced-share-button-state'
import {
  actualNameFor,
  assignActualName,
  idempotentSharingSource,
  resetIdempotentCreateState,
  idempotentCreateState as state,
} from './synced-share-idempotent-create-state'

BeforeScenario(async () => {
  resetIdempotentCreateState()
})

AfterScenario(async () => {
  if (state.secondContext) {
    await state.secondContext.close()
    state.secondContext = undefined
  }
})

Given(
  'a cloud file named {string} is seeded for idempotent sharing',
  async ({}, name: string) => {
    const displayName = name.replace(/\.poentaro$/, '')
    const file = await seedCloudFile(
      DEFAULT_E2E_EMAIL,
      `${assignActualName(displayName)}.poentaro`,
      idempotentSharingSource(name),
    )
    state.seededFileIds[displayName] = file.id
  },
)

When(
  'the app loads the cloud-backed file list for idempotent sharing',
  async ({ page }) => {
    await gotoCloudApp(page)
    await openFileList(page)
  },
)

/** Closes the share modal if a prior "Sync" click in this scenario left it
 * open -- Radix's Dialog overlay intercepts pointer events on the rest of
 * the page while open, so the header's file-switcher trigger (needed by
 * both this step and the rename step below) can't be clicked until it's
 * gone. Closing it only hides the UI; it doesn't stop an active sync (that
 * lives in React state / localStorage, untouched by the modal's own
 * open/closed state), so a later step re-opens it via `openSyncedTab`
 * whenever it needs to check or use it again. */
async function closeShareModalIfOpen(page: Page): Promise<void> {
  const modal = page.getByTestId('share-modal')
  if ((await modal.count()) === 0) return
  await page.keyboard.press('Escape')
  await modal.waitFor({ state: 'hidden' })
}

When(
  'I select the {string} tab to test idempotent sharing',
  async ({ page }, name: string) => {
    await closeShareModalIfOpen(page)
    await openFileList(page)
    // Exact-name match, not a substring `hasText` locator -- the cloud
    // account backing this feature is real and shared across the whole
    // e2e suite run (see `fileTabByExactName`'s own doc comment), so
    // "idempotent" seeded here can otherwise ambiguously match another
    // scenario's "idempotent-a"/"idempotent-b" tabs too.
    const actualName = actualNameFor(name)
    const tab = fileTabByExactName(page, actualName)
    await tab.waitFor({ timeout: 15_000 })
    await tab.click()
    await expect(fileSwitcherTrigger(page)).toContainText(actualName)
    await page.waitForSelector('.monaco-editor .view-lines', {
      timeout: 15_000,
    })
    await page.waitForSelector('.preview-page', { timeout: 15_000 })
    state.activeTabName = name
  },
)

/** Injects a synthetic `/files/list` entry carrying *another* account's
 * real D1 file id -- see `loadSecondContextOnCloudFile`'s doc comment for
 * why this is the only way to send a share request for someone else's file
 * through the real worker: D1's `files.id` is a globally unique primary
 * key, so two different owners can never really share one row. */
async function mockCloudFileListWithForeignFile(
  page: Page,
  fileId: string,
  tabName: string,
): Promise<void> {
  await page.route(workerRouteGlob('/files/list'), async (route) => {
    await route.fulfill({
      status: 200,
      json: {
        files: [
          {
            id: fileId,
            name: `${actualNameFor(tabName)}.poentaro`,
            content: idempotentSharingSource(tabName),
            revision: 0,
            trashedAt: null,
          },
        ],
      },
    })
  })
}

/** Seeds a second browser context's `localStorage` the same way "a cloud
 * file ... is seeded for idempotent sharing" (storage backend) and "the
 * owner is signed in as ..." (account session) do for
 * the main page, then opens the given cloud-backed tab and the share
 * modal's Synced-link tab. `context.addInitScript` (not
 * `page.addInitScript`) so it applies before this context's very first
 * navigation.
 *
 * When `email` is the same account that owns the seeded file, this loads
 * the file through the real cloud backend, same as the owner's own page --
 * a real D1 row, a real `/files/list` fetch. When `email` differs, no real
 * account can see another account's row (D1 file ownership is strictly
 * per-owner), so `mockCloudFileListWithForeignFile` above hands this
 * context a synthetic listing carrying the original owner's real file id
 * instead -- see that function's doc comment. */
async function loadSecondContextOnCloudFile(
  browser: Browser,
  email: string,
  tabName: string,
): Promise<Page> {
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
  if (email !== DEFAULT_E2E_EMAIL) {
    const fileId = state.seededFileIds[tabName]
    if (!fileId) {
      throw new Error(
        `loadSecondContextOnCloudFile: no seeded cloud file id tracked for tab ${JSON.stringify(tabName)}`,
      )
    }
    await mockCloudFileListWithForeignFile(page, fileId, tabName)
  }
  await page.goto('/')
  await openFileList(page)
  const tab = fileTabByExactName(page, actualNameFor(tabName))
  await tab.waitFor({ timeout: 15_000 })
  await tab.click()
  await page.waitForSelector('.preview-page', { timeout: 15_000 })
  await openSyncedTab(page)
  state.secondContext = context
  state.secondPage = page
  return page
}

When(
  'a separate browser context loads the same cloud-backed file, signed in as the same account {string}',
  async ({ browser }, email: string) => {
    if (!state.activeTabName) {
      throw new Error('no cloud-backed tab has been selected yet')
    }
    await loadSecondContextOnCloudFile(browser, email, state.activeTabName)
  },
)

When(
  'a separate browser context loads the same cloud-backed file, signed in as a different account {string}',
  async ({ browser }, email: string) => {
    if (!state.activeTabName) {
      throw new Error('no cloud-backed tab has been selected yet')
    }
    await loadSecondContextOnCloudFile(browser, email, state.activeTabName)
  },
)

When(
  'a separate browser context loads the cloud-backed file at its renamed path, signed in as the same account {string}',
  async ({ browser }, email: string) => {
    await loadSecondContextOnCloudFile(browser, email, 'renamed')
  },
)

When(
  'the owner clicks {string} in that separate browser context',
  async ({}, label: string) => {
    expect(label).toBe('Sync')
    const page = state.secondPage
    if (!page) throw new Error('second browser context was not opened yet')
    // The file may already be live (the server holds its share state), in
    // which case the modal offers the copy button instead of Start Sync.
    const startSync = page.getByTestId('share-modal-start-sync')
    if (await startSync.count()) await startSync.click()
    else await page.getByTestId('share-modal-copy-synced-link').click()
    await expect(page.getByTestId('share-modal-copy-synced-link')).toHaveText(
      'Link copied',
    )
    state.secondSyncedShareLink = await page.evaluate(() =>
      navigator.clipboard.readText(),
    )
  },
)

Then(
  'the synced link copied in the separate browser context is identical to the original link',
  async () => {
    if (!sharedState.originalSyncedLink) {
      throw new Error('originalSyncedLink was not captured yet')
    }
    if (!state.secondSyncedShareLink) {
      throw new Error('secondSyncedShareLink was not captured yet')
    }
    expect(state.secondSyncedShareLink).toEqual(sharedState.originalSyncedLink)
  },
)

Then(
  "the separate browser context's share modal already shows the stop-sync button",
  async () => {
    const page = state.secondPage
    if (!page) throw new Error('second browser context was not opened yet')
    await expect(page.getByTestId('share-modal-stop-sync')).toBeVisible()
  },
)

When(
  'the other account clicks {string} in that separate browser context',
  async ({}, label: string) => {
    expect(label).toBe('Start Sync')
    const page = state.secondPage
    if (!page) throw new Error('second browser context was not opened yet')
    await page.getByTestId('share-modal-start-sync').click()
  },
)

Then(
  'the separate browser context shows the synced share error dialog',
  async () => {
    const page = state.secondPage
    if (!page) throw new Error('second browser context was not opened yet')
    await expect(page.getByTestId('synced-share-error-dialog')).toBeVisible()
  },
)

Then('the share modal shows the cloud-only message', async ({ page }) => {
  await expect(page.getByTestId('share-modal-synced-cloud-only')).toBeVisible()
  await expect(page.getByTestId('share-modal-start-sync')).toHaveCount(0)
})

Then(
  'the synced link copied in the separate browser context has the same share id as the original link',
  async () => {
    if (!sharedState.originalSyncedLink) {
      throw new Error('originalSyncedLink was not captured yet')
    }
    if (!state.secondSyncedShareLink) {
      throw new Error('secondSyncedShareLink was not captured yet')
    }
    expect(syncedShareId(state.secondSyncedShareLink)).toEqual(
      syncedShareId(sharedState.originalSyncedLink),
    )
  },
)

Then('the copied sync link is different from the original link', async () => {
  if (!sharedState.originalSyncedLink) {
    throw new Error('originalSyncedLink was not captured yet')
  }
  if (!sharedState.syncedShareLink) {
    throw new Error('syncedShareLink was not captured yet')
  }
  expect(sharedState.syncedShareLink).not.toEqual(
    sharedState.originalSyncedLink,
  )
})

When(
  'the owner renames the active file to {string}',
  async ({ page }, newName: string) => {
    await closeShareModalIfOpen(page)
    await openFileList(page)
    const activeTab = page.locator('.file-tab--active .file-tab-name')
    await activeTab.dblclick()
    const input = page.locator('.file-tab--active input.file-tab-name')
    await input.fill(assignActualName(newName))
    await input.press('Enter')
    // Unlike the deleted GitHub-mock version of this step, there's no
    // in-memory seed map to keep in sync -- this is a real rename against
    // the running worker+D1 (`cloudBackend.ts`'s `renameFile`), so a later
    // "a separate browser context loads the cloud-backed file at its
    // renamed path ..." step (same account) sees the new name simply by
    // fetching a fresh `/files/list` for real.
    state.activeTabName = newName
  },
)

Then('the share modal still shows the stop-sync button', async ({ page }) => {
  // Reopen the modal -- the rename step above closed it to reach the
  // header's file-switcher trigger (see `closeShareModalIfOpen`'s own
  // comment); the sync itself was never affected by the modal's visibility.
  await openSyncedTab(page)
  await expect(page.getByTestId('share-modal-stop-sync')).toBeVisible()
})

/** Extracts just the share id from a synced-share URL --
 * `buildSyncedShareUrl`'s human-readable filename suffix is cosmetic only
 * (see its own doc comment: "a rename after sharing doesn't invalidate the
 * link, it just makes this cosmetic copy stale"), so "unchanged after the
 * rename" means the id, not the whole URL string. */
function syncedShareId(link: string): string {
  const shareId = new URL(link).searchParams.get('s')?.slice(0, 11)
  if (!shareId || !/^[0-9A-Za-z_-]{11}$/.test(shareId))
    throw new Error(`not a synced share link: ${link}`)
  return shareId
}

Then('the synced link is unchanged after the rename', async ({ page }) => {
  if (!sharedState.originalSyncedLink) {
    throw new Error('originalSyncedLink was not captured yet')
  }
  await page.getByTestId('share-modal-copy-synced-link').click()
  const copied = await page.evaluate(() => navigator.clipboard.readText())
  expect(syncedShareId(copied)).toEqual(
    syncedShareId(sharedState.originalSyncedLink),
  )
})
