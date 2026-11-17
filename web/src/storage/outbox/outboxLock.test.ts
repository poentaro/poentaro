import { afterEach, describe, expect, it, vi } from 'vitest'
import { runAsLeader } from './outboxLock'

type Waiter = {
  callback: () => Promise<void>
  resolve: () => void
  reject: (error: unknown) => void
  signal?: AbortSignal
}

function installFakeLocks() {
  let held = false
  const queue: Array<Waiter> = []
  const requestedNames: Array<string> = []
  const grantNext = () => {
    if (held) return
    const next = queue.shift()
    if (!next) return
    held = true
    next
      .callback()
      .then(next.resolve, next.reject)
      .finally(() => {
        held = false
        grantNext()
      })
  }
  const locks = {
    request: (
      name: string,
      options: { signal?: AbortSignal },
      callback: () => Promise<void>,
    ) => {
      requestedNames.push(name)
      return new Promise<void>((resolve, reject) => {
        const waiter: Waiter = {
          callback,
          resolve,
          reject,
          signal: options.signal,
        }
        options.signal?.addEventListener('abort', () => {
          const index = queue.indexOf(waiter)
          if (index >= 0) {
            queue.splice(index, 1)
            reject(new DOMException('aborted', 'AbortError'))
          }
        })
        queue.push(waiter)
        grantNext()
      })
    },
  }
  vi.stubGlobal('navigator', { locks })
  return { requestedNames }
}

const flush = () => new Promise((resolve) => setTimeout(resolve, 0))

function neverEndingTask(started: Array<string>, label: string) {
  return (signal: AbortSignal) =>
    new Promise<void>((resolve) => {
      started.push(label)
      signal.addEventListener('abort', () => resolve())
    })
}

afterEach(() => {
  vi.unstubAllGlobals()
})

describe('runAsLeader', () => {
  it('runs the task once the lock is acquired', async () => {
    const { requestedNames } = installFakeLocks()
    const started: Array<string> = []
    runAsLeader(neverEndingTask(started, 'a'))
    await flush()
    expect(started).toEqual(['a'])
    expect(requestedNames).toEqual(['poentaro-outbox-looper'])
  })

  it('aborts the signal and releases the lock on stop', async () => {
    installFakeLocks()
    let seenSignal: AbortSignal | undefined
    const stop = runAsLeader((signal) => {
      seenSignal = signal
      return new Promise<void>(() => undefined)
    })
    await flush()
    expect(seenSignal?.aborted).toBe(false)
    stop()
    expect(seenSignal?.aborted).toBe(true)
    const started: Array<string> = []
    runAsLeader(neverEndingTask(started, 'b'))
    await flush()
    expect(started).toEqual(['b'])
  })

  it('makes a second caller wait until the first stops', async () => {
    installFakeLocks()
    const started: Array<string> = []
    const stopFirst = runAsLeader(neverEndingTask(started, 'first'))
    runAsLeader(neverEndingTask(started, 'second'))
    await flush()
    expect(started).toEqual(['first'])
    stopFirst()
    await flush()
    expect(started).toEqual(['first', 'second'])
  })

  it('never runs a waiting task that is stopped before acquiring', async () => {
    installFakeLocks()
    const started: Array<string> = []
    const stopFirst = runAsLeader(neverEndingTask(started, 'first'))
    const stopSecond = runAsLeader(neverEndingTask(started, 'second'))
    await flush()
    stopSecond()
    stopFirst()
    await flush()
    expect(started).toEqual(['first'])
  })

  it('runs the task directly when navigator.locks is missing', async () => {
    vi.stubGlobal('navigator', {})
    const started: Array<string> = []
    let seenSignal: AbortSignal | undefined
    const stop = runAsLeader((signal) => {
      seenSignal = signal
      started.push('direct')
      return Promise.resolve()
    })
    await flush()
    expect(started).toEqual(['direct'])
    stop()
    expect(seenSignal?.aborted).toBe(true)
  })
})
