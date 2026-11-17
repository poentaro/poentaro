import { useEffect, useState } from 'react'
import type { FileStoreState } from '../fileStore'
import type { StorageBackend } from '../storage/types'
import { traceIdOf, WorkerRequestError } from '../syncedShare/workerClient'

const INITIAL_RETRY_DELAY_MS = 1_000
const MAX_RETRY_DELAY_MS = 30_000

/** Exponential backoff (1s, 2s, 4s, ...) capped at `MAX_RETRY_DELAY_MS`
 * between failed attempts to load the cloud listing. `attempt` is the
 * zero-based index of the attempt that just failed. */
export function cloudLoadRetryDelayMs(attempt: number): number {
  return Math.min(INITIAL_RETRY_DELAY_MS * 2 ** attempt, MAX_RETRY_DELAY_MS)
}

/** True when the worker rejected the stored identity token itself (its
 * `unauthorized` `ApiError` -- e.g. the session expired, or a password
 * change elsewhere ended it), as opposed to a transient network/server failure.
 * Retrying such a token can never succeed. */
export function isAuthRejection(error: unknown): boolean {
  return (
    error instanceof WorkerRequestError &&
    error.apiError?.code === 'unauthorized'
  )
}

/** The last failed load while a retry is pending. `traceId` is `null` when
 * the request never reached the worker. */
export interface CloudLoadFailure {
  traceId: string | null
}

/**
 * (Re)loads the cloud listing whenever the backend identity changes (kind or
 * token). A failed load used to leave `cloudStore` at `null` forever (an
 * endless loading spinner) until the backend identity changed -- in
 * practice, until the user signed out and back in. Now:
 *
 * - An auth rejection clears the stored token via `clearAccountAuth`, which
 *   makes `useStorageBackend` fall back to `localBackend` and the UI offer
 *   a fresh sign-in, same as `useSyncedShareOwner`'s `recordSyncFailure`.
 * - Any other failure (worker mid-deploy, a 5xx, offline) is retried with
 *   `cloudLoadRetryDelayMs` backoff, and immediately when the browser comes
 *   back online.
 *
 * Returns the last failure until a load succeeds, so the UI can say the
 * listing is being retried and show the failed request's trace ID.
 */
export function useCloudStoreLoader(
  backend: StorageBackend,
  setCloudStore: (state: FileStoreState | null) => void,
  clearAccountAuth: () => void,
): CloudLoadFailure | null {
  const [failure, setFailure] = useState<CloudLoadFailure | null>(null)

  useEffect(() => {
    if (backend.kind !== 'cloud') return
    let cancelled = false
    let attempt = 0
    let retryTimer: ReturnType<typeof setTimeout> | undefined

    const load = () => {
      clearTimeout(retryTimer)
      retryTimer = undefined
      backend.load().then(
        (state) => {
          if (cancelled) return
          setFailure(null)
          setCloudStore(state)
        },
        (error: unknown) => {
          if (cancelled) return
          if (isAuthRejection(error)) {
            clearAccountAuth()
            return
          }
          setFailure({ traceId: traceIdOf(error) })
          retryTimer = setTimeout(load, cloudLoadRetryDelayMs(attempt))
          attempt += 1
        },
      )
    }
    // Only when a retry is actually waiting -- never double up on a load
    // that's still in flight.
    const retryNow = () => {
      if (retryTimer !== undefined) load()
    }

    setCloudStore(null)
    setFailure(null)
    load()
    window.addEventListener('online', retryNow)
    return () => {
      cancelled = true
      clearTimeout(retryTimer)
      window.removeEventListener('online', retryNow)
    }
  }, [backend, setCloudStore, clearAccountAuth])

  return backend.kind === 'cloud' ? failure : null
}
