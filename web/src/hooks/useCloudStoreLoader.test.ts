import { describe, expect, it } from 'vitest'
import { SessionEnded } from '../protocol/transport'
import { NetworkFailure, WorkerRequestError } from '../syncedShare/workerClient'
import { cloudLoadRetryDelayMs, isAuthRejection } from './useCloudStoreLoader'

describe('cloudLoadRetryDelayMs', () => {
  it('backs off exponentially from one second', () => {
    expect([0, 1, 2, 3].map(cloudLoadRetryDelayMs)).toEqual([
      1_000, 2_000, 4_000, 8_000,
    ])
  })

  it('caps the delay at thirty seconds', () => {
    expect(cloudLoadRetryDelayMs(10)).toBe(30_000)
  })
})

function workerError(status: number, body: unknown): WorkerRequestError {
  return new WorkerRequestError(new Response(null, { status }), {
    body,
  })
}

describe('isAuthRejection', () => {
  it('treats an unauthorized ApiError as a rejected token', () => {
    expect(isAuthRejection(new SessionEnded('Your session has ended.'))).toBe(
      true,
    )
  })

  it('treats server errors and network failures as transient', () => {
    expect(
      isAuthRejection(workerError(500, { code: 'internal', message: 'boom' })),
    ).toBe(false)
    expect(isAuthRejection(new NetworkFailure('fetch failed'))).toBe(false)
  })
})
