import type { BrowserContext, Page } from '@playwright/test'
import { seedCloudFile } from '../../cloudFileHelpers'
import { DEFAULT_E2E_EMAIL } from '../../e2eAccounts.ts'

export const SYNCED_FILE_BASE_NAME = 'synced-test'
export const SYNCED_SOURCE = [
  'metadata { title "Synced Score" },',
  '',
  'parts ["Melody": notes],',
  '',
  'score """',
  '    (time=4/4 key=C4 bpm=120)',
  '    1 2 3 4',
  '    """,',
].join('\n')

/** Seeds `baseName` as a real cloud file owned by the default e2e
 * account, under a per-scenario unique name (the scenarios run
 * `fullyParallel` against one shared account, and a live link belongs to
 * exactly one `files` row -- two scenarios seeding the same name would share,
 * and stop, each other's share), and makes the owner's page load onto the
 * cloud backend. A live link can only point at a cloud file, so every synced
 * scenario starts here. "the owner loads the app and clicks Sync" then opens
 * the app on this file via `?file=`. */
export async function seedSyncedCloudFile(
  page: Page,
  baseName: string,
  source: string,
): Promise<void> {
  const name = `${baseName}-${crypto.randomUUID().slice(0, 8)}.poentaro`
  const file = await seedCloudFile(DEFAULT_E2E_EMAIL, name, source)
  syncedShareButtonState.ownerFileName = name
  syncedShareButtonState.ownerFileId = file.id
  await page.addInitScript(() => {
    localStorage.setItem(
      'poentaro:storage-backend:v1',
      JSON.stringify({ backend: 'cloud' }),
    )
  })
}

// Cross-step state shared between synced-share-button.steps.ts and
// synced-share-button-viewer-range-select.steps.ts (split out of one file to stay under
// the repo's max-file-lines limit). Each scenario's first Given resets this
// so state never leaks across scenarios. `syncedShareLink` always holds the most
// recently copied link; `originalSyncedLink` is pinned to the very first link
// copied in the scenario, so later "revived link" assertions can compare a
// fresh clipboard read against it without it having been clobbered by an
// intervening copy of what should be the same link.
export interface SyncedShareButtonState {
  syncedShareLink: string | undefined
  originalSyncedLink: string | undefined
  viewerPage: Page | undefined
  lateViewerPage: Page | undefined
  /** The owner's seeded cloud file (see `seedSyncedCloudFile`), unique per
   * scenario. */
  ownerFileName: string | undefined
  ownerFileId: string | undefined
  // Isolated browser contexts (not `context.newPage()`) for the viewer/late
  // viewer -- a real viewer is a different, signed-out browser that doesn't
  // share the owner's localStorage (and so never loads the owner's cloud
  // files or edits them itself).
  viewerContext: BrowserContext | undefined
  lateViewerContext: BrowserContext | undefined
}

export const syncedShareButtonState: SyncedShareButtonState = {
  syncedShareLink: undefined,
  originalSyncedLink: undefined,
  ownerFileName: undefined,
  ownerFileId: undefined,
  viewerPage: undefined,
  lateViewerPage: undefined,
  viewerContext: undefined,
  lateViewerContext: undefined,
}
