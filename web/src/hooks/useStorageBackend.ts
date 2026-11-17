import { useCallback, useEffect, useMemo, useRef, useState } from 'react'
import { useDebouncedCallback } from 'use-debounce'
import { useLocalStorage } from 'usehooks-ts'
import {
  DEMO_FILE_NAMES,
  FILE_STORE_KEY,
  type FileStoreState,
  fileContent,
} from '../fileStore'
import { useAccountAuth } from '../storage/accountAuth'
import { createCloudBackend } from '../storage/cloudBackend'
import type { CloudBackend } from '../storage/cloudBackendTypes'
import {
  deserializeStoreSync,
  localBackend,
  readInitialStoreSync,
} from '../storage/localBackend'
import type { SaveStatus, StorageBackend } from '../storage/types'
import {
  hasUnsyncedWork,
  outboxSaveBadge,
  shouldWarnBeforeUnload,
} from './outboxSaveBadge'
import { useCloudOutbox } from './useCloudOutbox'
import { useCloudStoreLoader } from './useCloudStoreLoader'
import type { UseStorageBackendResult } from './useStorageBackendResult'
import { useSwitchToCloudOnSignIn } from './useSwitchToCloudOnSignIn'

/**
 * Idle interval at which a changed active file's content is flushed to the
 * backend via `saveContent()`. Shared by both backends — immaterial for
 * `localBackend` (whose `saveContent` is a no-op, since `useLocalStorage`
 * already persists every state change) — so there is a single autosave
 * cadence instead of a redundant per-backend config knob. Chosen to keep
 * write frequency to the cloud backend low while a file is being actively
 * edited.
 */
export const AUTOSAVE_DEBOUNCE_MS = 20_000

const STORAGE_BACKEND_PREFERENCE_KEY = 'poentaro:storage-backend:v1'

/**
 * Persisted choice of backend. Unlike the old GitHub backend, `'cloud'`
 * carries no connection details of its own — it's entirely reconstructed
 * from the one shared `accountAuth.ts` token (see `backend` below), so
 * there's no repo/owner concept to persist alongside the choice.
 */
export interface StorageBackendPreference {
  backend: 'local' | 'cloud'
}

const DEFAULT_PREFERENCE: StorageBackendPreference = { backend: 'local' }

/** Placeholder `FileStoreState` shown for the brief window between
 * selecting the cloud backend and `backend.load()` resolving. Identical in
 * shape to `fileStore.ts`'s own `DEFAULT_FILE_STORE` (not exported from
 * there, so reconstructed here). */
const EMPTY_STORE: FileStoreState = {
  active: DEMO_FILE_NAMES[0] ?? '',
  userFiles: {},
  bin: {},
  fileIds: {},
}

export type StorageBackendTarget = { kind: 'local' } | { kind: 'cloud' }

/**
 * `SaveStatus` plus a UI-only `'unsaved'` state: a debounced edit is pending
 * but hasn't been handed to `backend.saveContent` yet. Derived from
 * `autosaveDeadline` in this hook rather than tracked by `StorageBackend`
 * itself, since it describes debounce state (owned here, see
 * `AUTOSAVE_DEBOUNCE_MS`) rather than backend/network state.
 */
export type DisplaySaveStatus = SaveStatus | 'unsaved' | 'waiting' | 'attention'

/**
 * Decides whether an autosave should be scheduled for a `store` transition.
 * Pure (no backend/React access) so it's unit-testable in isolation: a save
 * is only warranted when the *same* active file's content actually changed
 * — not merely when the user switched which file is active (whose content
 * naturally differs from the previously active file's) — and never for a
 * non-cloud backend, since only the cloud backend's `saveContent` does real
 * work. A switch away from a file with a still-*pending* debounced save is
 * not covered by "already saved" — that's instead handled by
 * `flushPendingSave`, which callers should invoke before changing the
 * active file.
 */
type ContentSnapshot = { active: string; content: string }

function activeContentSnapshot(store: FileStoreState): ContentSnapshot {
  return { active: store.active, content: fileContent(store, store.active) }
}

export function shouldScheduleAutosave(
  backendKind: StorageBackend['kind'],
  previous: ContentSnapshot | null,
  next: ContentSnapshot,
): boolean {
  if (backendKind !== 'cloud') return false
  if (!previous) return false
  return previous.active === next.active && previous.content !== next.content
}

