import { expect, type Page } from '@playwright/test'
import { fileSwitcherTrigger, openFileActions } from '../../fileSwitcherHelpers'
import {
  ACCOUNT_AUTH_STORAGE_KEY,
  seedSignedInSession,
} from './cloud-account-helpers'
import { AfterScenario, Given, Then, When } from './fixtures'
import {
  SYNCED_FILE_BASE_NAME,
  SYNCED_SOURCE,
  seedSyncedCloudFile,
  syncedShareButtonState as state,
} from './synced-share-button-state'

/** Opens the share modal's Synced-link tab -- opening the file-actions menu
 * and clicking "Share" first if the modal isn't already open (it's left
 * open across several steps within one scenario, since only the viewer's
 * page reloads, not the owner's -- except when a `SyncedShareErrorDialog`
 * has opened on top of it, which closes it: two independent Radix
 * `Dialog.Root`s open at once fight over the focus trap, and the newer one
 * wins, so the share modal's `onOpenChange` fires with `false`. Exported for
 * `synced-share-sign-in.steps.ts`, which re-opens the modal after
 * dismissing that error dialog for the same reason). */
export async function openSyncedTab(page: Page) {
  const modal = page.getByTestId('share-modal')
  if ((await modal.count()) === 0) {
    await openFileActions(page)
    await page.getByTestId('share-button').click()
  }
  await page.getByTestId('share-modal-synced-tab').click()
}

// Mirrors `useStorageBackend.ts`'s `AUTOSAVE_DEBOUNCE_MS` -- the cloud
// autosave is the only thing that ever changes what a viewer sees. Not imported
// directly — that module transitively pulls in `fileStore.ts`'s Vite-only
// `?raw` import, which Playwright's test loader can't resolve (see the same
// note in `autosave-github.steps.ts`).
const AUTOSAVE_DEBOUNCE_MS = 20_000

// `viewerContext` comes from `browser.newContext()`, which — unlike the
// per-test `context`/`page` fixtures — Playwright never closes on its own,
// so every scenario that opens one must close it itself once done. Doing
// that here (rather than in whichever assertion step happens to run last)
// means it's not tied to a particular scenario's step order.
AfterScenario(async () => {
  if (state.viewerContext) {
    await state.viewerContext.close()
    state.viewerContext = undefined
  }
  if (state.lateViewerContext) {
    await state.lateViewerContext.close()
    state.lateViewerContext = undefined
  }
})

Given('clipboard permissions are granted', async ({ context }) => {
  state.syncedShareLink = undefined
  state.originalSyncedLink = undefined
  state.ownerFileName = undefined
  state.ownerFileId = undefined
  state.viewerPage = undefined
  state.lateViewerPage = undefined
  state.viewerContext = undefined
  state.lateViewerContext = undefined
  await context.grantPermissions(['clipboard-read', 'clipboard-write'])
})

Given('the file store is seeded with the synced score', async ({ page }) => {
  await seedSyncedCloudFile(page, SYNCED_FILE_BASE_NAME, SYNCED_SOURCE)
})

Given(
  'the file store is seeded with a two-section synced score',
  async ({ page }) => {
    // Mirrors `section-jump-select.steps.ts`'s two-section fixture, but with
    // `[M]`-prefixed lines (not the bare `M = notes` shorthand) so each note
    // gets its own click-target rect — needed here so a bar-line tap
    // actually paints a per-note highlight to prove stale afterward (see
    // this hook's own doc comment: the bare shorthand renders no individual
    // note click targets at all, so `applyPersistedNoteHighlights` would
    // have nothing to flag either way, silently passing regardless of the
    // bug this scenario guards against).
    const sectionSource = [
      'metadata { title "Synced Section Score" },',
      '',
      'parts ["Melody" [M]: notes],',
      '',
      'score """',
      '    time=4/4 key=C4 bpm=120 label="A"',
      '    [M] 1 2 3 4',
      '',
      "    [M] 5 6 7 1'",
      '',
      '    label="B"',
      "    [M] 1' 7 6 5",
      '',
      '    [M] 4 3 2 1',
      '    """,',
    ].join('\n')
    await seedSyncedCloudFile(page, 'synced-section-test', sectionSource)
  },
)

Given('local storage is cleared', async ({ page }) => {
  await page.addInitScript(() => {
    localStorage.clear()
  })
})

