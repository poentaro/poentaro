import { useEffect, useState } from 'react'
import { useAccountAuth } from '../storage/accountAuth'
import { type AccountFailure, signIn, signUp } from '../storage/accountRequests'
import {
  AccountDialogFrame,
  AccountForm,
  ErrorBanner,
  Field,
  footnoteStyle,
  LockoutBanner,
  linkButtonStyle,
  useLockoutCountdown,
} from './accountDialogParts'
import { SpinnerLabel } from './SpinnerLabel'
import { bannerStyle, primaryButtonStyle } from './storageSettingsModalStyles'

type Mode = 'sign-in' | 'sign-up'

/** What the dialog shows above the fields after a refused request. */
type ShownFailure =
  | { kind: 'email-taken' }
  | { kind: 'message'; message: string }

export interface AccountAuthModalProps {
  open: boolean
  onOpenChange: (open: boolean) => void
}

/**
 * Sign in, or create an account, with an email and a password. Opened from
 * the header's "Sign in" button and the sign-in shortcuts of the share and
 * storage dialogs. On success it stores the worker's session
 * (`accountAuth.ts`) and closes; everything that reads the session
 * re-renders from there. The worker decides every refusal (taken email,
 * short password, lockout); this only shows them.
 */
export function AccountAuthModal({
  open,
  onOpenChange,
}: AccountAuthModalProps) {
  const [, setAccountAuth] = useAccountAuth()
  const [mode, setMode] = useState<Mode>('sign-in')
  const [email, setEmail] = useState('')
  const [password, setPassword] = useState('')
  const [confirm, setConfirm] = useState('')
  const [busy, setBusy] = useState(false)
  const [failure, setFailure] = useState<ShownFailure | null>(null)
  const [traceId, setTraceId] = useState<string | null>(null)
  const [hadWrongPassword, setHadWrongPassword] = useState(false)
  const lockout = useLockoutCountdown()

  useEffect(() => {
    if (!open) return
    setMode('sign-in')
    setEmail('')
    setPassword('')
    setConfirm('')
    setFailure(null)
    setTraceId(null)
    setHadWrongPassword(false)
  }, [open])

  function switchTo(next: Mode) {
    setMode(next)
    setPassword('')
    setConfirm('')
    setFailure(null)
    setTraceId(null)
    setHadWrongPassword(false)
  }

  function changeEmail(next: string) {
    setEmail(next)
    // A lockout is per email: a different one may well be allowed.
    lockout.clear()
  }

  function showFailure(refused: AccountFailure) {
    if (refused.kind === 'locked') {
      setFailure(null)
      setPassword('')
      lockout.lock(refused.retryAfterSeconds)
    } else if (refused.kind === 'email-taken') {
      setFailure({ kind: 'email-taken' })
    } else if (refused.kind === 'incorrect-credentials') {
      setFailure({ kind: 'message', message: 'Incorrect email or password.' })
      setHadWrongPassword(true)
      setPassword('')
    } else if (refused.kind === 'message') {
      setFailure(refused)
    }
  }

  async function submit() {
    if (busy || lockout.seconds !== null) return
    if (mode === 'sign-up' && password !== confirm) {
      setFailure({ kind: 'message', message: "The passwords don't match." })
      setTraceId(null)
      return
    }
    setBusy(true)
    const result =
      mode === 'sign-up'
        ? await signUp(email, password)
        : await signIn(email, password)
    setBusy(false)
    if (result.ok) {
      setAccountAuth(result.value)
      onOpenChange(false)
      return
    }
    showFailure(result.failure)
    setTraceId(result.traceId)
  }

  const locked = mode === 'sign-in' && lockout.seconds !== null

  return (
    <AccountDialogFrame
      open={open}
      onOpenChange={onOpenChange}
      title={mode === 'sign-up' ? 'Create account' : 'Sign in'}
      testId="account-auth-modal"
    >
      <AccountForm onSubmit={() => void submit()}>
        {locked && lockout.seconds !== null ? (
          <LockoutBanner seconds={lockout.seconds} />
        ) : failure?.kind === 'email-taken' ? (
          <ErrorBanner traceId={traceId}>
            An account with this email already exists.{' '}
            <button
              type="button"
              style={linkButtonStyle}
              onClick={() => switchTo('sign-in')}
            >
              Sign in instead
            </button>
          </ErrorBanner>
        ) : failure ? (
          <ErrorBanner traceId={traceId}>{failure.message}</ErrorBanner>
        ) : null}
        <Field
          id="account-email"
          label="Email"
          type="email"
          value={email}
          onChange={changeEmail}
          autoComplete="email"
        />
        <Field
          id="account-password"
          label="Password"
          type="password"
          value={password}
          onChange={setPassword}
          autoComplete={
            mode === 'sign-up' ? 'new-password' : 'current-password'
          }
          disabled={locked}
          hint={
            mode === 'sign-up'
              ? 'At least 12 characters.'
              : hadWrongPassword && !locked
                ? 'After 5 wrong attempts, sign-in pauses for 15 minutes.'
                : undefined
          }
        />
        {mode === 'sign-up' ? (
          <>
            <Field
              id="account-confirm-password"
              label="Confirm password"
              type="password"
              value={confirm}
              onChange={setConfirm}
              autoComplete="new-password"
            />
            <div style={bannerStyle}>
              We don't verify emails yet, and there's no password reset by
              email. Keep your password somewhere safe.
            </div>
          </>
        ) : null}
        <button
          type="submit"
          style={primaryButtonStyle}
          disabled={busy || locked}
        >
          <SpinnerLabel
            pending={busy}
            label={mode === 'sign-up' ? 'Create account' : 'Sign in'}
          />
        </button>
        {mode === 'sign-up' ? (
          <p style={footnoteStyle}>
            Already have an account?{' '}
            <button
              type="button"
              style={linkButtonStyle}
              onClick={() => switchTo('sign-in')}
            >
              Sign in
            </button>
          </p>
        ) : locked ? (
          <p style={footnoteStyle}>
            The pause doubles each time the limit is hit again. The admin can
            clear it early.
          </p>
        ) : (
          <>
            <p style={footnoteStyle}>
              No account?{' '}
              <button
                type="button"
                style={linkButtonStyle}
                onClick={() => switchTo('sign-up')}
              >
                Create one
              </button>
            </p>
            <p style={footnoteStyle}>
              Forgot your password? Ask the admin to reset it.
            </p>
          </>
        )}
      </AccountForm>
    </AccountDialogFrame>
  )
}
