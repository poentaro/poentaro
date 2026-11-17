import type { BrowserContext, Page } from '@playwright/test'

// Cross-step state for `synced-share-idempotent-create.steps.ts`. The
// "original" link itself is read from `synced-share-button-state.ts`'s
// shared `syncedShareButtonState` (set by the existing "the synced link is
// copied" step, reused verbatim here) -- this file only holds what's new:
// the second ("separate browser context") owner session, which cloud-backed
// tab is currently active (so a later "a separate browser context loads the
// same cloud-backed file ..." step knows which file to open without
// threading it through every step's params), each seeded file's real D1 id
// (see `seededFileIds` below), and its real, per-scenario unique name (see
// `actualNames`).
export interface IdempotentCreateState {
  activeTabName: string | undefined
  secondContext: BrowserContext | undefined
  secondPage: Page | undefined
  secondSyncedShareLink: string | undefined
  /** Maps each `Given`-seeded cloud file's display name (extension
   * stripped) to its real D1 `files.id`, as returned by `seedCloudFile`.
   * Captured so the "different account" scenario can hand a *different*
   * signed-in account a synthetic `/files/list` entry carrying that exact
   * id -- `files.id` is a globally unique primary key (`0003_files.sql`),
   * so two different accounts can never really own a row with the same id,
   * and this is the only way to send `POST /files/:id/share` for someone
   * else's file through the real worker. See
   * `loadSecondContextOnCloudFile`'s own doc comment. */
  seededFileIds: Record<string, string>
  /** Maps each display name a scenario uses (extension stripped, e.g.
   * "idempotent") to the real file name it stands for (e.g.
   * "idempotent-1a2b3c4d") -- scenarios run `fullyParallel` against one
   * shared account, so a fixed name would let one scenario share, or
   * rename, another's file. */
  actualNames: Record<string, string>
}

export const idempotentCreateState: IdempotentCreateState = {
  activeTabName: undefined,
  secondContext: undefined,
  secondPage: undefined,
  secondSyncedShareLink: undefined,
  seededFileIds: {},
  actualNames: {},
}

export function resetIdempotentCreateState(): void {
  idempotentCreateState.activeTabName = undefined
  idempotentCreateState.secondContext = undefined
  idempotentCreateState.secondPage = undefined
  idempotentCreateState.secondSyncedShareLink = undefined
  idempotentCreateState.seededFileIds = {}
  idempotentCreateState.actualNames = {}
}

/** The real, per-scenario unique file name (extension stripped) behind a
 * display name -- see `actualNames`. */
export function actualNameFor(displayName: string): string {
  const actual = idempotentCreateState.actualNames[displayName]
  if (!actual) {
    throw new Error(
      `no cloud file name tracked for ${JSON.stringify(displayName)}`,
    )
  }
  return actual
}

/** Records a fresh, unique real name for `displayName` and returns it. */
export function assignActualName(displayName: string): string {
  const actual = `${displayName}-${crypto.randomUUID().slice(0, 8)}`
  idempotentCreateState.actualNames[displayName] = actual
  return actual
}

/** Deterministic per-name content for a cloud file seeded by "a cloud file
 * named {string} is seeded for idempotent sharing" -- distinct titles just
 * make a failing assertion's diff readable, the content itself isn't
 * asserted on by any scenario in this feature. */
export function idempotentSharingSource(name: string): string {
  const title = name.replace(/\.poentaro$/, '')
  return [
    `metadata { title "${title}" },`,
    '',
    'parts ["Melody": notes],',
    '',
    'score """',
    '    (time=4/4 key=C4 bpm=120)',
    '    1 2 3 4',
    '    """,',
  ].join('\n')
}
