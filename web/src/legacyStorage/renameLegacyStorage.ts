import { openDB } from 'idb'
import { FILE_STORE_KEY, renameLegacyExtensions } from '../fileStore'
import { parseStoredFileStore } from '../fileStorePersistence'
import { ASSET_CACHE_NAME } from '../hooks/useAssetLoader'
import {
  OUTBOX_OBJECT_STORE_NAME,
  openIndexedDbOutboxStore,
  type StoredRecord,
} from '../storage/outbox/outboxStore'
import { ensureWasmInit } from '../wasmInit'
import {
  currentLocalStorageKey,
  currentOutboxDatabaseName,
  LEGACY_LOCAL_STORAGE_PREFIX,
  LEGACY_OUTBOX_DATABASE_PREFIX,
} from './legacyStorageNames'

/** Where a legacy file store goes when a current one differs from it, so
 * it is never lost (only reachable by hand). */
const LEGACY_FILE_STORE_BACKUP_KEY = `${FILE_STORE_KEY}:legacy-backup`

/** What the downloaded-assets cache was named before the rename. */
const LEGACY_ASSET_CACHE_NAME = 'jianpu-assets-v1'

/**
 * Moves what this browser saved before the app was renamed to Poentaro to
 * the current names (see "Rename to Poentaro" in `ARCHITECTURE.md`): every
 * cloud outbox database, the downloaded-assets cache and every
 * `localStorage` key. Something already saved under a current name wins.
 * The `localStorage` keys move last, so a rename cut short is finished on
 * the next visit.
 */
export async function renameLegacyStorage(): Promise<void> {
  await Promise.all([renameLegacyOutboxes(), renameLegacyAssetCache()])
  renameLegacyLocalStorage()
}

/** Renames the files of the saved file store that still end in the legacy
 * extension. Rust decides which (see `file_tree::rename_legacy_extensions`). */
export async function renameLegacyFiles(): Promise<void> {
  const raw = localStorage.getItem(FILE_STORE_KEY)
  const state = raw === null ? null : parseStoredFileStore(raw)
  if (state === null) return
  await ensureWasmInit()
  const renamed = renameLegacyExtensions(state)
  if (renamed !== null) {
    localStorage.setItem(FILE_STORE_KEY, JSON.stringify(renamed))
  }
}

function renameLegacyLocalStorage(): void {
  const legacyKeys = Array.from({ length: localStorage.length }, (_, index) =>
    localStorage.key(index),
  ).filter(
    (key): key is string =>
      key?.startsWith(LEGACY_LOCAL_STORAGE_PREFIX) === true,
  )
  for (const key of legacyKeys) {
    const value = localStorage.getItem(key)
    const current = currentLocalStorageKey(key)
    const existing = localStorage.getItem(current)
    if (value !== null && existing === null) {
      localStorage.setItem(current, value)
    } else if (
      value !== null &&
      value !== existing &&
      current === FILE_STORE_KEY
    ) {
      // A tab still running the old build kept saving files under the
      // legacy key. Keep its copy rather than drop it.
      localStorage.setItem(LEGACY_FILE_STORE_BACKUP_KEY, value)
    }
    localStorage.removeItem(key)
  }
}

async function renameLegacyOutboxes(): Promise<void> {
  const databases = await indexedDB.databases().catch(() => [])
  const legacyNames = databases
    .map((database) => database.name)
    .filter(
      (name): name is string =>
        name?.startsWith(LEGACY_OUTBOX_DATABASE_PREFIX) === true,
    )
  await Promise.all(legacyNames.map(renameLegacyOutbox))
}

/** Copies one account's queued edits to the current database, then deletes
 * the legacy one. When the current one already holds edits, both are kept
 * and nothing is deleted, so no queued edit is ever lost. */
async function renameLegacyOutbox(legacyName: string): Promise<void> {
  const records = await readAndCloseDatabase(legacyName)
  const current = openIndexedDbOutboxStore(
    currentOutboxDatabaseName(legacyName),
  )
  if (records.length > 0) {
    if ((await current.loadAll()).length > 0) return
    await current.apply({ puts: records, deletes: [] })
  }
  await deleteDatabase(legacyName)
}

/** Every record of the outbox database `name`, closing it afterwards so it
 * can be deleted. */
async function readAndCloseDatabase(name: string): Promise<StoredRecord[]> {
  const database = await openDB(name)
  try {
    if (!database.objectStoreNames.contains(OUTBOX_OBJECT_STORE_NAME)) return []
    const transaction = database.transaction(OUTBOX_OBJECT_STORE_NAME)
    const [keys, values] = await Promise.all([
      transaction.store.getAllKeys(),
      transaction.store.getAll(),
    ])
    return keys.map((key, index) => ({
      key: String(key),
      value: String(values[index] ?? ''),
    }))
  } finally {
    database.close()
  }
}

function deleteDatabase(name: string): Promise<void> {
  return new Promise((resolve) => {
    const request = indexedDB.deleteDatabase(name)
    request.onsuccess = () => resolve()
    request.onerror = () => resolve()
    request.onblocked = () => resolve()
  })
}

async function renameLegacyAssetCache(): Promise<void> {
  if (typeof caches === 'undefined') return
  if (!(await caches.has(LEGACY_ASSET_CACHE_NAME))) return
  const legacy = await caches.open(LEGACY_ASSET_CACHE_NAME)
  const current = await caches.open(ASSET_CACHE_NAME)
  const requests = await legacy.keys()
  await Promise.all(
    requests.map(async (request) => {
      const response = await legacy.match(request)
      if (response !== undefined && !(await current.match(request))) {
        await current.put(request, response)
      }
    }),
  )
  await caches.delete(LEGACY_ASSET_CACHE_NAME)
}
