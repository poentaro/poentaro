import * as Dialog from '@radix-ui/react-dialog'
import type { SyncedShareFailure } from '../syncedShare/errors'
import { redactSecrets } from '../syncedShare/errors'
import { TraceId } from './TraceId'

const NEW_ISSUE_URL = 'https://github.com/poentaro/poentaro/issues/new'

export interface SyncedShareErrorDialogProps {
  /** Non-null shows the dialog; `null` keeps it closed. Task 9's decision:
   * a full-screen dialog (not a toast/inline banner), dismissible, with no
   * automatic retry -- dismissing just returns to idle state. */
  failure: SyncedShareFailure | null
  onDismiss: () => void
}

// Full-screen per §0/task 9 ("a full-screen dialog/modal (not a toast or
// inline banner)") -- deliberately not the centered-box layout `ErrorModal`
// uses, since this needs to hold a lot of verbose, screenshot-friendly
// context.
const overlayStyle: React.CSSProperties = {
  position: 'fixed',
  inset: 0,
  background: 'rgba(0,0,0,0.5)',
  zIndex: 2000,
}

const contentStyle: React.CSSProperties = {
  position: 'fixed',
  inset: 0,
  width: '100vw',
  height: '100vh',
  background: '#fff',
  color: '#1a1a1a',
  zIndex: 2001,
  display: 'flex',
  flexDirection: 'column',
  fontFamily: 'var(--mono, monospace)',
}

const headerStyle: React.CSSProperties = {
  display: 'flex',
  alignItems: 'center',
  justifyContent: 'space-between',
  padding: '20px 24px',
  borderBottom: '1px solid #eee',
  flexShrink: 0,
}

const bodyStyle: React.CSSProperties = {
  overflowY: 'auto',
  flex: 1,
  padding: '24px',
  fontSize: '13px',
  display: 'flex',
  flexDirection: 'column',
  gap: '16px',
  maxWidth: '900px',
}

const fieldRowStyle: React.CSSProperties = {
  display: 'flex',
  gap: '8px',
  alignItems: 'baseline',
}

const fieldLabelStyle: React.CSSProperties = {
  fontWeight: 600,
  minWidth: '110px',
  flexShrink: 0,
  color: '#555',
}

const preStyle: React.CSSProperties = {
  margin: 0,
  padding: '10px 12px',
  borderRadius: '4px',
  background: '#f5f5f5',
  border: '1px solid #eee',
  fontSize: '12px',
  whiteSpace: 'pre-wrap',
  wordBreak: 'break-word',
  color: '#333',
}

function formatTimestamp(millis: number): string {
  try {
    return `${new Date(millis).toISOString()} (${millis}ms since epoch)`
  } catch {
    return `${millis}ms since epoch`
  }
}

/**
 * Full-screen verbose failure dialog (mockup Screen 4, task 9 in
 * `TODO-synced-share-rust-d1-migration.md`). Shows the full structured
 * context `useSyncedShareOwner.ts` captured for a failed write -- reason,
 * timestamp, attempt count, raw HTTP status/body -- deliberately verbose
 * (per §0) so a screenshot of this dialog is directly useful for debugging.
 *
 * The one hard exception: the session token and its stored hash must
 * never appear here. The primary guarantee is upstream (the worker's
 * `VerificationFailure` response has no such field, see
 * `auth/session.rs`); every string rendered below additionally passes
 * through `redactSecrets` as a defensive backstop, in case some other
 * failure path ever carried one.
 */
export function SyncedShareErrorDialog({
  failure,
  onDismiss,
}: SyncedShareErrorDialogProps) {
  return (
    <Dialog.Root
      open={failure !== null}
      onOpenChange={(open) => {
        if (!open) onDismiss()
      }}
    >
      <Dialog.Portal>
        <Dialog.Overlay style={overlayStyle} />
        <Dialog.Content
          data-testid="synced-share-error-dialog"
          style={contentStyle}
          aria-describedby={undefined}
        >
          <div style={headerStyle}>
            <Dialog.Title
              style={{
                margin: 0,
                fontSize: '18px',
                fontWeight: 600,
                color: '#b00020',
              }}
            >
              Synced Share failed to sync
            </Dialog.Title>
            <Dialog.Close
              data-testid="synced-share-error-dialog-close"
              style={{
                background: 'none',
                border: 'none',
                cursor: 'pointer',
                fontSize: '20px',
                color: '#666',
                lineHeight: 1,
                padding: '4px 8px',
              }}
              aria-label="Close"
            >
              ×
            </Dialog.Close>
          </div>
          {failure && (
            <div style={bodyStyle}>
              <p style={{ margin: 0 }}>
                This change could not be synced. Take a screenshot of this
                dialog and{' '}
                <a
                  href={NEW_ISSUE_URL}
                  target="_blank"
                  rel="noreferrer noopener"
                  data-testid="synced-share-error-dialog-issue-link"
                >
                  file a GitHub issue
                </a>{' '}
                with it attached.
              </p>
              <div
                data-testid="synced-share-error-dialog-details"
                style={{ display: 'flex', flexDirection: 'column', gap: '6px' }}
              >
                <div style={fieldRowStyle}>
                  <span style={fieldLabelStyle}>Operation</span>
                  <span>{redactSecrets(failure.operation)}</span>
                </div>
                <div style={fieldRowStyle}>
                  <span style={fieldLabelStyle}>Reason</span>
                  <span>{redactSecrets(failure.reason)}</span>
                </div>
                <div style={fieldRowStyle}>
                  <span style={fieldLabelStyle}>Failed at</span>
                  <span>{formatTimestamp(failure.failedAt)}</span>
                </div>
                {failure.attempts !== undefined && (
                  <div style={fieldRowStyle}>
                    <span style={fieldLabelStyle}>Attempts</span>
                    <span>{failure.attempts}</span>
                  </div>
                )}
                {failure.httpStatus !== undefined && (
                  <div style={fieldRowStyle}>
                    <span style={fieldLabelStyle}>HTTP status</span>
                    <span>
                      {failure.httpStatus}
                      {failure.httpStatusText
                        ? ` ${redactSecrets(failure.httpStatusText)}`
                        : ''}
                    </span>
                  </div>
                )}
                <TraceId traceId={failure.traceId} />
              </div>
              {failure.rawResponseBody && (
                <div>
                  <div style={{ ...fieldLabelStyle, marginBottom: '4px' }}>
                    Raw response body
                  </div>
                  <pre
                    style={preStyle}
                    data-testid="synced-share-error-dialog-raw-body"
                  >
                    {redactSecrets(failure.rawResponseBody)}
                  </pre>
                </div>
              )}
            </div>
          )}
        </Dialog.Content>
      </Dialog.Portal>
    </Dialog.Root>
  )
}
