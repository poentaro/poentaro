import { useCallback, useEffect, useRef, useState } from 'react'
import { useAccountAuth } from '../storage/accountAuth'
import { signOutSession } from '../storage/accountRequests'
import {
  buildSyncedShareFailure,
  type SyncedShareFailure,
  type SyncedShareOperation,
} from '../syncedShare/errors'
import { callWorker, createWorkerClient } from '../syncedShare/workerClient'
import { buildSyncedShareUrl } from '../syncedShareUrl'
import { type FolderShares, useFolderShares } from './useFolderShares'
import type { DisplaySaveStatus } from './useStorageBackend'

/** Which account dialog is open, if any. */
export type AccountDialog = 'sign-in' | 'change-password' | 'api-keys'

/** A file's share state as last reported by the worker. Tagged with the
 * `fileId` it belongs to, so a response for a file the owner has since
 * switched away from can never be read as the current file's state. */
interface FileShareStatus {
  fileId: string
  shareId: string
  ended: boolean
}

export interface UseSyncedShareOwnerResult {
  /** Whether a live link is possible for the current file at all: only a
   * cloud-stored (account-owned) file can be shared live, since the link
   * points at its `files` row. Local and demo files get the static
   * `#share=` link only. */
  canSync: boolean
  isSynced: boolean
  syncedShareLink: string | null
  /** Whether this browser has an account session (`accountAuth.ts`). */
  isSignedIn: boolean
  /** The signed-in account's email, for the header chip and the "Synced
   * as" identity row -- `null` while signed out. */
  accountEmail: string | null
  /** Starts (or resumes) sharing this file and returns its viewer link, or
   * `null` if it couldn't. The worker always hands back the file's one
   * share id, so re-sharing -- from any device -- reproduces the same link.
   *
   * A no-op (resolves `null`) while signed out -- it never itself opens
   * the sign-in dialog. Callers (e.g. `ShareModal`) check `isSignedIn`
   * first and show their sign-in state instead of calling this. A failed
   * request (e.g. an expired session) resolves `null` too, surfaced instead
   * via `syncFailure` below. */
  startSync: () => Promise<string | null>
  stopSync: () => void
  /** The account dialog that is open, if any -- rendered by
   * `AccountDialogs`. */
  accountDialog: AccountDialog | null
  /** Opens the account dialog on its sign-in form. Signing in never itself
   * starts a share -- the user clicks "Start Sync" again once signed in. */
  openSignIn: () => void
  openChangePassword: () => void
  /** Opens the dialog that connects an AI agent with an API key. */
  openApiKeys: () => void
  closeAccountDialog: () => void
  /** Ends this browser's session: stops the current file's live share
   * first (mirrors clicking "Stop Sync"), tells the worker to end the
   * session, and clears the stored token so `isSignedIn` goes back to
   * `false`. */
  signOut: () => void
  /** Set whenever a request (start, stop, or the status lookup) fails --
   * the worker's `unauthorized` (the session ended), any other failure
   * response, or a network-level error. Drives the full-screen error dialog;
   * `null` means no failure is currently being shown. There is no
   * automatic retry -- dismissing it (`dismissSyncFailure`) just returns to
   * idle. An auth rejection additionally clears the stored session (see
   * `recordSyncFailure`), so the next "Sync" click re-prompts sign-in
   * instead of resending the same dead token forever. */
  syncFailure: SyncedShareFailure | null
  dismissSyncFailure: () => void
  /** Live links of whole folders (and the overview of every live link). */
  folderShares: FolderShares
}

/** What the folder side of the owner hook needs from the file store. */
export interface FolderShareContext {
  /** The cloud backend is active. */
  isCloud: boolean
  /** Changes whenever the set of file/folder paths changes. */
  pathKey: string
  saveStatus: DisplaySaveStatus
}

/**
 * Owns the owner side of a Synced Share for the current file. A share is a
 * pointer to the file's cloud `files` row (see
 * `crates/live-share-worker/src/share.rs`): viewers read the file's saved
 * content directly, so this hook never sends content anywhere -- the
 * ordinary cloud autosave is the only write path.
 *
 * Whether the file is shared lives on the server, fetched from `POST
 * /files/:id/share/status` whenever the file or the signed-in identity
 * changes, so every device the owner signs in on sees the same live/stopped
 * state and link. `isSynced` is derived from that status (and only counts
 * when it's tagged with the current file), never mirrored into separate
 * state, so a file switch can't briefly pair one file with another's share.
 *
 * Ownership is always a signed-in account -- there is no
 * anonymous/device-secret ownership path.
 */
