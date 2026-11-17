import * as Dialog from '@radix-ui/react-dialog'
import type { CSSProperties } from 'react'
import { useEffect, useState } from 'react'
import { displayFileName } from '../fileStore'
import type { MoveTarget } from '../poentaroWasm'
import './FileDialogs.css'

export interface MoveToDialogProps {
  open: boolean
  onOpenChange: (open: boolean) => void
  /** Full path of the file being moved. */
  path: string
  /** What the hint calls the thing being moved; defaults to the file's name. */
  subject?: string
  /** Folder currently holding the file (`undefined` at the top level). */
  currentFolder: string | undefined
  /** Whether the thing being moved sits in one folder that is not a valid
   * choice (false for a selection spread over several folders). */
  hasCurrentFolder?: boolean
  /** The picker's choices, computed in Rust. */
  targets: MoveTarget[]
  /** Where `path` lands in the folder (`undefined` = top level), from Rust. */
  destination: (folder: string | undefined) => string | undefined
  /** Rust's reason moving `path` to `to` would be rejected, if it would. */
  moveError: (to: string) => string | undefined
  onMove: (from: string, to: string) => void
}

/** Folder picker for "Move to…". Picking a folder previews the move: Rust
 * says whether it clashes, and the message shows before anything changes. */
export function MoveToDialog({
  open,
  onOpenChange,
  path,
  subject,
  currentFolder,
  hasCurrentFolder = true,
  targets,
  destination,
  moveError,
  onMove,
}: MoveToDialogProps) {
  const [selected, setSelected] = useState(currentFolder ?? '')

  useEffect(() => {
    if (open) setSelected(currentFolder ?? '')
  }, [open, currentFolder])

  // Rust is only asked while open: the component is mounted before the
  // wasm component has necessarily finished loading.
  const target = open
    ? destination(selected === '' ? undefined : selected)
    : undefined
  const error = target === undefined ? undefined : moveError(target)
  const isCurrent = hasCurrentFolder && selected === (currentFolder ?? '')
  const canConfirm = target !== undefined && error === undefined && !isCurrent

  return (
    <Dialog.Root open={open} onOpenChange={onOpenChange}>
      <Dialog.Portal>
        <Dialog.Overlay className="file-dialog-overlay" />
        <Dialog.Content className="file-dialog" aria-describedby={undefined}>
          <div className="file-dialog-header">
            <Dialog.Title className="file-dialog-title">Move to</Dialog.Title>
            <Dialog.Close className="file-dialog-close" aria-label="Close">
              ×
            </Dialog.Close>
          </div>
          <div className="file-dialog-body">
            <p className="file-dialog-hint">
              {`Move ${subject ?? displayFileName(path)} into:`}
            </p>
            <div className="move-targets">
              {targets.map((folder) => (
                <button
                  key={folder.path}
                  type="button"
                  aria-pressed={selected === folder.path}
                  className="move-target"
                  style={{ '--move-depth': folder.depth } as CSSProperties}
                  onClick={() => setSelected(folder.path)}
                >
                  <span className="move-target-name">{folder.name}</span>
                  {hasCurrentFolder && folder.path === (currentFolder ?? '') ? (
                    <span className="move-target-current">current</span>
                  ) : null}
                </button>
              ))}
            </div>
            {error ? (
              <p className="file-dialog-warn" role="alert">
                {error}
              </p>
            ) : null}
          </div>
          <div className="file-dialog-footer">
            <button
              type="button"
              className="file-dialog-button"
              onClick={() => onOpenChange(false)}
            >
              Cancel
            </button>
            <button
              type="button"
              className="file-dialog-button file-dialog-button--primary"
              disabled={!canConfirm}
              onClick={() => {
                if (target === undefined) return
                onOpenChange(false)
                onMove(path, target)
              }}
            >
              Move
            </button>
          </div>
        </Dialog.Content>
      </Dialog.Portal>
    </Dialog.Root>
  )
}
