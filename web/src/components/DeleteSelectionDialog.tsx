import * as Dialog from '@radix-ui/react-dialog'
import type { SelectionDeleteSummary } from '../poentaroWasm'
import './FileDialogs.css'

export interface DeleteSelectionDialogProps {
  open: boolean
  onOpenChange: (open: boolean) => void
  /** What deleting the selection does, from Rust; `undefined` while closed. */
  summary: SelectionDeleteSummary | undefined
  onConfirm: () => void
}

/** Asks before the selected files and folders move to the bin, listing each
 * of them. */
export function DeleteSelectionDialog({
  open,
  onOpenChange,
  summary,
  onConfirm,
}: DeleteSelectionDialogProps) {
  return (
    <Dialog.Root open={open} onOpenChange={onOpenChange}>
      <Dialog.Portal>
        <Dialog.Overlay className="file-dialog-overlay" />
        <Dialog.Content className="file-dialog" aria-describedby={undefined}>
          <div className="file-dialog-header">
            <Dialog.Title className="file-dialog-title">
              Delete selected?
            </Dialog.Title>
            <Dialog.Close className="file-dialog-close" aria-label="Close">
              ×
            </Dialog.Close>
          </div>
          <div className="file-dialog-body">
            <p className="file-dialog-warn-box">{summary?.message}</p>
            <ul className="file-dialog-list" aria-label="Items to delete">
              {summary?.lines.map((line) => (
                <li key={line}>{line}</li>
              ))}
            </ul>
            {summary?.shareWarning ? (
              <p
                className="file-dialog-warn-box"
                data-testid="delete-selection-share-warning"
              >
                {summary.shareWarning}
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
              className="file-dialog-button file-dialog-button--danger"
              onClick={() => {
                onOpenChange(false)
                onConfirm()
              }}
            >
              Move to bin
            </button>
          </div>
        </Dialog.Content>
      </Dialog.Portal>
    </Dialog.Root>
  )
}
