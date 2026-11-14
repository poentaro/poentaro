import type { KeysCreatedKey, KeysKey } from '../generated/protocol/app'
import {
  appClient,
  type ClientOptions,
  publicClient,
} from '../protocol/clients'
import { type CallOutcome, expectValue, settled } from '../protocol/outcome'
import { SessionEnded } from '../protocol/transport'
import { NetworkFailure, traceIdOf } from '../syncedShare/workerClient'
import type { StoredAccountAuth } from './accountAuth'

/**
 * The account methods of `poentaro public` and `poentaro app`, with each
 * failure turned into what the account dialogs show. Every rule -- what
 * counts as an email, the password length, the lockout -- is the worker's;
 * this only words its answers.
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

/** Every error an account method can answer, by `kind`. */
type AccountError =
  | { kind: 'emailTaken' }
  | { kind: 'weakPassword'; message: string }
  | { kind: 'invalidEmail' }
  | { kind: 'tooManyRequests' }
  | { kind: 'incorrect' }
  | { kind: 'incorrectPassword' }
  | { kind: 'lockedFor'; seconds: number }

const SOMETHING_WENT_WRONG: AccountFailure = {
  kind: 'message',
  message: 'Something went wrong. Please try again.',
}

function errorFailure(error: AccountError): AccountFailure {
  switch (error.kind) {
    case 'emailTaken':
      return { kind: 'email-taken' }
    case 'lockedFor':
      return { kind: 'locked', retryAfterSeconds: error.seconds }
    case 'invalidEmail':
      return { kind: 'message', message: 'Enter a valid email address.' }
    case 'weakPassword':
      return { kind: 'message', message: error.message }
    case 'incorrect':
    case 'incorrectPassword':
      return { kind: 'incorrect-credentials' }
    case 'tooManyRequests':
      return {
        kind: 'message',
        message: 'Too many attempts from this network. Wait a minute.',
      }
  }
}

function thrownFailure(error: unknown): AccountFailure {
  if (error instanceof NetworkFailure) {
    return {
      kind: 'message',
      message: "Couldn't reach the server. Check your connection and retry.",
    }
  }
  if (error instanceof SessionEnded) return { kind: 'session-ended' }
  return SOMETHING_WENT_WRONG
}

/** Runs `call` with client options that record its response's trace ID,
 * turning its method's errors and anything thrown into what the dialogs
 * show. */
async function attempt<T, E extends AccountError>(
  call: (options: ClientOptions) => Promise<CallOutcome<T, E>>,
): Promise<AccountResult<T>> {
  let traceId: string | null = null
  const options: ClientOptions = {
    onTraceId: (id) => {
      traceId = id
    },
  }
  try {
    const outcome = settled(await call(options))
    return outcome.kind === 'value'
      ? { ok: true, value: outcome.value }
      : { ok: false, failure: errorFailure(outcome.error), traceId }
  } catch (error) {
    return {
      ok: false,
      failure: thrownFailure(error),
      traceId: traceIdOf(error) ?? traceId,
    }
  }
}

function stored(session: { token: string; email: string }): StoredAccountAuth {
  return { token: session.token, email: session.email }
}

export async function signUp(
  email: string,
  password: string,
): Promise<AccountResult<StoredAccountAuth>> {
  const result = await attempt(async (options) =>
    (await publicClient(options)).account.signUpPassword(email, password),
  )
  return result.ok ? { ok: true, value: stored(result.value) } : result
}

export async function signIn(
  email: string,
  password: string,
): Promise<AccountResult<StoredAccountAuth>> {
  const result = await attempt(async (options) =>
    (await publicClient(options)).account.signInPassword(email, password),
  )
  return result.ok ? { ok: true, value: stored(result.value) } : result
}

/** Sets a new password. The worker keeps this session and ends the
 * account's others. */
export async function changePassword(
  identityToken: string,
  currentPassword: string,
  newPassword: string,
): Promise<AccountResult<null>> {
  const result = await attempt(async (options) =>
    (await appClient(identityToken, options)).account.changePasswordFromTo(
      currentPassword,
      newPassword,
    ),
  )
  return result.ok ? { ok: true, value: null } : result
}

/** Ends the session on the worker. Fire-and-forget: the caller discards the
 * token either way, so a failed request only leaves the session to expire
 * on its own. */
export function signOutSession(identityToken: string): void {
  void appClient(identityToken)
    .then((client) => expectValue(client.account.signOut()))
    .catch(() => {})
}

export type ApiKeySummary = KeysKey
export type CreatedApiKey = KeysCreatedKey

/** Runs a call whose method has no errors of its own. */
async function attemptValue<T>(
  call: () => Promise<T>,
): Promise<AccountResult<T>> {
  try {
    return { ok: true, value: await call() }
  } catch (error) {
    return {
      ok: false,
      failure: thrownFailure(error),
      traceId: traceIdOf(error),
    }
  }
}

/** The account's API keys, newest first. */
export function listApiKeys(
  identityToken: string,
): Promise<AccountResult<ApiKeySummary[]>> {
  return attemptValue(async () =>
    expectValue((await appClient(identityToken)).keys.list()),
  )
}

/** A new API key, with the prompt that carries it: the only time either is
 * shown. */
export function createApiKey(
  identityToken: string,
): Promise<AccountResult<CreatedApiKey>> {
  return attemptValue(async () =>
    expectValue((await appClient(identityToken)).keys.create()),
  )
}

/** Stops the key `id` working at once. A key that is already gone counts as
 * revoked. */
export function revokeApiKey(
  identityToken: string,
  id: string,
): Promise<AccountResult<null>> {
  return attemptValue(async () => {
    settled(await (await appClient(identityToken)).keys.revoke(id))
    return null
  })
}
