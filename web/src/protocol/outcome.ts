import type { Reply } from '../generated/protocol/api'
import { SessionEnded } from './transport'

/** What a call of the old anyx protocol (`test`) comes back as. */
export type ProtocolOutcome<T, E> =
  | { kind: 'value'; value: T }
  | { kind: 'error'; error: E }
  | { kind: 'notRun' }
  | { kind: 'internalError' }
  | { kind: 'invalidRequest'; line: number; column: number; message: string }

/** What a generated call comes back as: a `Reply` of the core schema
 * `api`, or an outcome of the old protocol `test`. */
export type CallOutcome<T, E> = Reply<T, E> | ProtocolOutcome<T, E>

/** A call that came back with a value or one of its endpoint's errors. */
export type Settled<T, E> =
  | { kind: 'value'; value: T }
  | { kind: 'error'; error: E }

/** The worker failed the call (it logged why). */
export class ProtocolFailure extends Error {}

/** `outcome` as a value or one of its endpoint's errors. A session that
 * ended throws `SessionEnded`; a failed server throws `ProtocolFailure`. */
export function settled<T, E>(outcome: CallOutcome<T, E>): Settled<T, E> {
  switch (outcome.kind) {
    case 'value':
    case 'error':
      return outcome
    case 'invalidRequest':
      throw new ProtocolFailure(
        `the worker couldn't read the request at ${outcome.line}:${outcome.column}: ${outcome.message}`,
      )
    case 'notRun':
    case 'internalError':
      throw new ProtocolFailure(`the call failed: ${outcome.kind}`)
    case 'answered':
      return { kind: 'value', value: outcome.value }
    case 'failed':
      return { kind: 'error', error: outcome.problem }
    case 'unauthenticated':
      throw new SessionEnded('the session ended')
    case 'internal error':
      throw new ProtocolFailure('the call failed: internal error')
  }
}

/** `outcome`'s value; an error of its endpoint throws `ProtocolFailure`
 * too. For endpoints whose errors the caller has no answer to. */
export async function expectValue<T, E>(
  pending: Promise<CallOutcome<T, E>>,
): Promise<T> {
  const outcome = settled(await pending)
  if (outcome.kind === 'error') {
    throw new ProtocolFailure(
      `the call failed: ${JSON.stringify(outcome.error, (_, value) =>
        typeof value === 'bigint' ? value.toString() : value,
      )}`,
    )
  }
  return outcome.value
}
