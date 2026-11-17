import * as Dialog from '@radix-ui/react-dialog'
import { useEffect, useRef, useState } from 'react'
import './FileDialogs.css'

export interface NewPresetDialogProps {
  open: boolean
  onOpenChange: (open: boolean) => void
  onCreate: (name: string) => void
  /** Rust's reason the name can't be used, if it can't. */
  nameError: (name: string) => string | undefined
}

/** Names a new preset, which starts from the parts and notation shown now. */
export function NewPresetDialog({
  open,
  onOpenChange,
  onCreate,
  nameError,
}: NewPresetDialogProps) {
  const [draft, setDraft] = useState('')
  const inputRef = useRef<HTMLInputElement>(null)

  useEffect(() => {
    if (!open) setDraft('')
  }, [open])

  const trimmed = draft.trim()
  const error = open && trimmed ? nameError(trimmed) : undefined
  const canConfirm = trimmed !== '' && error === undefined

  const confirm = () => {
    if (!canConfirm) return
    onOpenChange(false)
    onCreate(trimmed)
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
              New preset
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
              <input
                ref={inputRef}
                type="text"
                className="file-dialog-input"
                aria-label="Preset name"
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
