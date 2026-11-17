import { describe, expect, it, vi } from 'vitest'
import { DEMO_FILE_NAMES, type FileStoreState } from '../fileStore'
import {
  apiErrorResponse,
  createTestBackend,
  fetchMock,
  jsonResponse,
  lastCall,
} from './cloudBackendTestHelpers'

// See also `cloudBackendRetry.test.ts` for forceOverwrite/single-flight/
// offline-retry/name-collision-retry coverage -- split out to stay under
// this repo's 400-line cap.

describe('createCloudBackend: load()', () => {
  it('POSTs identityToken to /files/list and partitions rows by trashedAt', async () => {
    fetchMock.mockResolvedValue(
      jsonResponse(200, {
        files: [
          {
            id: 'id-a',
            name: 'a.poentaro',
            content: '1 2 3',
            revision: 3,
            trashedAt: null,
          },
          {
            id: 'id-b',
            name: 'b.poentaro',
            content: 'x',
            revision: 0,
            trashedAt: 12345,
          },
        ],
      }),
    )

    const backend = createTestBackend().backend
    const state = await backend.load()

    const { url, method, body } = await lastCall()
    expect(url).toBe('http://localhost:8787/files/list')
    expect(method).toBe('POST')
    expect(body).toEqual({ identityToken: 'test-token' })

    expect(state.userFiles).toEqual({ 'a.poentaro': '1 2 3' })
    expect(state.bin).toEqual({ 'b.poentaro': 'x' })
    expect(state.fileIds).toEqual({
      'a.poentaro': 'id-a',
      'b.poentaro': 'id-b',
    })
  })
})

describe('createCloudBackend: createFile()', () => {
  it('POSTs the generated id/name/content to /files', async () => {
    fetchMock.mockResolvedValue(
      jsonResponse(200, {
        id: 'whatever',
        name: 'untitled.poentaro',
        content: '',
        revision: 0,
        trashedAt: null,
      }),
    )

    const { backend, looper } = createTestBackend()
    const state: FileStoreState = {
      active: DEMO_FILE_NAMES[0] ?? '',
      userFiles: {},
      bin: {},
      fileIds: {},
    }
    const nextState = await backend.createFile(state, undefined, 'untitled')
    await looper.whenIdle()

    const { url, body } = await lastCall()
    expect(url).toBe('http://localhost:8787/files')
    expect(body.identityToken).toBe('test-token')
    expect(body.name).toBe('untitled.poentaro')
    expect(typeof body.id).toBe('string')
    expect(nextState.active).toBe('untitled.poentaro')
  })
})

