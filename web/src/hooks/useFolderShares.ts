import { useCallback, useEffect, useRef, useState } from 'react'
import type { ShareOverview } from '../poentaroWasm'
import type { SyncedShareOperation } from '../syncedShare/errors'
import {
  callWorker,
  type createWorkerClient,
} from '../syncedShare/workerClient'
import { buildSyncedShareUrl } from '../syncedShareUrl'
import type { DisplaySaveStatus } from './useStorageBackend'

const NO_SHARES: ShareOverview = { files: [], folders: [] }

type WorkerClient = ReturnType<typeof createWorkerClient>

/** Sends one owner-side request and records its failure (see
 * `useSyncedShareOwner`'s `runShareRequest`). */
type RunShareRequest = <Data>(
  operation: SyncedShareOperation,
  send: (client: WorkerClient) => Promise<Data>,
  options?: { silent?: boolean },
) => Promise<{ data: Data } | null>

/** What the file list and the folder Share dialog use. */
export interface FolderShares {
  /** The owner's live links as the worker last reported them -- Rust turns
   * this into markers, "via" tags and warnings. Empty unless signed in with
   * the cloud backend. */
  overview: ShareOverview
  /** Whether folders can be shared live at all: cloud backend only. */
  canShare: boolean
  /** Starts (or resumes) the folder's link; resolves its URL, `null` on failure. */
  startSync: (folder: string, name: string) => Promise<string | null>
  stopSync: (folder: string) => Promise<void>
  linkFor: (shareId: string, name: string) => string
}

interface FolderSharesInput {
  identityToken: string | null
  /** The cloud backend is active. */
  isCloud: boolean
  /** Changes whenever the set of file/folder paths changes. */
  pathKey: string
  saveStatus: DisplaySaveStatus
  /** Changes when the open file's own link starts or stops. */
  fileSyncKey: string
  runShareRequest: RunShareRequest
}

/**
 * Keeps the owner's share overview (`POST /shares/overview`) fresh: on load,
 * after start/stop, and after the file paths change -- once right away and
 * again when the cloud save settles, since a folder rename/move reaches the
 * worker (which re-points the folder link) through the outbox a moment later.
 * Overview failures are silent; the markers are advisory.
 */
export function useFolderShares({
  identityToken,
  isCloud,
  pathKey,
  saveStatus,
  fileSyncKey,
  runShareRequest,
}: FolderSharesInput): FolderShares {
  const [overview, setOverview] = useState<ShareOverview>(NO_SHARES)
  const enabled = isCloud && identityToken !== null

  const refresh = useCallback(async () => {
    if (!enabled || !identityToken) {
      setOverview(NO_SHARES)
      return
    }
    const result = await runShareRequest(
      'status',
      (client) =>
        callWorker(
          client.POST('/shares/overview', { body: { identityToken } }),
        ),
      { silent: true },
    )
    if (result) setOverview(result.data)
  }, [enabled, identityToken, runShareRequest])

  const pathsChangedRef = useRef(false)
  useEffect(() => {
    if (pathsChangedRef.current && saveStatus === 'saved') {
      pathsChangedRef.current = false
      void refresh()
    }
  }, [saveStatus, refresh])
  // biome-ignore lint/correctness/useExhaustiveDependencies: `pathKey` is the trigger
  useEffect(() => {
    pathsChangedRef.current = true
    void refresh()
  }, [pathKey, fileSyncKey, refresh])

  const startSync = useCallback(
    async (folder: string, name: string): Promise<string | null> => {
      if (!enabled || !identityToken) return null
      const result = await runShareRequest('start', (client) =>
        callWorker(
          client.POST('/folders/share', { body: { identityToken, folder } }),
        ),
      )
      if (!result) return null
      await refresh()
      return buildSyncedShareUrl(result.data.shareId, name)
    },
    [enabled, identityToken, runShareRequest, refresh],
  )

  const stopSync = useCallback(
    async (folder: string): Promise<void> => {
      if (!enabled || !identityToken) return
      const result = await runShareRequest('stop', (client) =>
        callWorker(
          client.POST('/folders/share/stop', {
            body: { identityToken, folder },
          }),
        ),
      )
      if (result) await refresh()
    },
    [enabled, identityToken, runShareRequest, refresh],
  )

  return {
    overview: enabled ? overview : NO_SHARES,
    canShare: isCloud,
    startSync,
    stopSync,
    linkFor: buildSyncedShareUrl,
  }
}