// Pre-seeds a signed-in session (bypassing the account dialog -- that flow
// has its own coverage in account-auth.feature and
// synced-share-sign-in.feature) so "Sync" starts a share directly via
// `POST /files/:id/share`. The same session signs the cloud backend in, so
// a seeded cloud file loads too. Must be registered (via `addInitScript`)
// after any "local storage is cleared" step in the same scenario, which
// would otherwise wipe it.
Given('the owner is signed in as {string}', async ({ page }, email: string) => {
  await seedSignedInSession(page, email)
})

/** Waits until the scenario's seeded cloud file (if any) is the one on
 * screen -- until the cloud listing loads the active file is a demo, whose
 * modal shows the cloud-only message rather than "Start Sync". Skipped
 * while signed out: the cloud backend (and so the seeded file) only loads
 * once signed in. Exported for `synced-share-sign-in.steps.ts`. */
export async function waitForSeededSyncedFile(page: Page): Promise<void> {
  if (!state.ownerFileName) return
  const signedIn = await page.evaluate(
    (key) => localStorage.getItem(key) !== null,
    ACCOUNT_AUTH_STORAGE_KEY,
  )
  if (!signedIn) return
  await expect(fileSwitcherTrigger(page)).toContainText(
    state.ownerFileName.replace(/\.poentaro$/, ''),
    { timeout: 15_000 },
  )
}

When(
  'the owner loads the app and clicks {string}',
  async ({ page }, label: string) => {
    expect(label).toBe('Sync')
    // Skip navigating if a prior step already loaded the app on this page
    // (e.g. "the app loads the GitHub-backed file list ..." + "I select the
    // ... tab ..." for a GitHub-backed scenario) -- a second `page.goto('/')`
    // is a full reload that would discard whichever file tab was just
    // selected, resetting the GitHub backend's active file back to the demo
    // default. A fresh Playwright `page` fixture starts at `about:blank`;
    // checking that (rather than e.g. the URL's trailing slash) is robust to
    // `urlFileParam.ts`'s `writeFileNameToUrl` adding a `?file=...` query
    // param once a tab has been selected, which would otherwise make this
    // look like a fresh page again.
    if (page.url() === 'about:blank') {
      await page.goto(
        state.ownerFileName
          ? `/?file=${encodeURIComponent(state.ownerFileName)}`
          : '/',
      )
    }
    await waitForSeededSyncedFile(page)
    await openSyncedTab(page)
    // When disconnected, the Synced-link tab shows its sign-in state
    // instead of a "Start Sync" button (see
    // synced-share-sign-in.feature) -- this shared step just opens
    // the tab in that case and leaves the sign-in flow to the caller.
    const startSync = page.getByTestId('share-modal-start-sync')
    if (await startSync.count()) await startSync.click()
  },
)

Then('the synced link is copied', async ({ page }) => {
  await expect(page.getByTestId('share-modal-copy-synced-link')).toHaveText(
    'Link copied',
  )
  state.syncedShareLink = await page.evaluate(async () => {
    return navigator.clipboard.readText()
  })
  if (state.originalSyncedLink === undefined) {
    state.originalSyncedLink = state.syncedShareLink
  }
})

When(
  'a viewer opens the copied sync link in a new page',
  async ({ browser }) => {
    if (!state.syncedShareLink)
      throw new Error('syncedShareLink was not captured yet')
    // An isolated browser context, not `context.newPage()` -- see the
    // `viewerContext` field's own comment in `synced-share-button-state.ts`
    // for why sharing the owner's localStorage would be wrong here.
    state.viewerContext = await browser.newContext()
    state.viewerPage = await state.viewerContext.newPage()
    await state.viewerPage.goto(state.syncedShareLink)

    // No edit was made on the owner's side — the share's initial doc must
    // still arrive from that first fetch.
    await state.viewerPage.waitForSelector('.preview-page', {
      timeout: 15_000,
    })
  },
)

Then("the viewer's preview contains {string}", async ({}, text: string) => {
  if (!state.viewerPage) throw new Error('viewerPage was not opened yet')
  // Retrying, not a one-shot read: right after a reload `.preview-page` can
  // still be the placeholder score until the share fetch lands.
  await expect(state.viewerPage.locator('.preview-page').first()).toContainText(
    text,
  )
})

Then(
  'the copied sync link contains the filename as a human-readable suffix',
  async () => {
    if (!state.syncedShareLink)
      throw new Error('syncedShareLink was not captured yet')
    if (!state.ownerFileName)
      throw new Error('ownerFileName was not seeded yet')
    const payload = new URL(state.syncedShareLink).searchParams.get('s')
    expect(payload).toContain(
      `--${state.ownerFileName.replace(/\.poentaro$/, '').replace(/\s+/g, '-')}`,
    )
  },
)

