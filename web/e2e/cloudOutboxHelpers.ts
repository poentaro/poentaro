import { expect, type Page, type Route } from '@playwright/test'
import { refuseCall, routeAppCall } from './appCalls'

// Generic helpers for e2e scenarios about the cloud outbox (content saves,
// and later the structural / visibility / conflict phases). Plain functions,
// no step registrations, so any `.steps.ts` file can import them.

/** What a matching content-save request should suffer. */
type ContentSaveFault =
  /** The request never reaches the server (a network failure). */
  | { kind: 'abort' }
  /** The server answers with `status` without applying the save. */
  | { kind: 'status'; status: number }
  /** The server refuses the save with the method error `error`, e.g.
   * `trashed`. */
  | { kind: 'refuse'; error: string }
  /** The server applies the save, but the browser never sees the response. */
  | { kind: 'drop-response' }

export interface ContentSaveFaultRule {
  fault: ContentSaveFault
  /** Only requests for this file name (resolved via `nameForId`); every
   * file when omitted. */
  fileName?: string
  /** How many matching requests to fault before letting later ones through;
   * every matching request when omitted. */
  times?: number
}

interface RuleState {
  rule: ContentSaveFaultRule
  used: number
}

const rulesByPage = new WeakMap<Page, RuleState[]>()

/** Maps a worker file id to its current name (see `ContentSaveTracker`). */
export type FileNameResolver = (fileId: string) => string | undefined

const WRITE = 'files/write _ at revision _ content _'

async function handleContentSave(
  route: Route,
  fileId: string,
  rules: RuleState[],
  nameForId: FileNameResolver,
): Promise<void> {
  const fileName = nameForId(fileId)
  const matching = rules.find(
    (state) =>
      (state.rule.fileName === undefined || state.rule.fileName === fileName) &&
      state.used < (state.rule.times ?? Number.POSITIVE_INFINITY),
  )
  if (!matching) {
    await route.continue()
    return
  }
  matching.used += 1
  const { fault } = matching.rule
  switch (fault.kind) {
    case 'abort':
      await route.abort('failed')
      return
    case 'status':
      await route.fulfill({
        status: fault.status,
        body: `faked ${fault.status}`,
      })
      return
    case 'refuse':
      await refuseCall(route, fault.error)
      return
    case 'drop-response':
      await route.fetch()
      await route.abort('failed')
      return
  }
}

/**
 * Faults content-save requests (`files/write` calls) according to
 * `rule`. Rules accumulate and the first still-active matching one applies.
 * Survives page reloads, so a fault can span a refresh until
 * `allowContentSaves` is called. `nameForId` is only needed for per-file rules.
 */
export async function interceptContentSaves(
  page: Page,
  rule: ContentSaveFaultRule,
  nameForId: FileNameResolver,
): Promise<void> {
  const existing = rulesByPage.get(page)
  if (existing) {
    existing.push({ rule, used: 0 })
    return
  }
  const rules: RuleState[] = [{ rule, used: 0 }]
  rulesByPage.set(page, rules)
  await routeAppCall(page, WRITE, (route, call) =>
    handleContentSave(route, String(call.arguments[0]), rules, nameForId),
  )
}

/** Stops faulting content saves: every later request reaches the server. */
export function allowContentSaves(page: Page): void {
  const rules = rulesByPage.get(page)
  if (rules) rules.length = 0
}

const AUTOSAVE_DEBOUNCE_MS = 20_000
// The outbox's backoff is capped at 60s (`crates/cloud-outbox/src/backoff.rs`).
const OUTBOX_RETRY_CAP_MS = 60_000

/**
 * With a fake clock installed (`page.clock.install()`): fires the autosave
 * debounce, waits for the failed save to park its lane as "waiting", then
 * jumps past the longest possible retry delay. (`fastForward` fires each
 * timer at most once, so the retry timer, armed only after the first
 * attempt fails, needs its own jump.)
 */
export async function fastForwardPastOutboxRetryDelay(
  page: Page,
): Promise<void> {
  await page.clock.fastForward(AUTOSAVE_DEBOUNCE_MS)
  await expect(saveStatusBadge(page)).toContainText('waiting', {
    timeout: 10_000,
  })
  await page.clock.fastForward(OUTBOX_RETRY_CAP_MS)
}

/** The save status badge (`SaveStatusBadge`). */
export function saveStatusBadge(page: Page) {
  return page.getByTestId('save-status-badge')
}

/** Asserts the badge text exactly, e.g. "Saved" or "1 change waiting". */
export async function expectSaveBadge(page: Page, text: string) {
  await expect(saveStatusBadge(page)).toHaveText(text, { timeout: 15_000 })
}
