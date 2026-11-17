import type { StoredRecord } from '../storage/outbox/outboxStore'

/**
 * The browser data an origin migration carries from the app's old origin
 * (`jianpu-generator.pages.dev`) to its current one (`poentaro.pages.dev`),
 * see "Origin migration" in `ARCHITECTURE.md`. The worker stores it as
 * opaque JSON text; only the bridge page (`src/bridge/`) writes it and only
 * the app (`importOriginMigration.ts`) reads it.
 */
export interface OriginMigrationPayload {
  version: 1
  /** Every `localStorage` entry whose key starts with
   * `LEGACY_LOCAL_STORAGE_PREFIX`: the old origin only ever ran the app
   * under the names it had before the rename to Poentaro. */
  localStorage: Record<string, string>
  /** Every cloud outbox database (`LEGACY_OUTBOX_DATABASE_PREFIX`), by
   * name. */
  indexedDb: SavedDatabase[]
}

export interface SavedDatabase {
  name: string
  records: StoredRecord[]
}

/** Query param the bridge adds to its redirect: the handoff id to claim. */
export const MIGRATE_QUERY_PARAM = 'migrate'

/** Set on the OLD origin once its data was handed over, so later visits
 * redirect straight away. Never carried in a payload. Keeps its legacy name:
 * old origins already hold it under this one. */
export const ORIGIN_MIGRATED_KEY = 'jianpu:origin-migrated:v1'

/** Where the bridge sends visitors, unless `VITE_ORIGIN_MIGRATION_TARGET`
 * overrides it (e2e points it at the local app). */
export const DEFAULT_MIGRATION_TARGET_ORIGIN = 'https://poentaro.pages.dev'

export function isEmptyPayload(payload: OriginMigrationPayload): boolean {
  return (
    Object.keys(payload.localStorage).length === 0 &&
    payload.indexedDb.every((database) => database.records.length === 0)
  )
}