Then("the viewer's page URL has no ?file= param", async () => {
  if (!state.viewerPage) throw new Error('viewerPage was not opened yet')
  expect(new URL(state.viewerPage.url()).searchParams.has('file')).toBe(false)
})

Then("the viewer's page URL has no ?s= param", async () => {
  if (!state.viewerPage) throw new Error('viewerPage was not opened yet')
  expect(new URL(state.viewerPage.url()).searchParams.has('s')).toBe(false)
})

Then('the copied sync link matches the synced URL format', async () => {
  if (!state.syncedShareLink)
    throw new Error('syncedShareLink was not captured yet')
  const url = new URL(state.syncedShareLink)
  expect(url.hash).toBe('')
  expect(url.searchParams.get('s')).toMatch(/^[0-9A-Za-z_-]{11}(--\S+)?$/)
})

Then('the share modal shows the stop-sync button', async ({ page }) => {
  await expect(page.getByTestId('share-modal-stop-sync')).toBeVisible()
})

Then('the share modal shows the start-sync button', async ({ page }) => {
  await expect(page.getByTestId('share-modal-start-sync')).toBeVisible()
})

When('the owner clicks the copy-synced-link button', async ({ page }) => {
  await page.getByTestId('share-modal-copy-synced-link').click()
})

Then('the copied link is unchanged from before', async ({ page }) => {
  if (!state.syncedShareLink)
    throw new Error('syncedShareLink was not captured yet')
  const copiedAgain = await page.evaluate(() => navigator.clipboard.readText())
  expect(copiedAgain).toEqual(state.syncedShareLink)
})

When('the owner clicks the stop-sync button', async ({ page }) => {
  await page.getByTestId('share-modal-stop-sync').click()
  await page.getByTestId('share-modal-confirm-stop-sync').click()
})

Then('the viewer sees the preview page', async () => {
  if (!state.viewerPage) throw new Error('viewerPage was not opened yet')
  await state.viewerPage.waitForSelector('.preview-page', { timeout: 15_000 })
})

Then('the viewer sees {string}', async ({}, text: string) => {
  if (!state.viewerPage) throw new Error('viewerPage was not opened yet')
  // A viewer connected *before* the owner stops should lose the score the
  // moment the owner does.
  await expect(state.viewerPage.getByText(text)).toBeVisible()
})

Then(
  "the viewer's preview no longer contains {string}",
  async ({}, text: string) => {
    if (!state.viewerPage) throw new Error('viewerPage was not opened yet')
    // `filter` + count, not `not.toContainText`: an ended share now replaces
    // the score with a notice, so no preview page exists at all.
    await expect(
      state.viewerPage.locator('.preview-page').filter({ hasText: text }),
    ).toHaveCount(0)
  },
)

When('the viewer reloads the page', async () => {
  if (!state.viewerPage) throw new Error('viewerPage was not opened yet')
  await state.viewerPage.reload()
})

// Installed on the owner's page only — the autosave this guards is entirely
// owner-side (viewers just read the file's saved content), so there is
// nothing for the viewer's clock to affect.
Given('the clock is under test control', async ({ page }) => {
  await page.clock.install()
})

// Edits the owner's editor content directly via the Monaco model rather
// than typing character-by-character (as `typeAtEditorEnd` does for
// appending text) — a title change needs to replace an existing line, not
// just append after it. `setValue` still fires the model's change event, so
// this exercises the same `onChange` -> autosave path a real edit would.
When(
  "the owner edits the synced score's title to {string}",
  async ({ page }, title: string) => {
    const edited = SYNCED_SOURCE.replace(
      'title "Synced Score"',
      `title "${title}"`,
    )
    await page.evaluate((value) => {
      const editor = (
        window as unknown as {
          monaco?: typeof import('monaco-editor')
        }
      ).monaco?.editor.getEditors()[0]
      editor?.getModel()?.setValue(value)
    }, edited)
  },
)

When("the owner's autosave debounce interval elapses", async ({ page }) => {
  // Waits for the save itself to land, not just the timer -- the viewer
  // reads whatever the worker has stored, so reloading it before the
  // `POST /files/:id/content` response would race the write.
  await Promise.all([
    page.waitForResponse(
      (response) =>
        /\/files\/[^/]+\/content$/.test(response.url()) && response.ok(),
    ),
    page.clock.fastForward(AUTOSAVE_DEBOUNCE_MS),
  ])
})
