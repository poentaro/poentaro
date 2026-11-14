import { describe, expect, it } from 'vitest'
import type { FileStoreState } from '../fileStore'
import { createTestBackend } from './cloudBackendTestHelpers'

// forceOverwrite/single-flight/offline-retry coverage for `cloudBackend.ts`
// -- split out of `cloudBackend.test.ts` to stay under this repo's
// 400-line cap. See that file for load/createFile/saveContent coverage.

const WRITE = 'files/write _ at revision _ content _'

function stateWith(content: string): FileStoreState {
  return {
    active: 'a.poentaro',
    userFiles: { 'a.poentaro': content },
    bin: {},
    fileIds: { 'a.poentaro': 'id-a' },
  }
}

describe('createCloudBackend: forceOverwrite()', () => {
  it('realigns the recorded revision to the conflict-reported one, then retries the save', async () => {
    const { backend, looper, server } = createTestBackend([
      { id: 'id-a', path: 'a.poentaro', content: 'theirs', revision: 9 },
    ])
    const state = stateWith('mine')

    await backend.saveContent(state)
    await looper.whenIdle()
    expect(backend.lastError()).toEqual({
      kind: 'conflict',
      currentRevision: 9,
    })

    await backend.forceOverwrite(state)
    await looper.whenIdle()

    expect(server.lastCall()).toEqual({
      method: WRITE,
      args: ['id-a', 9, 'mine'],
    })
    expect(backend.status()).toBe('idle')
    expect(backend.lastError()).toBeNull()
  })
})

describe('createCloudBackend: delivery retry', () => {
  it('keeps an offline save queued and delivers it once the retry is due', async () => {
    const { backend, looper, server, wake, advance } = createTestBackend([
      { id: 'id-a', path: 'a.poentaro', content: '' },
    ])
    server.failNext(WRITE, new TypeError('Failed to fetch'))

    await backend.saveContent(stateWith('A'))
    await looper.whenIdle()
    expect(backend.status()).toBe('offline')
    expect(backend.lastError()).toEqual({ kind: 'network' })

    advance(120_000)
    wake()
    await looper.whenIdle()

    expect(backend.status()).toBe('idle')
    expect(server.callsTo(WRITE)).toHaveLength(2)
    expect(server.file('id-a')?.content).toBe('A')
  })

  it('never runs two content deliveries concurrently', async () => {
    const { backend, looper, server } = createTestBackend([
      { id: 'id-a', path: 'a.poentaro', content: '' },
    ])
    server.slow(10)

    await Promise.all([
      backend.saveContent(stateWith('A')),
      backend.saveContent(stateWith('A')),
    ])
    await looper.whenIdle()

    expect(server.maxInFlight()).toBe(1)
  })
})
