import { useEffect, useState } from 'react'
import { useAccountAuth } from '../storage/accountAuth'
import { changePassword } from '../storage/accountRequests'
import {
  AccountDialogFrame,
  AccountForm,
  ErrorBanner,
  Field,
  footnoteStyle,
  LockoutBanner,
  useLockoutCountdown,
} from './accountDialogParts'
import { SpinnerLabel } from './SpinnerLabel'
import { primaryButtonStyle } from './storageSettingsModalStyles'

export interface ChangePasswordModalProps {
  open: boolean
  onOpenChange: (open: boolean) => void
}

/**
 * Sets a new password for the signed-in account, from the header chip's
 * menu. The worker keeps this browser's session and ends the account's
 * others. A wrong current password counts toward the same lockout as a
 * wrong sign-in.
 */
export function ChangePasswordModal({
  open,
  onOpenChange,
}: ChangePasswordModalProps) {
  const [accountAuth, setAccountAuth] = useAccountAuth()
  const [current, setCurrent] = useState('')
  const [next, setNext] = useState('')
  const [confirm, setConfirm] = useState('')
  const [busy, setBusy] = useState(false)
  const [error, setError] = useState<string | null>(null)
  const [traceId, setTraceId] = useState<string | null>(null)
  const lockout = useLockoutCountdown()

  useEffect(() => {
    if (!open) return
    setCurrent('')
    setNext('')
    setConfirm('')
    setError(null)
    setTraceId(null)
  }, [open])

  async function submit() {
    if (!accountAuth || busy || lockout.seconds !== null) return
    if (next !== confirm) {
      setError("The new passwords don't match.")
      setTraceId(null)
      return
    }
    setBusy(true)
    const result = await changePassword(accountAuth.token, current, next)
    setBusy(false)
    if (result.ok) {
      onOpenChange(false)
      return
    }
    const { failure } = result
    setTraceId(result.traceId)
    switch (failure.kind) {
      case 'session-ended':
        setAccountAuth(null)
        onOpenChange(false)
        return
      case 'locked':
        setError(null)
        lockout.lock(failure.retryAfterSeconds)
        return
      case 'incorrect-credentials':
        setError('The current password is incorrect.')
        setCurrent('')
        return
      case 'message':
        setError(failure.message)
        return
      case 'email-taken':
        return
    }
  }

  const locked = lockout.seconds !== null

  return (
    <AccountDialogFrame
      open={open}
      onOpenChange={onOpenChange}
      title="Change password"
      testId="change-password-modal"
    >
      <AccountForm onSubmit={() => void submit()}>
        {lockout.seconds !== null ? (
          <LockoutBanner seconds={lockout.seconds} />
        ) : error ? (
          <ErrorBanner traceId={traceId}>{error}</ErrorBanner>
        ) : null}
        <Field
          id="change-password-current"
          label="Current password"
          type="password"
          value={current}
          onChange={setCurrent}
          autoComplete="current-password"
          disabled={locked}
        />
        <Field
          id="change-password-new"
          label="New password"
          type="password"
          value={next}
          onChange={setNext}
          autoComplete="new-password"
          hint="At least 12 characters."
        />
        <Field
          id="change-password-confirm"
          label="Confirm new password"
          type="password"
          value={confirm}
          onChange={setConfirm}
          autoComplete="new-password"
        />
        <button
          type="submit"
          style={primaryButtonStyle}
          disabled={busy || locked}
        >
          <SpinnerLabel pending={busy} label="Change password" />
        </button>
        <p style={footnoteStyle}>Your other devices will be signed out.</p>
      </AccountForm>
    </AccountDialogFrame>
  )
}
