import * as Dialog from '@radix-ui/react-dialog'
import { useEffect, useRef, useState } from 'react'
import type { PendingDownload } from '../hooks/usePoentaroWorkerTypes'

export interface DownloadRenameModalProps {
  pending: PendingDownload | null
  onConfirm: (filename: string) => void
  onCancel: () => void
}

const overlayStyle: React.CSSProperties = {
  position: 'fixed',
  inset: 0,
  background: 'rgba(0,0,0,0.35)',
  zIndex: 1000,
}

const contentStyle: React.CSSProperties = {
  position: 'fixed',
  top: '50%',
  left: '50%',
  transform: 'translate(-50%, -50%)',
  background: '#fff',
  border: '1px solid #ddd',
  borderRadius: '6px',
  boxShadow: '0 8px 32px rgba(0,0,0,0.16)',
  zIndex: 1001,
  minWidth: '360px',
  maxWidth: '90vw',
  fontFamily: 'var(--mono, monospace)',
}

const headerStyle: React.CSSProperties = {
  display: 'flex',
  alignItems: 'center',
  justifyContent: 'space-between',
  padding: '12px 16px',
  borderBottom: '1px solid #eee',
}

const bodyStyle: React.CSSProperties = {
  padding: '16px',
  fontSize: '13px',
  display: 'flex',
  flexDirection: 'column',
  gap: '8px',
}

const inputStyle: React.CSSProperties = {
  fontFamily: 'inherit',
  fontSize: '13px',
  padding: '6px 8px',
  border: '1px solid #ccc',
  borderRadius: '4px',
}

const footerStyle: React.CSSProperties = {
  display: 'flex',
  justifyContent: 'flex-end',
  gap: '8px',
  marginTop: '4px',
}

const errorStyle: React.CSSProperties = {
  color: '#b00020',
  fontSize: '12px',
  margin: 0,
}

const buttonStyle: React.CSSProperties = {
  display: 'inline-flex',
  alignItems: 'center',
  gap: '6px',
  padding: '6px 12px',
  fontSize: '13px',
  borderRadius: '4px',
  cursor: 'pointer',
}

const cancelButtonStyle: React.CSSProperties = {
  ...buttonStyle,
  border: '1px solid #ccc',
  background: '#fff',
}

const confirmButtonStyle: React.CSSProperties = {
  ...buttonStyle,
  border: '1px solid #2563eb',
  background: '#2563eb',
  color: '#fff',
}

const keyHintStyle: React.CSSProperties = {
  fontFamily: 'var(--mono, monospace)',
  fontSize: '11px',
  lineHeight: 1,
  padding: '2px 5px',
  borderRadius: '3px',
  background: 'rgba(0,0,0,0.06)',
  color: '#666',
}

const confirmKeyHintStyle: React.CSSProperties = {
  ...keyHintStyle,
  background: 'rgba(255,255,255,0.22)',
  color: '#fff',
}

/** Base-name length of `filename` (everything before its final `.ext`) —
 * used to preselect just the base name on open, matching OS Save-As
 * convention so typing replaces the name but leaves the extension intact.
 * A filename with no extension selects the whole string. */
function baseNameLength(filename: string): number {
  const dotIndex = filename.lastIndexOf('.')
  return dotIndex === -1 ? filename.length : dotIndex
}

/** Modal interposed before every export download fires (PDF, MIDI, the ZIP
 * bundles, and the WAV/MP3 preview player's download button) — see
 * `PendingDownload` in `usePoentaroWorkerTypes.ts`. Pre-fills the computed
 * filename with just its base name selected; Enter or the "Download"
 * button confirms, Escape/overlay/Cancel aborts with no download. */
