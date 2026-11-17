import * as Dialog from '@radix-ui/react-dialog'
import { type ReactElement, useEffect, useState } from 'react'
import { useDiffLayout } from '../../hooks/useDiffLayout'
import type {
  CloudBackend,
  FileVersion,
  FileVersionSummary,
} from '../../storage/cloudBackendTypes'
import { WorkerRequestError } from '../../syncedShare/workerClient'
import { DiffLayoutToggle, PoentaroDiffEditor } from '../PoentaroDiffEditor'
import { SpinnerLabel } from '../SpinnerLabel'
import './FileHistory.css'

export interface FileHistoryModalProps {
  open: boolean
  onOpenChange: (open: boolean) => void
  backend: CloudBackend
  fileId: string
  fileName: string
  /** The editor's current text, which a version is diffed against. */
  currentContent: string
  /** Puts `content` in the editor; it is saved like any edit, as a new
   * version. */
  onRestore: (content: string) => void
}

type Load<T> =
  | { status: 'loading' }
  | { status: 'ready'; value: T }
  | { status: 'error'; message: string }

function errorMessage(error: unknown): string {
  return error instanceof Error ? error.message : String(error)
}

/** A file the server has not got yet has no history, not an error. */
function isNotFound(error: unknown): boolean {
  return (
    error instanceof WorkerRequestError && error.apiError?.code === 'not_found'
  )
}

/** Runs `load` whenever `key` changes; a null `key` loads nothing. */
function useLoad<T>(key: string | null, load: () => Promise<T>): Load<T> {
  const [state, setState] = useState<Load<T>>({ status: 'loading' })
  // biome-ignore lint/correctness/useExhaustiveDependencies: `key` names everything `load` reads
  useEffect(() => {
    if (key === null) return
    let current = true
    setState({ status: 'loading' })
    load().then(
      (value) => current && setState({ status: 'ready', value }),
      (error) =>
        current && setState({ status: 'error', message: errorMessage(error) }),
    )
    return () => {
      current = false
    }
  }, [key])
  return state
}

const savedAtFormat = new Intl.DateTimeFormat(undefined, {
  dateStyle: 'medium',
  timeStyle: 'short',
})

export function FileHistoryModal({
  open,
  onOpenChange,
  backend,
  fileId,
  fileName,
  currentContent,
  onRestore,
}: FileHistoryModalProps): ReactElement {
  const versions = useLoad<FileVersionSummary[]>(open ? fileId : null, () =>
    backend
      .listVersions(fileId)
      .catch((error) => (isNotFound(error) ? [] : Promise.reject(error))),
  )
  const [picked, setPicked] = useState<number | null>(null)
  const newest =
    versions.status === 'ready' ? (versions.value[0]?.revision ?? null) : null
  const revision = picked ?? newest
  const version = useLoad<FileVersion>(
    open && revision !== null ? `${fileId}@${revision}` : null,
    () => backend.fetchVersion(fileId, revision ?? 0),
  )
  useEffect(() => {
    if (!open) setPicked(null)
  }, [open])

  return (
    <Dialog.Root open={open} onOpenChange={onOpenChange}>
      <Dialog.Portal>
        <Dialog.Overlay className="file-history__overlay" />
        <Dialog.Content
          className="file-history"
          data-testid="file-history"
          aria-describedby={undefined}
        >
          <header className="file-history__header">
            <Dialog.Title className="file-history__title">
              {fileName}
            </Dialog.Title>
            <Dialog.Close className="file-history__close" aria-label="Close">
              ×
            </Dialog.Close>
          </header>
          {versions.status === 'loading' ? (
            <p className="file-history__status">
              <SpinnerLabel pending label="Loading" />
            </p>
          ) : versions.status === 'error' ? (
            <p className="file-history__status" role="alert">
              {versions.message}
            </p>
          ) : versions.value.length === 0 ? (
            <p className="file-history__status">No earlier versions</p>
          ) : (
            <div className="file-history__body">
              <ul className="file-history__versions" aria-label="Versions">
                {versions.value.map((summary) => (
                  <li key={summary.revision}>
                    <button
                      type="button"
                      className="file-history__version"
                      aria-current={summary.revision === revision}
                      onClick={() => setPicked(summary.revision)}
                    >
                      {savedAtFormat.format(summary.savedAt)}
                    </button>
                  </li>
                ))}
              </ul>
              <VersionPane
                version={version}
                currentContent={currentContent}
                onRestore={(content) => {
                  onOpenChange(false)
                  onRestore(content)
                }}
              />
            </div>
          )}
        </Dialog.Content>
      </Dialog.Portal>
    </Dialog.Root>
  )
}

function VersionPane({
  version,
  currentContent,
  onRestore,
}: {
  version: Load<FileVersion>
  currentContent: string
  onRestore: (content: string) => void
}): ReactElement {
  const { layout, toggle } = useDiffLayout()
  if (version.status === 'loading') {
    return (
      <div className="file-history__pane">
        <p className="file-history__status">
          <SpinnerLabel pending label="Loading" />
        </p>
      </div>
    )
  }
  if (version.status === 'error') {
    return (
      <div className="file-history__pane">
        <p className="file-history__status" role="alert">
          {version.message}
        </p>
      </div>
    )
  }
  return (
    <div className="file-history__pane">
      <div className="file-history__diff" data-testid="file-history-diff">
        <PoentaroDiffEditor
          original={currentContent}
          modified={version.value.content}
          layout={layout}
        />
      </div>
      <div className="file-history__actions">
        <DiffLayoutToggle
          layout={layout}
          onToggle={toggle}
          testId="file-history-layout-toggle"
        />
        <button
          type="button"
          className="file-dialog-button file-dialog-button--primary"
          onClick={() => onRestore(version.value.content)}
        >
          Restore
        </button>
      </div>
    </div>
  )
}
