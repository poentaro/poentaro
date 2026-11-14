/** What a generated client's call can come back as. Every generated
 * `Outcome` has this shape. */
export type CallOutcome<T, E> =
  | { kind: 'value'; value: T }
  | { kind: 'error'; error: E }
  | { kind: 'notRun' }
  | { kind: 'internalError' }
  | { kind: 'invalidRequest'; line: number; column: number; message: string }

/** A call that came back with a value or one of its method's errors. */
export type Settled<T, E> =
  | { kind: 'value'; value: T }
  | { kind: 'error'; error: E }

/** The worker failed the call (it logged why), or couldn't read it. */
export class ProtocolFailure extends Error {}

/** `outcome` as a value or one of its method's errors; anything else
 * throws `ProtocolFailure`. */
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
  }
}

/** `outcome`'s value; an error of its method throws `ProtocolFailure` too.
 * For methods whose errors the caller has no answer to. */
export async function expectValue<T, E>(
  pending: Promise<CallOutcome<T, E>>,
): Promise<T> {
  const outcome = settled(await pending)
  if (outcome.kind === 'error') {
    throw new ProtocolFailure(
      `the call failed: ${JSON.stringify(outcome.error)}`,
    )
  }
  return outcome.value
}
