import {
  LEGACY_LOCAL_STORAGE_PREFIX,
  LEGACY_OUTBOX_DATABASE_PREFIX,
} from '../legacyStorage/legacyStorageNames'
import {
  ORIGIN_MIGRATED_KEY,
  type OriginMigrationPayload,
  type SavedDatabase,
} from '../originMigration/originMigrationPayload'
import { openIndexedDbOutboxStore } from '../storage/outbox/outboxStore'

/** Everything this origin saved: the app's `localStorage` entries and every
 * account's cloud outbox, all under the names the app had before the rename
 * to Poentaro. Storage that can't be read counts as empty. */
export async function collectSavedData(): Promise<OriginMigrationPayload> {
  return {
    version: 1,
    localStorage: savedLocalStorage(),
    indexedDb: await savedOutboxes(),
  }
}

function savedLocalStorage(): Record<string, string> {
  try {
    return Object.fromEntries(
      Array.from({ length: localStorage.length }, (_, index) =>
        localStorage.key(index),
      )
        .filter(
          (key): key is string =>
            key?.startsWith(LEGACY_LOCAL_STORAGE_PREFIX) === true &&
            key !== ORIGIN_MIGRATED_KEY,
        )
        .flatMap((key) => {
          const value = localStorage.getItem(key)
          return value === null ? [] : [[key, value] as const]
        }),
    )
  } catch {
    return {}
  }
}

async function savedOutboxes(): Promise<SavedDatabase[]> {
  const databases = await indexedDB.databases().catch(() => [])
  const names = databases
    .map((database) => database.name)
    .filter(
      (name): name is string =>
        name?.startsWith(LEGACY_OUTBOX_DATABASE_PREFIX) === true,
    )
  return Promise.all(
    names.map(async (name) => ({
      name,
      records: await openIndexedDbOutboxStore(name)
        .loadAll()
        .catch(() => []),
    })),
  )
}
