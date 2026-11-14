import * as Dialog from '@radix-ui/react-dialog'
import { useEffect, useRef, useState } from 'react'
import { useWasmReady } from '../hooks/useWasmReady'
import './FileDialogs.css'

export interface NewEntryDialogProps {
  /** What is being named: a new file or a new folder. */
  kind: 'file' | 'folder'
  open: boolean
  onOpenChange: (open: boolean) => void
  /** The folder the new entry goes inside; `undefined` for the top level. */
  parent: string | undefined
  onCreate: (parent: string | undefined, name: string) => void
  /** Rust's reason creating this folder would be rejected, if it would. */
  createError: (parent: string | undefined, name: string) => string | undefined
}

/** Asks for the new file's or folder's name; nothing is created until it is
 * confirmed.
 * Rust explains live why a name is rejected. */
export function NewEntryDialog({
  kind,
  open,
  onOpenChange,
  parent,
  onCreate,
  createError,
}: NewEntryDialogProps) {
  const noun = kind === 'file' ? 'file' : 'folder'
  const [draft, setDraft] = useState('')
  const inputRef = useRef<HTMLInputElement>(null)

  useEffect(() => {
    if (!open) setDraft('')
  }, [open])

  const trimmed = draft.trim()
  // The name is checked in Rust, so it can't be confirmed before the wasm
  // component is ready.
  const wasmReady = useWasmReady()
  const error =
    open && trimmed && wasmReady ? createError(parent, trimmed) : undefined
  const canConfirm = trimmed !== '' && wasmReady && error === undefined

  const confirm = () => {
    if (!canConfirm) return
    onOpenChange(false)
    onCreate(parent, trimmed)
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
          }}
        >
          <div className="file-dialog-header">
            <Dialog.Title className="file-dialog-title">
              {kind === 'file' ? 'New file' : 'New folder'}
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
                {parent === undefined
                  ? `Name the new ${noun}.`
                  : `Name the new ${noun} inside ${parent}.`}
              </p>
              <input
                ref={inputRef}
                type="text"
                className="file-dialog-input"
                aria-label={kind === 'file' ? 'File name' : 'Folder name'}
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
                Create
              </button>
            </div>
          </form>
        </Dialog.Content>
      </Dialog.Portal>
    </Dialog.Root>
  )
}