/**
 * Maps a `StorageBackend.status()` reading to the `SaveStatus` shown in the
 * UI: `'idle'` alone doesn't tell the user whether the store is idle because
 * it was just persisted or because nothing has happened yet, so both
 * `runSave` (after an autosave/force-save lands) and `refreshSaveStatus`
 * (after a conflict resolution mutates the backend's status directly)
 * normalize it to the more informative `'saved'`.
 */
function displaySaveStatus(status: SaveStatus): SaveStatus {
  return status === 'idle' ? 'saved' : status
}

/**
 * Holds the file store's in-memory state and wires it to a `StorageBackend`,
 * switchable between `local` and `cloud`. `store`/`setStore` keep the same
 * ergonomics `useFileStore` used to expose, so callers still perform sync
 * updates (e.g. selecting a file, or applying `backend.updateActiveContent`)
 * via `setStore`. Structural operations (create/duplicate/rename/delete/
 * restore) are modeled as async on `StorageBackend`, so callers
 * `await backend.xxxFile(store)` and then reconcile the result into the
 * latest state with `fileStore.ts`'s `mergeBackendResult` via a functional
 * `setStore(prev => ...)` update — never a plain `setStore(next)`, which
 * would discard any edits made to `prev` while the await was in flight.
 * Those calls hit the backend immediately, unlike content edits (see
 * `AUTOSAVE_DEBOUNCE_MS`).
 *
 * Signing in switches `preference` to `'cloud'` (see
 * `useSwitchToCloudOnSignIn.ts`). The `cloud` backend below is only ever constructed once both `preference`
 * asks for it *and* an `accountAuth` token is present; losing either one
 * (switching back to local, or the token being cleared by a disconnect
 * anywhere in the app) falls back to `localBackend` on the very next
 * render, with no extra plumbing needed.
 */
