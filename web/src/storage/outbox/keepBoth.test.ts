import { describe, expect, it } from 'vitest'
import type { FileStoreState } from '../../fileStore'
import {
  createTestBackend,
  fetchMock,
  jsonResponse,
} from '../cloudBackendTestHelpers'
import { keepBoth } from './keepBoth'

const state: FileStoreState = {
  active: 'a.poentaro',
  userFiles: { 'a.poentaro': 'local text' },
  bin: {},
  fileIds: { 'a.poentaro': 'id-a' },
}

const listing = jsonResponse(200, {
  files: [
    {
      id: 'id-a',
      name: 'a.poentaro',
      content: 'server text',
      revision: 4,
      trashedAt: null,
    },
  ],
})

async function run() {
  const harness = createTestBackend()
  fetchMock.mockImplementation(async (request) =>
    request.url.endsWith('/files/list')
      ? listing.clone()
      : jsonResponse(200, { revision: 1, id: 'x', name: 'n', content: '' }),
  )
  const next = await keepBoth({
    backend: harness.backend,
    state,
    fileId: 'id-a',
    today: '2026-10-10',
  })
  await harness.looper.whenIdle()
  return { harness, next }
}

describe('keepBoth', () => {
  it('saves the local text as a copy named with the date', async () => {
    const { next } = await run()
    expect(next.userFiles['a (conflicted copy 2026-10-10).poentaro']).toBe(
      'local text',
    )
  })

  it('queues the creation of the copy with the local text', async () => {
    const { harness } = await run()
    const bodies = await Promise.all(
      fetchMock.mock.calls
        .map(([request]) => request)
        .filter((request) => request.url.endsWith('/files'))
        .map((request) => request.clone().json()),
    )
    expect(bodies[0]).toMatchObject({
      name: 'a (conflicted copy 2026-10-10).poentaro',
      content: 'local text',
    })
    expect(harness.looper.snapshot().summary.hasUnsynced).toBe(false)
  })

  it('reverts the original to the server version and keeps it open', async () => {
    const { next } = await run()
    expect(next.userFiles['a.poentaro']).toBe('server text')
    expect(next.active).toBe('a.poentaro')
  })
})
