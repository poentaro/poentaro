import { FILE_STORE_KEY, mergeMigratedFileStore } from '../fileStore'
import { parseStoredFileStore } from '../fileStorePersistence'
import {
  currentLocalStorageKey,
  currentOutboxDatabaseName,
} from '../legacyStorage/legacyStorageNames'
import {
  OUTBOX_DATABASE_PREFIX,
  openIndexedDbOutboxStore,
} from '../storage/outbox/outboxStore'
import { callWorker, createWorkerClient } from '../syncedShare/workerClient'
import { ensureWasmInit } from '../wasmInit'
import {
  MIGRATE_QUERY_PARAM,
  type OriginMigrationPayload,
  type SavedDatabase,
} from './originMigrationPayload'

/** Where a carried file store goes when Rust can't merge it, so it is never
 * lost (only reachable by hand). */
const UNMERGED_FILE_STORE_KEY = 'poentaro:files:v1:unmerged-origin-migration'

/**
 * Claims the data the old origin's bridge page handed over (the
 * `?migrate=<id>` it redirected with) and saves it here, before the app
 * first reads storage. See "Origin migration" in `ARCHITECTURE.md`. The
 * param is removed from the address bar first, so a reload never claims
 * twice. The carried data has the names the app had before the rename to
 * Poentaro, so it is saved under the current ones. Anything already saved
 * here wins, except the file store, which Rust merges. A failed claim is
 * only logged: the app still starts.
 */
export async function importOriginMigration(): Promise<void> {
  const url = new URL(window.location.href)
  const id = url.searchParams.get(MIGRATE_QUERY_PARAM)
  if (id === null) return
  url.searchParams.delete(MIGRATE_QUERY_PARAM)
  window.history.replaceState(window.history.state, '', url)

  try {
    const { payload } = await callWorker(
      createWorkerClient(import.meta.env.VITE_SYNCED_SHARE_HOST ?? '').POST(
        '/origin-migrations/claim',
        { body: { id } },
      ),
    )
    const carried = JSON.parse(payload) as OriginMigrationPayload
    await saveLocalStorage(carried.localStorage)
    await Promise.all(carried.indexedDb.map(saveOutbox))
  } catch (error) {
    console.warn('origin migration: could not import saved data', error)
  }
}

async function saveLocalStorage(entries: Record<string, string>) {
  for (const [carriedKey, value] of Object.entries(entries)) {
    const key = currentLocalStorageKey(carriedKey)
    const existing = localStorage.getItem(key)
    if (existing === null) {
      localStorage.setItem(key, value)
    } else if (key === FILE_STORE_KEY && existing !== value) {
      await saveMergedFileStore(existing, value)
    }
  }
}

async function saveMergedFileStore(existing: string, carried: string) {
  const current = parseStoredFileStore(existing)
  const incoming = parseStoredFileStore(carried)
  if (current === null || incoming === null) {
    localStorage.setItem(UNMERGED_FILE_STORE_KEY, carried)
    return
  }
  await ensureWasmInit()
  const merged = mergeMigratedFileStore(current, incoming)
  if (merged === null) {
    localStorage.setItem(UNMERGED_FILE_STORE_KEY, carried)
    return
  }
  localStorage.setItem(FILE_STORE_KEY, JSON.stringify(merged))
}

/** Restores one account's cloud outbox, unless this origin already queued
 * edits for that account (then its own queue wins). */
async function saveOutbox(database: SavedDatabase) {
  const name = currentOutboxDatabaseName(database.name)
  if (
    !name.startsWith(OUTBOX_DATABASE_PREFIX) ||
    database.records.length === 0
  ) {
    return
  }
  const store = openIndexedDbOutboxStore(name)
  if ((await store.loadAll()).length > 0) return
  await store.apply({ puts: database.records, deletes: [] })
}
