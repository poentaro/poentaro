// Structured failure info for a Synced Share request (owner side), surfaced by
// `useSyncedShareOwner.ts` and rendered by `SyncedShareErrorDialog.tsx`.
// Implements task 9 in TODO-synced-share-rust-d1-migration.md: errors here
// are meant to be maximally verbose (per §0) EXCEPT for one hard exception --
// the session token and its stored hash must never appear. The worker's
// `unauthorized` `ApiError` (a `VerificationFailure`, `crates/live-share-worker/src/auth/session.rs`)
// has no field for either by construction, so the common case is already
// safe by the time it reaches here; `redactSecrets` below is a defensive
// backstop for any other failure shape (a raw non-2xx body, a thrown network
// error) that might somehow echo one back.

import { WorkerRequestError } from './workerClient'

export type SyncedShareOperation = 'start' | 'stop' | 'status'

export interface SyncedShareFailure {
  operation: SyncedShareOperation
  reason: string
  /** Milliseconds since the epoch. From the worker's `VerificationFailure`
   * when available, otherwise the time this client observed the failure. */
  failedAt: number
  /** Present only for the worker's `unauthorized` `ApiError`. */
  attempts?: number
  /** The worker rejected the identity token itself (`unauthorized`), so
   * resending it can never succeed. */
  authRejected: boolean
  httpStatus?: number
  httpStatusText?: string
  /** The worker's trace ID for this request; absent when the request never
   * reached the worker. */
  traceId?: string
  /** Redacted (see `redactSecrets`) raw response body, kept only for
   * failure shapes this client doesn't specifically recognize. */
  rawResponseBody?: string
}

const REDACTED = '[redacted]'

/** A session token: 32 random bytes as base64url, so exactly 43 characters
 * of that alphabet (`crypto.rs`'s `new_session_token`). */
const SESSION_TOKEN_PATTERN =
  /(?<![A-Za-z0-9_-])[A-Za-z0-9_-]{43}(?![A-Za-z0-9_-])/g

/** A long run of hex characters is what a SHA-256 token hash (64 hex chars,
 * per `auth/session.rs`'s `hash_token`) looks like -- redact
 * generically even without knowing the exact value in scope. */
const HEX_HASH_PATTERN = /\b[0-9a-f]{32,}\b/gi

/**
 * Defensively redacts anything that looks like a session token or a
 * token hash from `text` before it is ever rendered in the error dialog.
 * This is a backstop, not the primary guarantee -- the primary guarantee is
 * that `VerificationFailure` (the worker's actual failure payload) has no
 * such field to begin with.
 */
export function redactSecrets(text: string): string {
  return text
    .replace(SESSION_TOKEN_PATTERN, REDACTED)
    .replace(HEX_HASH_PATTERN, REDACTED)
}

function describeRawBody(body: unknown): string {
  if (typeof body === 'string') return body
  return JSON.stringify(body) ?? ''
}

/**
 * Builds structured, dialog-ready failure info from whatever a Synced Share
 * request threw (`callWorker`'s `WorkerRequestError`/`NetworkFailure`).
 * Renders the worker's `unauthorized` `ApiError` verbosely; falls back to
 * the raw status/body (redacted) for any other failure, so the dialog stays
 * useful even for one this client doesn't specifically know about (per
 * §0's "maximally verbose" decision).
 */
export function buildSyncedShareFailure(
  operation: SyncedShareOperation,
  error: unknown,
): SyncedShareFailure {
  if (!(error instanceof WorkerRequestError)) {
    const message = error instanceof Error ? error.message : String(error)
    return {
      operation,
      reason: redactSecrets(`Network error: ${message}`),
      failedAt: Date.now(),
      authRejected: false,
    }
  }
  const { apiError, response } = error
  const traceId = error.traceId ?? undefined
  if (apiError?.code === 'unauthorized') {
    return {
      operation,
      reason: redactSecrets(apiError.reason),
      failedAt: apiError.failedAt,
      attempts: apiError.attempts,
      httpStatus: response.status,
      httpStatusText: response.statusText,
      traceId,
      authRejected: true,
    }
  }
  return {
    operation,
    reason: `Synced Share "${operation}" request failed`,
    failedAt: Date.now(),
    httpStatus: response.status,
    httpStatusText: response.statusText,
    traceId,
    rawResponseBody: redactSecrets(describeRawBody(error.rawBody)),
    authRejected: false,
  }
}
