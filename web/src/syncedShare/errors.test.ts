import { describe, expect, it } from 'vitest'
import { SessionEnded } from '../protocol/transport'
import { buildSyncedShareFailure, redactSecrets } from './errors'
import { NetworkFailure, WorkerRequestError } from './workerClient'

const TRACE_ID = '0123456789abcdef'

describe('redactSecrets', () => {
  it('redacts a session-token-shaped value', () => {
    expect(
      redactSecrets('token=Zq3_kP9-xV2mL8nR4tY7wB1cD6fH0jK5sA2eG9uI3oQ'),
    ).toBe('token=[redacted]')
  })

  it('leaves longer base64url runs alone', () => {
    const longer = 'Zq3_kP9-xV2mL8nR4tY7wB1cD6fH0jK5sA2eG9uI3oQx'
    expect(redactSecrets(longer)).toBe(longer)
  })

  it('redacts long hex runs that look like a token hash', () => {
    const hash = 'a'.repeat(64)
    expect(redactSecrets(`hash=${hash}`)).toBe('hash=[redacted]')
  })

  it('leaves ordinary text untouched', () => {
    expect(redactSecrets('verification failed: timeout after 3 attempts')).toBe(
      'verification failed: timeout after 3 attempts',
    )
  })
})

function workerError(
  status: number,
  statusText: string,
  body: string,
): WorkerRequestError {
  return new WorkerRequestError(
    new Response(null, {
      status,
      statusText,
      headers: { 'X-Trace-Id': TRACE_ID },
    }),
    body,
  )
}

describe('buildSyncedShareFailure: worker failure responses', () => {
  it('recognizes an ended protocol session', () => {
    const failure = buildSyncedShareFailure('start', new SessionEnded('ended'))
    expect(failure.authRejected).toBe(true)
    expect(failure.reason).toBe('Your session has ended.')
  })

  it('falls back to raw status/body (redacted) for any other failure', () => {
    const failure = buildSyncedShareFailure(
      'stop',
      workerError(
        500,
        'Internal Server Error',
        `internal error, hash=${'b'.repeat(40)}`,
      ),
    )
    expect(failure.operation).toBe('stop')
    expect(failure.httpStatus).toBe(500)
    expect(failure.httpStatusText).toBe('Internal Server Error')
    expect(failure.attempts).toBeUndefined()
    expect(failure.authRejected).toBe(false)
    expect(failure.rawResponseBody).toContain('[redacted]')
    expect(failure.rawResponseBody).not.toContain('b'.repeat(40))
  })

  it("keeps the trace ID from the header when the body isn't the worker's", () => {
    const failure = buildSyncedShareFailure(
      'stop',
      workerError(502, 'Bad Gateway', '<html>bad gateway</html>'),
    )
    expect(failure.traceId).toBe(TRACE_ID)
  })
})

describe('buildSyncedShareFailure: network failures', () => {
  it('describes a network failure', () => {
    const failure = buildSyncedShareFailure(
      'start',
      new NetworkFailure('Failed to fetch'),
    )
    expect(failure.operation).toBe('start')
    expect(failure.reason).toContain('Failed to fetch')
    expect(failure.attempts).toBeUndefined()
    expect(failure.authRejected).toBe(false)
    expect(failure.traceId).toBeUndefined()
  })

  it('redacts a token-shaped value from a network failure message', () => {
    const failure = buildSyncedShareFailure(
      'stop',
      new NetworkFailure(
        'leaked Zq3_kP9-xV2mL8nR4tY7wB1cD6fH0jK5sA2eG9uI3oQ in error',
      ),
    )
    expect(failure.reason).not.toContain(
      'Zq3_kP9-xV2mL8nR4tY7wB1cD6fH0jK5sA2eG9uI3oQ',
    )
  })
})