export function useStorageBackend(): UseStorageBackendResult {
  const [preference, setPreference] = useLocalStorage<StorageBackendPreference>(
    STORAGE_BACKEND_PREFERENCE_KEY,
    DEFAULT_PREFERENCE,
  )
  const [accountAuth, setAccountAuth] = useAccountAuth()
  useSwitchToCloudOnSignIn(accountAuth, setPreference)

  const [localStore, setLocalStore] = useLocalStorage<FileStoreState>(
    FILE_STORE_KEY,
    readInitialStoreSync,
    { deserializer: deserializeStoreSync },
  )
  const [cloudStore, setCloudStore] = useState<FileStoreState | null>(null)
  const [saveStatus, setSaveStatus] = useState<SaveStatus>('idle')
  const [autosaveDeadline, setAutosaveDeadline] = useState<number | null>(null)

  const clearAccountAuth = useCallback(
    () => setAccountAuth(null),
    [setAccountAuth],
  )

  const backend = useMemo<StorageBackend | CloudBackend>(() => {
    if (preference.backend === 'cloud' && accountAuth) {
      return createCloudBackend({
        token: accountAuth.token,
        account: accountAuth.email,
        workerHost: import.meta.env.VITE_SYNCED_SHARE_HOST ?? '',
        onSessionEnded: clearAccountAuth,
      })
    }
    return localBackend
  }, [preference, accountAuth, clearAccountAuth])

  // `localBackend`'s state instead lives in `localStore` above, seeded
  // synchronously, so no loader is needed for it.
  const cloudLoadFailure = useCloudStoreLoader(
    backend,
    setCloudStore,
    clearAccountAuth,
  )

  useEffect(() => {
    setSaveStatus(backend.status())
  }, [backend])

  const outboxSnapshot = useCloudOutbox(backend)

  const store =
    backend.kind === 'cloud' ? (cloudStore ?? EMPTY_STORE) : localStore
  const isLoadingCloud = backend.kind === 'cloud' && cloudStore === null

  const setStore = useCallback(
    (value: FileStoreState | ((prev: FileStoreState) => FileStoreState)) => {
      if (backend.kind === 'cloud') {
        setCloudStore((prev) => {
          const base = prev ?? EMPTY_STORE
          return typeof value === 'function' ? value(base) : value
        })
      } else {
        setLocalStore(value)
      }
    },
    [backend, setLocalStore],
  )

  const pendingSaveRef = useRef<Promise<void> | null>(null)
  /** Saves handed to the backend that it has not yet accepted (for cloud: not
   * yet recorded in the outbox). */
  const handingOverRef = useRef(0)
  const lastContentRef = useRef<ContentSnapshot | null>(null)
  const runSave = useCallback(
    (state: FileStoreState) => {
      // Mark accounted-for now — a same-content run of the scheduling effect
      // below, delayed by React under load, could otherwise re-arm
      // autosaveDeadline right after this clears it.
      lastContentRef.current = activeContentSnapshot(state)
      setAutosaveDeadline(null)
      setSaveStatus('saving')
      handingOverRef.current += 1
      const promise = backend
        .saveContent(state)
        .then(() => {
          setSaveStatus(displaySaveStatus(backend.status()))
        })
        .catch(() => {
          setSaveStatus(backend.status())
        })
        .finally(() => {
          handingOverRef.current -= 1
        })
      pendingSaveRef.current = promise
    },
    [backend],
  )
  const debouncedSave = useDebouncedCallback(runSave, AUTOSAVE_DEBOUNCE_MS)

  useEffect(() => {
    const next = activeContentSnapshot(store)
    if (shouldScheduleAutosave(backend.kind, lastContentRef.current, next)) {
      debouncedSave(store)
      setAutosaveDeadline(Date.now() + AUTOSAVE_DEBOUNCE_MS)
    }
    lastContentRef.current = next
  }, [store, backend, debouncedSave])

  // Flushes a pending debounced save on blur/tab-hide so edits made just
  // before closing the tab aren't lost waiting for the idle interval.
  useEffect(() => {
    const flush = () => debouncedSave.flush()
    window.addEventListener('blur', flush)
    document.addEventListener('visibilitychange', flush)
    return () => {
      window.removeEventListener('blur', flush)
      document.removeEventListener('visibilitychange', flush)
    }
  }, [debouncedSave])

  // Warns before closing/reloading the tab if a cloud save hasn't landed
  // yet — the blur/visibilitychange flush above only *starts* the save; it
  // can't guarantee the underlying request finishes before the page
  // actually unloads. Native confirmation is the only backstop for that
  // gap. No-op (and thus no dialog) whenever nothing is pending or in
  // flight, including for `localBackend`, which never has a pending save.
  useEffect(() => {
    const handler = (event: BeforeUnloadEvent) => {
      // The live outbox, not `outboxSnapshot`, which lags a just-handed-over save.
      const cloud = backend.kind === 'cloud' ? (backend as CloudBackend) : null
      if (
        shouldWarnBeforeUnload(
          backend.kind,
          debouncedSave.isPending() || handingOverRef.current > 0,
          cloud !== null && hasUnsyncedWork(cloud.outbox().snapshot()),
        )
      ) {
        event.preventDefault()
      }
    }
    window.addEventListener('beforeunload', handler)
    return () => window.removeEventListener('beforeunload', handler)
  }, [backend, debouncedSave])

  const forceSave = useCallback(() => {
    debouncedSave.cancel()
    runSave(store)
  }, [debouncedSave, runSave, store])

  const flushPendingSave = useCallback(() => {
    if (backend.kind === 'cloud' && debouncedSave.isPending()) {
      debouncedSave.flush()
    }
  }, [backend, debouncedSave])

  const refreshSaveStatus = useCallback(
    (syncedStore?: FileStoreState) => {
      if (syncedStore) {
        lastContentRef.current = activeContentSnapshot(syncedStore)
        setAutosaveDeadline(null)
      }
      setSaveStatus(displaySaveStatus(backend.status()))
    },
    [backend],
  )

  const switchBackend = useCallback(
    async (target: StorageBackendTarget) => {
      if (backend.kind === 'cloud' && debouncedSave.isPending()) {
        debouncedSave.flush()
        await pendingSaveRef.current
      }
      if (target.kind === 'local') {
        setPreference({ backend: 'local' })
        setLocalStore((prev) => ({ ...prev, active: DEMO_FILE_NAMES[0] ?? '' }))
      } else {
        setPreference({ backend: 'cloud' })
      }
    },
    [backend, debouncedSave, setPreference, setLocalStore],
  )

  const outboxBadge = outboxSnapshot ? outboxSaveBadge(outboxSnapshot) : null
  const baseStatus: DisplaySaveStatus = outboxBadge?.status ?? saveStatus

  return {
    store,
    setStore,
    backend,
    isLoadingCloud,
    cloudLoadFailure,
    saveStatus: autosaveDeadline !== null ? 'unsaved' : baseStatus,
    saveLabel: autosaveDeadline !== null ? null : (outboxBadge?.label ?? null),
    autosaveDeadline,
    preference,
    switchBackend,
    forceSave,
    flushPendingSave,
    refreshSaveStatus,
  }
}
