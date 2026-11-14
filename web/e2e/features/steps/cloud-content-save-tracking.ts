import type { Page } from '@playwright/test'
import type { FilesFileSummary } from '../../../src/generated/protocol/app'
import { poentaroWasm } from '../../../src/poentaroWasm'
import { appCallTo } from '../../appCalls'

const WRITE = 'files/write _ at revision _ content _'
const LIST = 'files/list'

// Shared observation helper for the cloud backend's content-save requests,
// used by several `.steps.ts` files (`files-cloud-backend`,
// `cmd-s-force-save-cloud`, `tab-switch-force-save-cloud`,
// `beforeunload-warning-cloud`, `storage-error-banner-cloud`,
// `cloud-backend-conflict-resolution`). Plain functions, no step
// registrations, matching `cloud-account-helpers.ts`'s split.

export interface ContentSaveRecord {
  id: string
  content: string
}

export interface ContentSaveTracker {
  records: ContentSaveRecord[]
  /** File id -> current name, learned from every `files/list`
   * answer the page receives -- lets assertions read naturally by
   * filename instead of by the file's opaque D1 id. */
  nameForId: Map<string, string>
}

/**
 * Observes (without intercepting or modifying) the browser's own network
 * traffic to the real local worker, via `page.on('request'/'response')`
 * rather than `page.route()` -- mirrors the deleted GitHub backend's
 * `onPut` callback + accumulated `putBodies` array, but for the cloud
 * backend's `files/write` calls. Nothing here
 * needs to change what actually happens on the wire (every `files/*`
 * call is worker+D1 mediated -- see this feature set's own framing note
 * in the plan), so a passive observer is simpler and safer than a route
 * handler that has to remember to `route.continue()`.
 *
 * Call once per scenario, before navigating, so the very first
 * `files/list` (which populates `nameForId`) is captured too.
 */
// Module-level, not a fixture -- matches every other `.steps.ts` file's
// established pattern for cross-step state within one scenario (e.g.
// `putBodies` in the deleted `autosave-github.steps.ts`). Holding a single
// "current" tracker here (rather than each `.steps.ts` file declaring its
// own private variable) lets a `Then` step in one feature's steps file read
// a tracker installed by a `When` step in another, without either file
// needing to import the other's internals -- several `-cloud` feature files
// share the same "assert on the last content-save request" shape.
let activeTracker: ContentSaveTracker | null = null

export function installContentSaveTracking(page: Page): ContentSaveTracker {
  const tracker: ContentSaveTracker = { records: [], nameForId: new Map() }
  activeTracker = tracker

  page.on('request', async (request) => {
    const call = await appCallTo(request, WRITE)
    if (!call) return
    const [id, , content] = call.arguments
    if (typeof id !== 'string' || typeof content !== 'string') return
    tracker.records.push({ id, content })
  })

  page.on('response', async (response) => {
    if (!(await appCallTo(response.request(), LIST))) return
    try {
      const outcome = JSON.parse(
        poentaroWasm().protocolDecodeResponse(
          'app',
          LIST,
          await response.text(),
        ),
      ) as { kind: string; value?: FilesFileSummary[] }
      for (const file of outcome.value ?? []) {
        tracker.nameForId.set(file.id, file.path)
      }
    } catch {
      // A failed or unreadable answer -- nothing to learn from it.
    }
  })

  return tracker
}

/** The tracker installed by the current scenario's own call to
 * `installContentSaveTracking` -- throws if none has been installed yet,
 * so a step ordering mistake fails loudly instead of silently reading
 * `undefined`. */
export function currentContentSaveTracker(): ContentSaveTracker {
  if (!activeTracker) {
    throw new Error(
      'currentContentSaveTracker: installContentSaveTracking was not called yet this scenario',
    )
  }
  return activeTracker
}

/** The most recent content-save record for the file currently known by
 * `name`, if any. `undefined` both when no such file has been seen yet
 * (via a `files/list` answer) and when it has but no save has landed
 * for it. */
export function lastContentSaveFor(
  tracker: ContentSaveTracker,
  name: string,
): string | undefined {
  const id = [...tracker.nameForId.entries()].find(
    ([, fileName]) => fileName === name,
  )?.[0]
  if (!id) return undefined
  return [...tracker.records].reverse().find((record) => record.id === id)
    ?.content
}
