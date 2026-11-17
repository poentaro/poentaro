import * as Dialog from '@radix-ui/react-dialog'
import { type ReactNode, useCallback, useEffect, useState } from 'react'
import {
  bannerStyle,
  bodyStyle,
  contentStyle,
  headerStyle,
  overlayStyle,
} from './storageSettingsModalStyles'
import { TraceId } from './TraceId'

/**
 * The pieces `AccountAuthModal` and `ChangePasswordModal` share: the dialog
 * frame (the Storage dialog's look), a labelled field, the error and
 * lockout banners, and the lockout countdown.
 */

const labelStyle: React.CSSProperties = { fontSize: '12px', color: '#444' }

const inputStyle: React.CSSProperties = {
  height: '32px',
  boxSizing: 'border-box',
  padding: '0 8px',
  border: '1px solid #cbd5e0',
  borderRadius: '4px',
  font: 'inherit',
  fontSize: '13px',
  color: '#1a1a1a',
}

const hintStyle: React.CSSProperties = { fontSize: '11px', color: '#555' }

export const footnoteStyle: React.CSSProperties = {
  margin: 0,
  fontSize: '12px',
  color: '#555',
}

/** A button that reads as an inline link ("Create one", "Sign in"). */
export const linkButtonStyle: React.CSSProperties = {
  border: 'none',
  background: 'none',
  padding: 0,
  font: 'inherit',
  color: '#1d4ed8',
  textDecoration: 'underline',
  cursor: 'pointer',
}

const errorBannerStyle: React.CSSProperties = {
  ...bannerStyle,
  background: '#fef2f2',
  border: '1px solid #fecaca',
  color: '#b91c1c',
}

export function AccountDialogFrame({
  open,
  onOpenChange,
  title,
  testId,
  children,
}: {
  open: boolean
  onOpenChange: (open: boolean) => void
  title: string
  testId: string
  children: ReactNode
}) {
  return (
    <Dialog.Root open={open} onOpenChange={onOpenChange}>
      <Dialog.Portal>
        <Dialog.Overlay style={overlayStyle} />
        <Dialog.Content
          data-testid={testId}
          style={{ ...contentStyle, width: '420px' }}
          aria-describedby={undefined}
        >
          <div style={headerStyle}>
            <Dialog.Title
              style={{ margin: 0, fontSize: '14px', fontWeight: 600 }}
            >
              {title}
            </Dialog.Title>
            <Dialog.Close
              aria-label="Close"
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
          {children}
        </Dialog.Content>
      </Dialog.Portal>
    </Dialog.Root>
  )
}

/** The dialog body as a form, so Enter submits it. */
export function AccountForm({
  onSubmit,
  children,
}: {
  onSubmit: () => void
  children: ReactNode
}) {
  return (
    <form
      style={bodyStyle}
      noValidate
      onSubmit={(event) => {
        event.preventDefault()
        onSubmit()
      }}
    >
      {children}
    </form>
  )
}

export function Field({
  id,
  label,
  type,
  value,
  onChange,
  autoComplete,
  hint,
  disabled = false,
}: {
  id: string
  label: string
  type: 'email' | 'password'
  value: string
  onChange: (value: string) => void
  autoComplete: string
  hint?: string
  disabled?: boolean
}) {
  return (
    <div style={{ display: 'flex', flexDirection: 'column', gap: '4px' }}>
      <label htmlFor={id} style={labelStyle}>
        {label}
      </label>
      <input
        id={id}
        type={type}
        value={value}
        autoComplete={autoComplete}
        disabled={disabled}
        style={inputStyle}
        onChange={(event) => onChange(event.currentTarget.value)}
      />
      {hint ? <span style={hintStyle}>{hint}</span> : null}
    </div>
  )
}

/** `traceId`, when given, is the failed request's (see `TraceId`). */
export function ErrorBanner({
  children,
  traceId,
}: {
  children: ReactNode
  traceId?: string | null
}) {
  return (
    <div role="alert" style={errorBannerStyle}>
      {children}
      <TraceId traceId={traceId} />
    </div>
  )
}

/** `m:ss`, e.g. `14:32`. */
function formatCountdown(seconds: number): string {
  const minutes = Math.floor(seconds / 60)
  return `${minutes}:${String(seconds % 60).padStart(2, '0')}`
}

export function LockoutBanner({ seconds }: { seconds: number }) {
  return (
    <div role="alert" style={bannerStyle}>
      Too many attempts for this email. Try again in {formatCountdown(seconds)}.
    </div>
  )
}

/** Counts down the worker's `retryAfterSeconds` once a second. `seconds` is
 * `null` while not locked, and goes back to `null` when the pause ends. */
export function useLockoutCountdown() {
  const [lockedUntil, setLockedUntil] = useState<number | null>(null)
  const [now, setNow] = useState(() => Date.now())

  useEffect(() => {
    if (lockedUntil === null) return
    const timer = window.setInterval(() => setNow(Date.now()), 1000)
    return () => window.clearInterval(timer)
  }, [lockedUntil])

  const remaining =
    lockedUntil === null ? 0 : Math.ceil((lockedUntil - now) / 1000)

  useEffect(() => {
    if (lockedUntil !== null && remaining <= 0) setLockedUntil(null)
  }, [lockedUntil, remaining])

  const lock = useCallback((retryAfterSeconds: number) => {
    const at = Date.now()
    setNow(at)
    setLockedUntil(at + retryAfterSeconds * 1000)
  }, [])
  const clear = useCallback(() => setLockedUntil(null), [])

  return { seconds: remaining > 0 ? remaining : null, lock, clear }
}