export function useSyncedShareOwner(
  filename: string,
  /** The current file's cloud `files.id` (see `useScoreSource.ts`'s
   * `cloudFileIdFor`), `null` for a local or demo file, which can't be
   * shared live. */
  cloudFileId: string | null,
  folderContext: FolderShareContext,
): UseSyncedShareOwnerResult {
  const [status, setStatus] = useState<FileShareStatus | null>(null)
  const [syncFailure, setSyncFailure] = useState<SyncedShareFailure | null>(
    null,
  )
  const [accountAuth, setAccountAuth] = useAccountAuth()
  const [accountDialog, setAccountDialog] = useState<AccountDialog | null>(null)
  const isSignedIn = accountAuth !== null
  const accountEmail = accountAuth?.email ?? null
  const identityToken = accountAuth?.token ?? null

  // The file currently on screen, read by async completions below so a
  // response that lands after the owner switched files is dropped instead
  // of overwriting the new file's status.
  const cloudFileIdRef = useRef(cloudFileId)
  cloudFileIdRef.current = cloudFileId

  const applyStatusFor = useCallback(
    (fileId: string, next: FileShareStatus | null) => {
      if (cloudFileIdRef.current === fileId) setStatus(next)
    },
    [],
  )

  /** Records a request failure and, if it's an auth rejection (the worker's
   * `unauthorized` `ApiError` -- the session was signed out, expired, or
   * ended by a password change, not merely a transient network blip), also
   * clears the stored session: without this, a dead token keeps being
   * resent forever, and every retry just reproduces the same 401 and
   * re-shows this dialog instead of prompting a fresh sign-in. */
  const recordSyncFailure = useCallback(
    (failure: SyncedShareFailure) => {
      setSyncFailure(failure)
      if (failure.authRejected) setAccountAuth(null)
    },
    [setAccountAuth],
  )

  /** Sends one owner-side request (`send`, given the worker client).
   * Resolves `{ data }` with its success body, or `null` after recording the
   * failure. A `silent` request shows no dialog, but an auth rejection
   * still clears the stored session. */
  const runShareRequest = useCallback(
    async <Data>(
      operation: SyncedShareOperation,
      send: (client: ReturnType<typeof createWorkerClient>) => Promise<Data>,
      options?: { silent?: boolean },
    ): Promise<{ data: Data } | null> => {
      const host = import.meta.env.VITE_SYNCED_SHARE_HOST
      if (!host) return null
      try {
        return { data: await send(createWorkerClient(host)) }
      } catch (error) {
        const failure = buildSyncedShareFailure(operation, error)
        if (!options?.silent) {
          recordSyncFailure(failure)
        } else if (failure.authRejected) {
          setAccountAuth(null)
        }
        return null
      }
    },
    [recordSyncFailure, setAccountAuth],
  )

  useEffect(() => {
    if (!cloudFileId || !identityToken) return
    void runShareRequest('status', (client) =>
      callWorker(
        client.POST('/files/{id}/share/status', {
          params: { path: { id: cloudFileId } },
          body: { identityToken },
        }),
      ),
    ).then((result) => {
      if (!result) return
      const { share } = result.data
      applyStatusFor(
        cloudFileId,
        share ? { fileId: cloudFileId, ...share } : null,
      )
    })
  }, [cloudFileId, identityToken, runShareRequest, applyStatusFor])

  const startSync = useCallback(async (): Promise<string | null> => {
    // Never opens the sign-in dialog itself, it just declines to start a
    // share. `ShareModal` checks `isSignedIn` up front and shows its
    // sign-in state instead of calling this in that case -- this check is
    // a defensive backstop, not the primary gate.
    if (!cloudFileId || !identityToken) return null
    const result = await runShareRequest('start', (client) =>
      callWorker(
        client.POST('/files/{id}/share', {
          params: { path: { id: cloudFileId } },
          body: { identityToken },
        }),
      ),
    )
    if (!result) return null
    const { shareId } = result.data
    applyStatusFor(cloudFileId, {
      fileId: cloudFileId,
      shareId,
      ended: false,
    })
    return buildSyncedShareUrl(shareId, filename)
  }, [cloudFileId, identityToken, filename, runShareRequest, applyStatusFor])

  const isSynced =
    cloudFileId !== null &&
    identityToken !== null &&
    status?.fileId === cloudFileId &&
    !status.ended

  const stopSync = useCallback(() => {
    if (!isSynced || !cloudFileId || !identityToken || !status) return
    const stopped = { ...status, ended: true }
    void runShareRequest('stop', (client) =>
      callWorker(
        client.POST('/files/{id}/share/stop', {
          params: { path: { id: cloudFileId } },
          body: { identityToken },
        }),
      ),
    ).then((result) => {
      if (result) applyStatusFor(cloudFileId, stopped)
    })
  }, [
    isSynced,
    cloudFileId,
    identityToken,
    status,
    runShareRequest,
    applyStatusFor,
  ])

  const dismissSyncFailure = useCallback(() => {
    setSyncFailure(null)
  }, [])

  const openSignIn = useCallback(() => setAccountDialog('sign-in'), [])
  const openChangePassword = useCallback(
    () => setAccountDialog('change-password'),
    [],
  )
  const openApiKeys = useCallback(() => setAccountDialog('api-keys'), [])
  const closeAccountDialog = useCallback(() => setAccountDialog(null), [])

  const signOut = useCallback(() => {
    stopSync()
    // Ends only this device's session, so the owner stays signed in on
    // their other devices. Fire-and-forget: local state always wins, a
    // failed request never blocks signing out.
    if (accountAuth) signOutSession(accountAuth.token)
    setStatus(null)
    setAccountAuth(null)
  }, [stopSync, accountAuth, setAccountAuth])

  const folderShares = useFolderShares({
    identityToken,
    runShareRequest,
    fileSyncKey: `${cloudFileId ?? ''}:${isSynced}`,
    ...folderContext,
  })

  return {
    folderShares,
    canSync: cloudFileId !== null,
    isSynced,
    syncedShareLink:
      isSynced && status ? buildSyncedShareUrl(status.shareId, filename) : null,
    isSignedIn,
    accountEmail,
    startSync,
    stopSync,
    accountDialog,
    openSignIn,
    openChangePassword,
    openApiKeys,
    closeAccountDialog,
    signOut,
    syncFailure,
    dismissSyncFailure,
  }
}
