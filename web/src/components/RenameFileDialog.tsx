import * as Dialog from '@radix-ui/react-dialog'
import { useEffect, useRef, useState } from 'react'
import { displayFileName } from '../fileStore'
import './FileDialogs.css'

export interface RenameFileDialogProps {
  open: boolean
  onOpenChange: (open: boolean) => void
  /** Full path of the file or folder being renamed. */
  path: string
  /** A folder is shown with its whole path as is; a file without `.poentaro`. */
  kind?: 'file' | 'folder'
  onRename: (from: string, to: string) => void
  /** Rust's reason a rename/move would be rejected, if it would. */
  renameError: (from: string, to: string) => string | undefined
}

/** Renames (and, when the name contains `/`, moves) a file or folder. The input
 * holds the whole path, so the folders can be edited too; Rust explains why a
 * path is rejected, live, instead of the rename silently doing nothing. */
export function RenameFileDialog({
  open,
  onOpenChange,
  path,
  kind = 'file',
  onRename,
  renameError,
}: RenameFileDialogProps) {
  const initial = kind === 'folder' ? path : displayFileName(path)
  const [draft, setDraft] = useState(initial)
  const inputRef = useRef<HTMLInputElement>(null)

  useEffect(() => {
    if (open) setDraft(initial)
  }, [open, initial])

  const trimmed = draft.trim()
  // Rust is only asked while open: the component is mounted before the
  // wasm component has necessarily finished loading.
  const error = open && trimmed ? renameError(path, trimmed) : undefined
  const canConfirm = trimmed !== '' && error === undefined

  const confirm = () => {
    if (!canConfirm) return
    onOpenChange(false)
    if (trimmed !== initial) onRename(path, trimmed)
  }

  return (
    <Dialog.Root open={open} onOpenChange={onOpenChange}>
      <Dialog.Portal>
        <Dialog.Overlay className="file-dialog-overlay" />
        <Dialog.Content
          className="file-dialog"
          aria-describedby={undefined}
          onOpenAutoFocus={(event) => {
            event.preventDefault()
            inputRef.current?.focus()
            inputRef.current?.select()
          }}
        >
          <div className="file-dialog-header">
            <Dialog.Title className="file-dialog-title">
              {kind === 'folder' ? 'Rename folder' : 'Rename'}
            </Dialog.Title>
            <Dialog.Close className="file-dialog-close" aria-label="Close">
              ×
            </Dialog.Close>
          </div>
          <form
            onSubmit={(event) => {
              event.preventDefault()
              confirm()
            }}
          >
            <div className="file-dialog-body">
              <p className="file-dialog-hint">
                {kind === 'folder'
                  ? 'Use / to move the folder into another folder.'
                  : 'Use / to put the file in a folder.'}
              </p>
              <input
                ref={inputRef}
                type="text"
                className="file-dialog-input"
                aria-label={kind === 'folder' ? 'Folder path' : 'File path'}
                aria-invalid={error !== undefined}
                value={draft}
                onChange={(event) => setDraft(event.target.value)}
              />
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
                type="submit"
                className="file-dialog-button file-dialog-button--primary"
                disabled={!canConfirm}
              >
                Rename
              </button>
            </div>
          </form>
        </Dialog.Content>
      </Dialog.Portal>
    </Dialog.Root>
  )
}