describe('createCloudBackend: saveContent()', () => {
  function seededState(): FileStoreState {
    return {
      active: 'a.poentaro',
      userFiles: { 'a.poentaro': 'new content' },
      bin: {},
      fileIds: { 'a.poentaro': 'id-a' },
    }
  }

  async function loadWithRevision(
    backend: ReturnType<typeof createTestBackend>['backend'],
    revision: number,
  ): Promise<void> {
    fetchMock.mockResolvedValueOnce(
      jsonResponse(200, {
        files: [
          {
            id: 'id-a',
            name: 'a.poentaro',
            content: 'old content',
            revision,
            trashedAt: null,
          },
        ],
      }),
    )
    await backend.load()
  }

  it('returns once the save is persisted, before it is delivered', async () => {
    const { backend, looper } = createTestBackend()
    fetchMock.mockImplementation(() => new Promise(() => undefined))

    await backend.saveContent(seededState())

    expect(looper.snapshot().summary.totalPending).toBe(1)
    expect(backend.status()).toBe('saving')
  })

  it('sends the tracked revision as expectedRevision and advances it on success', async () => {
    const { backend, looper } = createTestBackend()
    await loadWithRevision(backend, 3)

    fetchMock.mockResolvedValueOnce(jsonResponse(200, { revision: 4 }))
    await backend.saveContent(seededState())
    await looper.whenIdle()

    const { url, body } = await lastCall()
    expect(url).toBe('http://localhost:8787/files/id-a/content')
    expect(body).toEqual({
      identityToken: 'test-token',
      content: 'new content',
      expectedRevision: 3,
    })
    expect(backend.status()).toBe('idle')

    fetchMock.mockResolvedValueOnce(jsonResponse(200, { revision: 5 }))
    await backend.saveContent(seededState())
    await looper.whenIdle()
    expect((await lastCall()).body).toMatchObject({ expectedRevision: 4 })
  })

  it('defaults expectedRevision to 0 for a file never seen via load()', async () => {
    const { backend, looper } = createTestBackend()
    fetchMock.mockResolvedValueOnce(jsonResponse(200, { revision: 1 }))

    await backend.saveContent(seededState())
    await looper.whenIdle()

    expect((await lastCall()).body).toMatchObject({ expectedRevision: 0 })
  })

  it('on a 409 revision conflict, reports the conflict and keeps the message', async () => {
    const { backend, looper } = createTestBackend()
    await loadWithRevision(backend, 3)
    fetchMock.mockResolvedValueOnce(
      apiErrorResponse(409, { code: 'revision_conflict', currentRevision: 7 }),
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

    await backend.saveContent(seededState())
    await looper.whenIdle()

    expect(backend.status()).toBe('error')
    expect(backend.lastError()).toEqual({
      kind: 'conflict',
      currentRevision: 7,
    })
    expect(looper.snapshot().summary.hasUnsynced).toBe(true)
  })

  it('401 classifies as an auth error and keeps the message', async () => {
    const { backend, looper } = createTestBackend()
    fetchMock.mockResolvedValueOnce(
      apiErrorResponse(401, {
        code: 'unauthorized',
        reason: 'expired',
        failedAt: 0,
        attempts: 3,
      }),
    )

    await backend.saveContent(seededState())
    await looper.whenIdle()

    expect(backend.status()).toBe('error')
    expect(backend.lastError()).toEqual({ kind: 'auth' })
    expect(looper.snapshot().summary.hasUnsynced).toBe(true)
  })

  it('reports the session as ended when a save is refused as unauthorized', async () => {
    const onSessionEnded = vi.fn()
    const { backend, looper } = createTestBackend({ onSessionEnded })
    fetchMock.mockResolvedValueOnce(
      apiErrorResponse(401, {
        code: 'unauthorized',
        reason: 'ended',
        failedAt: 0,
        attempts: 1,
      }),
    )

    await backend.saveContent(seededState())
    await looper.whenIdle()

    expect(onSessionEnded).toHaveBeenCalledTimes(1)
  })

  it('does not report the session as ended for any other failure', async () => {
    const onSessionEnded = vi.fn()
    const { backend, looper } = createTestBackend({ onSessionEnded })
    fetchMock.mockResolvedValueOnce(jsonResponse(500, { message: 'boom' }))

    await backend.saveContent(seededState())
    await looper.whenIdle()

    expect(onSessionEnded).not.toHaveBeenCalled()
  })

  it('classifies an unexpected non-2xx response as unknown', async () => {
    const { backend, looper } = createTestBackend()
    fetchMock.mockResolvedValueOnce(jsonResponse(500, { message: 'boom' }))

    await backend.saveContent(seededState())
    await looper.whenIdle()

    expect(backend.status()).toBe('offline')
    expect(backend.lastError()).toEqual({ kind: 'network' })
  })
})

describe('createCloudBackend: base snapshots', () => {
  const listing = (content: string) => async () =>
    jsonResponse(200, {
      files: [
        {
          id: 'id-a',
          name: 'a.poentaro',
          content,
          revision: 1,
          trashedAt: null,
        },
      ],
    })

  it('records each listed file text as its base on load', async () => {
    fetchMock.mockImplementation(listing('1 2 3'))
    const { backend } = createTestBackend()
    await backend.load()
    expect(await backend.bases().readBase('id-a')).toBe('1 2 3')
  })

  it('keeps the existing base of a file with unsent messages', async () => {
    const { backend, bases, looper } = createTestBackend()
    await bases.recordBase('id-a', 'old base')
    // The save never completes, so it stays unsent while the listing arrives.
    fetchMock.mockImplementation((request) =>
      request.url.endsWith('/files/list')
        ? listing('server text')()
        : new Promise<Response>(() => undefined),
    )
    await looper.start()
    await looper.enqueue('id-a', {
      tag: 'save-content',
      val: { content: 'local edit' },
    })
    await backend.load()
    expect(await bases.readBase('id-a')).toBe('old base')
  })

  it('records the acknowledged text as the base', async () => {
    const { bases, looper } = createTestBackend()
    fetchMock.mockImplementation(async () => jsonResponse(200, { revision: 2 }))
    await looper.start()
    await looper.enqueue('id-a', {
      tag: 'save-content',
      val: { content: 'saved text' },
    })
    await looper.whenIdle()
    expect(await bases.readBase('id-a')).toBe('saved text')
  })
})