export function DownloadRenameModal({
  pending,
  onConfirm,
  onCancel,
}: DownloadRenameModalProps) {
  const [name, setName] = useState('')
  const [error, setError] = useState<string | null>(null)
  const inputRef = useRef<HTMLInputElement>(null)
  const openedAtRef = useRef(0)
  const lastMouseDownAtRef = useRef(-Infinity)

  useEffect(() => {
    if (!pending) return
    setName(pending.filename)
    setError(null)
    // Some export downloads (e.g. the ZIP bundles) can resolve fast enough
    // that this modal mounts *while the export menu item's own click is
    // still being dispatched* by the browser: the menu item disappears
    // mid-click, and the same physical click's follow-through lands on
    // whatever now occupies that screen position — which can be this
    // modal's own "Download" button, silently firing the download before
    // the user ever saw the dialog. `openedAtRef` records when that became
    // possible; the click-capture guard below rejects any click whose
    // mousedown predates it, since a genuine click on this dialog's content
    // can only start (mousedown) after the dialog exists.
    openedAtRef.current = performance.now()
    // Wait a tick for the input to mount/render before focusing/selecting.
    const id = window.setTimeout(() => {
      const input = inputRef.current
      if (!input) return
      input.focus()
      input.setSelectionRange(0, baseNameLength(pending.filename))
    }, 0)
    return () => window.clearTimeout(id)
  }, [pending])

  useEffect(() => {
    // Both registered once, at the window, in the capture phase — the
    // mousedown listener runs regardless of whether the dialog is open (it
    // needs to see the export menu item's mousedown too), and the click
    // listener runs before Radix's own listeners and React's synthetic
    // event dispatch, so a rejected click never reaches the dialog's
    // buttons or triggers a form submit.
    const onMouseDownCapture = (event: MouseEvent) => {
      lastMouseDownAtRef.current = event.timeStamp
    }
    const onClickCapture = (event: MouseEvent) => {
      // `detail === 0` means this click was synthesized from a keyboard
      // activation (Enter/Space on a focused button, or "Enter submits the
      // form" on the input) rather than a real pointer press — never part
      // of a stale mouse gesture, so always let it through.
      if (event.detail === 0) return
      if (lastMouseDownAtRef.current >= openedAtRef.current) return
      event.preventDefault()
      event.stopPropagation()
    }
    window.addEventListener('mousedown', onMouseDownCapture, true)
    window.addEventListener('click', onClickCapture, true)
    return () => {
      window.removeEventListener('mousedown', onMouseDownCapture, true)
      window.removeEventListener('click', onClickCapture, true)
    }
  }, [])

  function submit() {
    const trimmed = name.trim()
    if (trimmed === '') {
      setError('Filename cannot be empty.')
      return
    }
    if (trimmed.includes('/') || trimmed.includes('\\')) {
      setError('Filename cannot contain "/" or "\\".')
      return
    }
    onConfirm(trimmed)
  }

  return (
    <Dialog.Root
      open={pending !== null}
      onOpenChange={(open) => {
        if (!open) onCancel()
      }}
    >
      <Dialog.Portal>
        <Dialog.Overlay style={overlayStyle} />
        <Dialog.Content
          data-testid="download-rename-modal"
          style={contentStyle}
        >
          <div style={headerStyle}>
            <Dialog.Title
              style={{ margin: 0, fontSize: '14px', fontWeight: 600 }}
            >
              Download as…
            </Dialog.Title>
            <Dialog.Close
              style={{
                background: 'none',
                border: 'none',
                cursor: 'pointer',
                fontSize: '16px',
                color: '#666',
                lineHeight: 1,
                padding: '2px 4px',
              }}
            >
              ×
            </Dialog.Close>
          </div>
          <form
            style={bodyStyle}
            onSubmit={(e) => {
              e.preventDefault()
              submit()
            }}
          >
            <input
              ref={inputRef}
              type="text"
              data-testid="download-rename-input"
              style={inputStyle}
              value={name}
              onChange={(e) => {
                setName(e.target.value)
                setError(null)
              }}
            />
            {error ? (
              <p data-testid="download-rename-error" style={errorStyle}>
                {error}
              </p>
            ) : null}
            <div style={footerStyle}>
              <button
                type="button"
                data-testid="download-rename-cancel"
                style={cancelButtonStyle}
                onClick={onCancel}
              >
                Cancel
                <span style={keyHintStyle}>Esc</span>
              </button>
              <button
                type="submit"
                data-testid="download-rename-confirm"
                style={confirmButtonStyle}
              >
                Download
                <span style={confirmKeyHintStyle}>Enter</span>
              </button>
            </div>
          </form>
        </Dialog.Content>
      </Dialog.Portal>
    </Dialog.Root>
  )
}
