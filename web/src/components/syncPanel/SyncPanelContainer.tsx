import { useEffect, useMemo, useState } from 'react'
import { type FileStoreState, loadedBinText, loadedText } from '../../fileStore'
import { useFocusRevisionCheck } from '../../hooks/useFocusRevisionCheck'
import type { CloudBackend, LocalRename } from '../../storage/cloudBackendTypes'
import type { ConflictDetails } from '../../storage/outbox/conflictFlow'
import { keepBoth } from '../../storage/outbox/keepBoth'
import type { OutboxSnapshot } from '../../storage/outbox/outboxLooper'
import type { StorageBackend } from '../../storage/types'
import { MergeEditorContainer } from '../mergeEditor/MergeEditorContainer'
import { OutboxHistoryDialog } from '../outboxHistory/OutboxHistoryDialog'
import { buildDiagnostics, copyDiagnostics } from './copyDiagnostics'
import { DiscardConfirmDialog } from './DiscardConfirmDialog'
import { downloadLocalCopy } from './downloadLocalCopy'
import { toLaneViews } from './laneViewMapping'

export interface SyncPanelContainerProps {
  open: boolean
  onOpenChange: (open: boolean) => void
  backend: StorageBackend
  store: FileStoreState
  /** Puts the server's text into the open store, replacing the local one. */
  onServerContent: (fileId: string, content: string) => void
  /** Applies a rename the outbox made after a name collision. */
  onLocalRename: (change: LocalRename) => void
  /** Replaces the whole open store, e.g. after keeping both versions. */
  replaceStore: (next: FileStoreState) => void
  /** An autosave is armed: typed text has not reached the outbox yet. */
  hasUnsavedEdits: boolean
}

function nameOfFile(store: FileStoreState, fileId: string): string {
  return (
    Object.entries(store.fileIds).find(([, id]) => id === fileId)?.[0] ?? fileId
  )
}

function useOutboxSnapshot(backend: CloudBackend): OutboxSnapshot {
  const looper = backend.outbox()
  const [snapshot, setSnapshot] = useState(() => looper.snapshot())
  useEffect(() => {
    setSnapshot(looper.snapshot())
    return looper.subscribe(setSnapshot)
  }, [looper])
  return snapshot
}

function CloudSyncPanel(
  props: SyncPanelContainerProps & { backend: CloudBackend },
) {
  const { backend, store } = props
  const snapshot = useOutboxSnapshot(backend)
  const [discardingFileId, setDiscardingFileId] = useState<string | null>(null)
  const [mergingFileId, setMergingFileId] = useState<string | null>(null)
  const [notice, setNotice] = useState<string | null>(null)
  const historyFiles = useMemo(
    () =>
      Object.entries(store.fileIds).map(([name, fileId]) => ({ fileId, name })),
    [store.fileIds],
  )
  useEffect(
    () => backend.onLocalRename(props.onLocalRename),
    [backend, props.onLocalRename],
  )
  useEffect(() => backend.onNotice(setNotice), [backend])
  useEffect(() => {
    if (notice === null) return
    const timer = setTimeout(() => setNotice(null), 6_000)
    return () => clearTimeout(timer)
  }, [notice])
  useFocusRevisionCheck({
    backend,
    activeFileId: store.fileIds[store.active],
    onServerContent: props.onServerContent,
    hasUnsavedEdits: props.hasUnsavedEdits,
  })
  const conflictOf = (fileId: string | null): ConflictDetails | null =>
    backend
      .conflicts()
      ?.conflicts()
      .find((candidate) => candidate.fileId === fileId) ?? null
  const keepBothVersions = async (fileId: string) => {
    const today = new Date().toISOString().slice(0, 10)
    props.replaceStore(await keepBoth({ backend, state: store, fileId, today }))
  }
  const lanes = toLaneViews(snapshot, (fileId) => nameOfFile(store, fileId))

  const download = (fileId: string) => {
    const name = nameOfFile(store, fileId)
    const text = loadedText(store, name) ?? loadedBinText(store, name)
    if (text !== undefined) downloadLocalCopy(name, text)
  }

  const discard = async (fileId: string) => {
    setDiscardingFileId(null)
    await backend.outbox().resolve(fileId, { tag: 'discard' })
    const server = await backend.fetchServerFile(fileId)
    if (server) props.onServerContent(fileId, server.content)
  }

  return (
    <>
      {notice !== null && (
        <div
          role="status"
          className="sync-panel__notice"
          data-testid="sync-notice"
        >
          {notice}
        </div>
      )}
      <OutboxHistoryDialog
        open={props.open}
        onOpenChange={props.onOpenChange}
        looper={backend.outbox()}
        files={historyFiles}
        onCopyDiagnostics={() => void copyDiagnostics(buildDiagnostics(lanes))}
        syncPanel={{
          lanes,
          onRetryNow: (fileId) =>
            void backend.outbox().resolve(fileId, { tag: 'retry' }),
          onReviewAndMerge: setMergingFileId,
          onKeepBoth: (fileId) => void keepBothVersions(fileId),
          onDiscard: setDiscardingFileId,
          onDownloadCopy: download,
        }}
      />
      <MergeEditorContainer
        conflict={conflictOf(mergingFileId)}
        fileName={nameOfFile(store, mergingFileId ?? '')}
        looper={backend.outbox()}
        onKeepBoth={keepBothVersions}
        onDiscard={setDiscardingFileId}
        onClose={() => setMergingFileId(null)}
      />
      <DiscardConfirmDialog
        open={discardingFileId !== null}
        fileName={nameOfFile(store, discardingFileId ?? '')}
        onDownloadFirst={() => {
          if (discardingFileId) download(discardingFileId)
        }}
        onConfirm={() => {
          if (discardingFileId) void discard(discardingFileId)
        }}
        onCancel={() => setDiscardingFileId(null)}
      />
    </>
  )
}

export function SyncPanelContainer(props: SyncPanelContainerProps) {
  const { backend } = props
  if (backend.kind !== 'cloud') return null
  return <CloudSyncPanel {...props} backend={backend as CloudBackend} />
}
