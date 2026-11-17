import { OUTBOX_DATABASE_PREFIX } from '../storage/outbox/outboxStore'

/** Every `localStorage` key the app writes starts with this. */
export const LOCAL_STORAGE_PREFIX = 'poentaro:'

/** What `localStorage` keys started with before the app was renamed to
 * Poentaro. The old origin's bridge page still collects these. */
export const LEGACY_LOCAL_STORAGE_PREFIX = 'jianpu:'

/** What cloud outbox databases were named before the rename (then the
 * account). */
export const LEGACY_OUTBOX_DATABASE_PREFIX = 'jianpu-outbox:'

/** The `localStorage` key `key` is saved under now: a legacy key gets the
 * current prefix, any other key is returned as is. */
export function currentLocalStorageKey(key: string): string {
  return key.startsWith(LEGACY_LOCAL_STORAGE_PREFIX)
    ? `${LOCAL_STORAGE_PREFIX}${key.slice(LEGACY_LOCAL_STORAGE_PREFIX.length)}`
    : key
}

/** The outbox database name `name` is saved under now. */
export function currentOutboxDatabaseName(name: string): string {
  return name.startsWith(LEGACY_OUTBOX_DATABASE_PREFIX)
    ? `${OUTBOX_DATABASE_PREFIX}${name.slice(LEGACY_OUTBOX_DATABASE_PREFIX.length)}`
    : name
}

/** Whether this browser still holds data saved under the legacy names.
 * Cheap and synchronous, so startup only waits for the rename when it has
 * something to do. A legacy outbox always comes with legacy keys (the
 * signed-in session), so checking `localStorage` is enough. */
export function hasLegacyLocalStorage(): boolean {
  try {
    return Array.from({ length: localStorage.length }, (_, index) =>
      localStorage.key(index),
    ).some((key) => key?.startsWith(LEGACY_LOCAL_STORAGE_PREFIX) === true)
  } catch {
    return false
  }
}
