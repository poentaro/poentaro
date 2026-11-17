import { useEffect, useRef } from 'react'
import type { CloudBackend, ServerFile } from '../storage/cloudBackendTypes'

export interface FocusRevisionDependencies {
  fetchServerFile(fileId: string): Promise<ServerFile | undefined>
  recordedRevision(fileId: string): bigint | undefined
  hasPendingMessages(fileId: string): boolean
  /** Typed text that has not reached the outbox yet (an armed autosave). */
  hasUnsavedEdits(): boolean
  applyServerFile(
    fileId: string,
    revision: number,
    content: string,
  ): Promise<void>
}

/** Loads the server's copy of `fileId` when it is ahead of the recorded
 * revision and nothing is waiting to be sent for that file. */
export async function checkFocusRevision(
  dependencies: FocusRevisionDependencies,
  fileId: string,
): Promise<void> {
  if (dependencies.hasUnsavedEdits() || dependencies.hasPendingMessages(fileId))
    return
  const serverFile = await dependencies.fetchServerFile(fileId)
  if (!serverFile) return
  const recorded = dependencies.recordedRevision(fileId)
  if (recorded !== undefined && BigInt(serverFile.revision) <= recorded) return
  // Messages may have been queued while the fetch was in flight.
  if (dependencies.hasUnsavedEdits() || dependencies.hasPendingMessages(fileId))
    return
  await dependencies.applyServerFile(
    fileId,
    serverFile.revision,
    serverFile.content,
  )
}

function focusRevisionDependencies(
  backend: CloudBackend,
  onServerContent: (fileId: string, content: string) => void,
  hasUnsavedEdits: () => boolean,
): FocusRevisionDependencies {
  return {
    hasUnsavedEdits,
    fetchServerFile: (fileId) => backend.fetchServerFile(fileId),
    recordedRevision: (fileId) =>
      backend
        .outbox()
        .snapshot()
        .queue.revisions.find((entry) => entry.fileId === fileId)?.revision,
    hasPendingMessages: (fileId) =>
      backend
        .outbox()
        .snapshot()
        .queue.lanes.some(
          (lane) => lane.fileId === fileId && lane.messages.length > 0,
        ),
    async applyServerFile(fileId, revision, content) {
      await backend.bases().recordBase(fileId, content)
      await backend.outbox().recordRevision(fileId, BigInt(revision))
      onServerContent(fileId, content)
    },
  }
}

/** How often a visible tab checks the open file for a save made elsewhere,
 * such as by the owner's AI agent (`POST /agent`). */
export const REVISION_POLL_MILLIS = 5000

/** Loads saves made elsewhere into the open file: when the tab comes back
 * into view or online, and every `REVISION_POLL_MILLIS` while it is in
 * view. Unsaved edits are never replaced; their save meets the newer
 * revision and goes through the outbox's conflict flow instead. */
export function useFocusRevisionCheck(options: {
  backend: CloudBackend | undefined
  activeFileId: string | undefined
  onServerContent: (fileId: string, content: string) => void
  hasUnsavedEdits: boolean
}): void {
  const { backend, activeFileId, onServerContent } = options
  const unsavedRef = useRef(options.hasUnsavedEdits)
  unsavedRef.current = options.hasUnsavedEdits
  useEffect(() => {
    if (!backend || !activeFileId) return
    const dependencies = focusRevisionDependencies(
      backend,
      onServerContent,
      () => unsavedRef.current,
    )
    const check = () => {
      if (document.visibilityState === 'hidden') return
      checkFocusRevision(dependencies, activeFileId).catch(() => undefined)
    }
    document.addEventListener('visibilitychange', check)
    window.addEventListener('online', check)
    const poll = window.setInterval(check, REVISION_POLL_MILLIS)
    return () => {
      document.removeEventListener('visibilitychange', check)
      window.removeEventListener('online', check)
      window.clearInterval(poll)
    }
  }, [backend, activeFileId, onServerContent])
}
