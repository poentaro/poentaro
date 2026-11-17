import {
  DEFAULT_MIGRATION_TARGET_ORIGIN,
  isEmptyPayload,
  ORIGIN_MIGRATED_KEY,
} from '../originMigration/originMigrationPayload'
import {
  callWorker,
  createWorkerClient,
  WorkerRequestError,
} from '../syncedShare/workerClient'
import { bridgeRedirectUrl } from './bridgeUrl'
import { collectSavedData } from './collectSavedData'

// The bridge page the app's OLD origins serve for every path once the app
// moved to `poentaro.pages.dev` (see "Origin migration" in
// `ARCHITECTURE.md`). It hands this browser's saved data over through the
// worker, then redirects to the same link on the new origin.

const targetOrigin =
  import.meta.env.VITE_ORIGIN_MIGRATION_TARGET ??
  DEFAULT_MIGRATION_TARGET_ORIGIN

function redirect(migrationId: string | null): void {
  window.location.replace(
    bridgeRedirectUrl(
      new URL(window.location.href),
      import.meta.env.BASE_URL,
      targetOrigin,
      migrationId,
    ),
  )
}

function markMigrated(): void {
  try {
    localStorage.setItem(ORIGIN_MIGRATED_KEY, new Date().toISOString())
  } catch {
    // Storage unavailable: the next visit just looks again.
  }
}

function alreadyMigrated(): boolean {
  try {
    return localStorage.getItem(ORIGIN_MIGRATED_KEY) !== null
  } catch {
    return false
  }
}

function failureMessage(error: unknown): string {
  if (
    error instanceof WorkerRequestError &&
    error.apiError?.code === 'origin_migration_too_large'
  ) {
    return 'Your saved files are too large to move automatically.'
  }
  return "We couldn't move your saved files. Check your connection and try again."
}

function showFailure(error: unknown): void {
  const status = document.getElementById('status')
  const failure = document.getElementById('failure')
  const message = document.getElementById('failure-message')
  const continueLink = document.getElementById('continue')
  if (status) status.hidden = true
  if (failure) failure.hidden = false
  if (message) message.textContent = failureMessage(error)
  continueLink?.setAttribute(
    'href',
    bridgeRedirectUrl(
      new URL(window.location.href),
      import.meta.env.BASE_URL,
      targetOrigin,
      null,
    ),
  )
  document
    .getElementById('retry')
    ?.addEventListener('click', () => window.location.reload())
}

async function run(): Promise<void> {
  if (alreadyMigrated()) {
    redirect(null)
    return
  }
  const payload = await collectSavedData()
  if (isEmptyPayload(payload)) {
    markMigrated()
    redirect(null)
    return
  }
  try {
    const { id } = await callWorker(
      createWorkerClient(import.meta.env.VITE_SYNCED_SHARE_HOST ?? '').POST(
        '/origin-migrations',
        { body: { payload: JSON.stringify(payload) } },
      ),
    )
    markMigrated()
    redirect(id)
  } catch (error) {
    showFailure(error)
  }
}

void run()
