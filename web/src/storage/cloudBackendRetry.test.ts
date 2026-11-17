import { describe, expect, it } from 'vitest'
import type { FileStoreState } from '../fileStore'
import {
  apiErrorResponse,
  createTestBackend,
  fetchMock,
  jsonResponse,
  lastCall,
} from './cloudBackendTestHelpers'

// forceOverwrite/single-flight/offline-retry/name-collision-retry coverage
// for `cloudBackend.ts` -- split out of `cloudBackend.test.ts` to stay
// under this repo's 400-line cap. See that file for load/createFile/
// saveContent coverage.

describe('createCloudBackend: forceOverwrite()', () => {
  it('realigns the recorded revision to the conflict-reported one, then retries the save', async () => {
    const { backend, looper } = createTestBackend()
    const state: FileStoreState = {
      active: 'a.poentaro',
      userFiles: { 'a.poentaro': 'mine' },
      bin: {},
      fileIds: { 'a.poentaro': 'id-a' },
    }

    fetchMock.mockResolvedValueOnce(
      apiErrorResponse(409, { code: 'revision_conflict', currentRevision: 9 }),
    )
    // The conflict makes the sender compare against the server's text.
    fetchMock.mockResolvedValueOnce(
      jsonResponse(200, {
        files: [
          {
            id: 'id-a',
            name: 'a.poentaro',
            content: 'theirs',
            revision: 9,
            trashedAt: null,
          },
        ],
      }),
    )
    // The conflict flow then fetches the server's file for its own merge.
    fetchMock.mockResolvedValueOnce(
      jsonResponse(200, {
        files: [
          {
            id: 'id-a',
            name: 'a.poentaro',
            content: 'theirs',
            revision: 9,
            trashedAt: null,
          },
        ],
      }),
    )
    await backend.saveContent(state)
    await looper.whenIdle()
    expect(backend.lastError()).toEqual({
      kind: 'conflict',
      currentRevision: 9,
    })

    fetchMock.mockResolvedValueOnce(jsonResponse(200, { revision: 10 }))
    await backend.forceOverwrite(state)
    await looper.whenIdle()

    expect((await lastCall()).body).toMatchObject({ expectedRevision: 9 })
    expect(backend.status()).toBe('idle')
    expect(backend.lastError()).toBeNull()
  })
})

describe('createCloudBackend: delivery retry', () => {
  it('keeps an offline save queued and delivers it once the retry is due', async () => {
    const { backend, looper, wake, advance } = createTestBackend()
    const state: FileStoreState = {
      active: 'a.poentaro',
      userFiles: { 'a.poentaro': 'A' },
      bin: {},
      fileIds: { 'a.poentaro': 'id-a' },
    }

    fetchMock.mockRejectedValueOnce(new TypeError('Failed to fetch'))
    await backend.saveContent(state)
    await looper.whenIdle()
    expect(backend.status()).toBe('offline')
    expect(backend.lastError()).toEqual({ kind: 'network' })

    fetchMock.mockResolvedValueOnce(jsonResponse(200, { revision: 1 }))
    advance(120_000)
    wake()
    await looper.whenIdle()

    expect(backend.status()).toBe('idle')
    expect(fetchMock).toHaveBeenCalledTimes(2)
  })

  it('never runs two content deliveries concurrently', async () => {
    let active = 0
    let maxConcurrent = 0
    fetchMock.mockImplementation(async () => {
      active++
      maxConcurrent = Math.max(maxConcurrent, active)
      await new Promise((resolve) => setTimeout(resolve, 10))
      active--
      return jsonResponse(200, { revision: 1 })
    })
    const { backend, looper } = createTestBackend()
    const state: FileStoreState = {
      active: 'a.poentaro',
      userFiles: { 'a.poentaro': 'A' },
      bin: {},
      fileIds: { 'a.poentaro': 'id-a' },
    }

    await Promise.all([backend.saveContent(state), backend.saveContent(state)])
    await looper.whenIdle()

    expect(maxConcurrent).toBe(1)
  })
})
