import * as Dialog from '@radix-ui/react-dialog'
import './FileDialogs.css'

export interface RevokeApiKeyDialogProps {
  /** The prefix of the key to revoke, or `null` when closed. */
  prefix: string | null
  onCancel: () => void
  onConfirm: () => void
}

/** Asks before an API key stops working. */
export function RevokeApiKeyDialog({
  prefix,
  onCancel,
  onConfirm,
}: RevokeApiKeyDialogProps) {
  return (
    <Dialog.Root
      open={prefix !== null}
      onOpenChange={(open) => {
        if (!open) onCancel()
      }}
    >
      <Dialog.Portal>
        <Dialog.Overlay className="file-dialog-overlay" />
        <Dialog.Content
          className="file-dialog"
          aria-describedby={undefined}
          data-testid="revoke-api-key-dialog"
        >
          <div className="file-dialog-header">
            <Dialog.Title className="file-dialog-title">
              {`Revoke key ${prefix ?? ''}…?`}
            </Dialog.Title>
            <Dialog.Close className="file-dialog-close" aria-label="Close">
              ×
            </Dialog.Close>
          </div>
          <div className="file-dialog-body">
            <p className="file-dialog-warn-box">
              Any AI agent using it stops working at once.
            </p>
          </div>
          <div className="file-dialog-footer">
            <button
              type="button"
              className="file-dialog-button"
              onClick={onCancel}
            >
              Cancel
            </button>
            <button
              type="button"
              className="file-dialog-button file-dialog-button--danger"
              onClick={onConfirm}
            >
              Revoke
            </button>
          </div>
        </Dialog.Content>
      </Dialog.Portal>
    </Dialog.Root>
  )
}
