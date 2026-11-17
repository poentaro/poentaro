import {
  callWorker,
  createWorkerClient,
  NetworkFailure,
  traceIdOf,
  WorkerRequestError,
  type WorkerSchemas,
} from '../syncedShare/workerClient'
import type { StoredAccountAuth } from './accountAuth'

/**
 * The account routes of the worker (`POST /auth/*`), with each failure
 * turned into what the account dialogs show. Every rule -- what counts as
 * an email, the password length, the lockout -- is the worker's; this only
 * words its answers.
 */

/** Why an account request was refused, as the dialog shows it. */
export type AccountFailure =
  /** Sign-up with an email that already has an account. */
  | { kind: 'email-taken' }
  /** Too many wrong passwords for this email; sign-in is refused until
   * `retryAfterSeconds` from now. */
  | { kind: 'locked'; retryAfterSeconds: number }
  /** A wrong password (or, at sign-in, an email with no account). */
  | { kind: 'incorrect-credentials' }
  /** The session behind a change password has ended. */
  | { kind: 'session-ended' }
  | { kind: 'message'; message: string }

/** `traceId` names the failed request in the worker's `app_log`, for the
 * dialog to show; `null` when it never reached the worker. */
export type AccountResult<T> =
  | { ok: true; value: T }
  | { ok: false; failure: AccountFailure; traceId: string | null }

function workerClient() {
  return createWorkerClient(import.meta.env.VITE_SYNCED_SHARE_HOST ?? '')
}

function toFailure(error: unknown): AccountFailure {
  if (error instanceof NetworkFailure) {
    return {
      kind: 'message',
      message: "Couldn't reach the server. Check your connection and retry.",
    }
  }
  const apiError =
    error instanceof WorkerRequestError ? error.apiError : undefined
  switch (apiError?.code) {
    case 'email_taken':
      return { kind: 'email-taken' }
    case 'sign_in_locked':
      return { kind: 'locked', retryAfterSeconds: apiError.retryAfterSeconds }
    case 'unauthorized':
      return { kind: 'session-ended' }
    case 'invalid_email':
      return { kind: 'message', message: 'Enter a valid email address.' }
    case 'weak_password':
      return { kind: 'message', message: apiError.message }
    case 'incorrect_credentials':
      return { kind: 'incorrect-credentials' }
    case 'rate_limited':
      return {
        kind: 'message',
        message: 'Too many attempts from this network. Wait a minute.',
      }
    default:
      return {
        kind: 'message',
        message: 'Something went wrong. Please try again.',
      }
  }
}

async function attempt<T>(send: () => Promise<T>): Promise<AccountResult<T>> {
  try {
    return { ok: true, value: await send() }
  } catch (error) {
    return { ok: false, failure: toFailure(error), traceId: traceIdOf(error) }
  }
}

async function startSession(
  path: '/auth/sign-up' | '/auth/sign-in',
  email: string,
  password: string,
): Promise<AccountResult<StoredAccountAuth>> {
  return attempt(async () => {
    const session = await callWorker(
      workerClient().POST(path, { body: { email, password } }),
    )
    return { token: session.sessionToken, email: session.email }
  })
}

export function signUp(email: string, password: string) {
  return startSession('/auth/sign-up', email, password)
}

export function signIn(email: string, password: string) {
  return startSession('/auth/sign-in', email, password)
}

/** Sets a new password. The worker keeps this session and ends the
 * account's others. */
export function changePassword(
  identityToken: string,
  currentPassword: string,
  newPassword: string,
): Promise<AccountResult<null>> {
  return attempt(async () => {
    await callWorker(
      workerClient().POST('/auth/change-password', {
        body: { identityToken, currentPassword, newPassword },
      }),
    )
    return null
  })
}

/** Ends the session on the worker. Fire-and-forget: the caller discards the
 * token either way, so a failed request only leaves the session to expire
 * on its own. */
export function signOutSession(identityToken: string): void {
  void workerClient()
    .POST('/auth/sign-out', { body: { identityToken } })
    .catch(() => {})
}

export type ApiKeySummary = WorkerSchemas['ApiKeySummary']
export type CreatedApiKey = WorkerSchemas['CreatedApiKey']

/** The account's API keys, newest first. */
export function listApiKeys(
  identityToken: string,
): Promise<AccountResult<ApiKeySummary[]>> {
  return attempt(async () => {
    const response = await callWorker(
      workerClient().POST('/api-keys/list', { body: { identityToken } }),
    )
    return response.keys
  })
}

/** A new API key, with the prompt that carries it: the only time either is
 * shown. */
export function createApiKey(
  identityToken: string,
): Promise<AccountResult<CreatedApiKey>> {
  return attempt(() =>
    callWorker(workerClient().POST('/api-keys', { body: { identityToken } })),
  )
}

/** Stops the key `id` working at once. */
export function revokeApiKey(
  identityToken: string,
  id: string,
): Promise<AccountResult<null>> {
  return attempt(async () => {
    await callWorker(
      workerClient().POST('/api-keys/{id}/revoke', {
        params: { path: { id } },
        body: { identityToken },
      }),
    )
    return null
  })
}
