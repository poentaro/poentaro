import type { FileStoreState } from '../fileStore'
import type { StorageBackend } from '../storage/types'
import type { CloudLoadFailure } from './useCloudStoreLoader'
import type {
  DisplaySaveStatus,
  StorageBackendPreference,
  StorageBackendTarget,
} from './useStorageBackend'

export interface UseStorageBackendResult {
  store: FileStoreState
  setStore: (
    value: FileStoreState | ((prev: FileStoreState) => FileStoreState),
  ) => void
  backend: StorageBackend
  /** True from the moment the `cloud` backend becomes active until its
   * `load()` resolves and populates `cloudStore` — the window during which
   * `store` is the `EMPTY_STORE` placeholder rather than the real listing.
   * Lets `StorageSettingsModal` show a loading spinner instead of briefly
   * flashing an empty file list. */
  isLoadingCloud: boolean
  /** Set while a failed cloud load waits to be retried. */
  cloudLoadFailure: CloudLoadFailure | null
  saveStatus: DisplaySaveStatus
  /** Badge text derived from the cloud outbox queue, or `null` for the local
   * backend (whose text comes from `saveStatus` alone). */
  saveLabel: string | null
  /** `Date.now()`-comparable timestamp at which the pending debounced
   * autosave will fire, or `null` when no save is pending. Lets the UI show
   * a countdown alongside the `'unsaved'` status. Cleared the moment the
   * save actually starts (debounce firing, `forceSave`, or
   * `flushPendingSave`), not when it completes. */
  autosaveDeadline: number | null
  /** Currently persisted backend choice, exposed so `StorageSettingsModal`
   * can reflect the active selection without re-deriving it from
   * `backend.kind` alone. */
  preference: StorageBackendPreference
  /**
   * Switches the active backend. If leaving the cloud backend with a
   * pending debounced save, forces it to flush (and awaits it) before
   * switching. Always lands on the demo file — there is no per-backend
   * "last active file" memory.
   */
  switchBackend: (target: StorageBackendTarget) => Promise<void>
  /**
   * Immediately persists the active file, bypassing the autosave debounce
   * interval. Wired to Ctrl/Cmd+S so users aren't stuck waiting out
   * `AUTOSAVE_DEBOUNCE_MS` when they explicitly ask to save. A no-op for
   * `localBackend` (already persisted synchronously by `useLocalStorage`),
   * but harmless to call unconditionally.
   */
  forceSave: () => void
  /**
   * Flushes a pending debounced save without cancelling the debounce timer
   * itself (unlike `forceSave`, which also bypasses scheduling a new one).
   * Wired to file-tab switches so an edit made just before switching away
   * from a file is persisted immediately instead of waiting out
   * `AUTOSAVE_DEBOUNCE_MS` — `shouldScheduleAutosave` deliberately does not
   * schedule a new save on such switches, so this is the only flush path
   * for that case. A no-op for `localBackend` or when nothing is pending.
   */
  flushPendingSave: () => void
  /**
   * Re-reads `backend.status()` into `saveStatus`. Needed after callers that
   * mutate a `CloudBackend`'s status by calling `saveContent`/`load`
   * directly rather than through `runSave` — namely
   * `StorageSettingsModal`'s conflict-resolution flow — so the "Saved"
   * badge in `FileTabBar` doesn't keep showing a stale status (e.g. the
   * conflict's `'error'`) after the conflict has actually been resolved.
   *
   * Pass `syncedStore` when the caller also just replaced the active file's
   * content with content it fetched from the backend (`discard-mine`'s
   * `updateActiveContent`, for instance): it's already in sync with what's
   * persisted, so this marks it as the new baseline the autosave-scheduling
   * effect diffs against. Omitting it would make that effect see the
   * content "change" on the next render and immediately schedule a
   * redundant autosave, flipping the badge back to "Unsaved" right after
   * this call set it to "Saved".
   */
  refreshSaveStatus: (syncedStore?: FileStoreState) => void
}
