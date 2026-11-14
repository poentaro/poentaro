import { describe, expect, it, vi } from 'vitest'
import { DEMO_FILE_NAMES, type FileStoreState } from '../fileStore'
import { SessionEnded } from '../protocol/transport'
import { WorkerRequestError } from '../syncedShare/workerClient'
import { createTestBackend } from './cloudBackendTestHelpers'

// See also `cloudBackendRetry.test.ts` for forceOverwrite/single-flight/
// offline-retry/name-collision-retry coverage -- split out to stay under
// this repo's 400-line cap.

const WRITE = 'files/write _ at revision _ content _'

describe('createCloudBackend: load()', () => {
  it('lists the files and partitions them by trashedAt', async () => {
    const { backend, server } = createTestBackend([
      { id: 'id-a', path: 'a.poentaro', content: '1 2 3', revision: 3 },
      { id: 'id-b', path: 'b.poentaro', content: 'x', trashedAt: 12345 },
    ])
    const state = await backend.load()

    expect(server.callsTo('files/list')).toHaveLength(1)
    expect(state.userFiles).toEqual({ 'a.poentaro': '1 2 3' })
    expect(state.bin).toEqual({ 'b.poentaro': 'x' })
    expect(state.fileIds).toEqual({
      'a.poentaro': 'id-a',
      'b.poentaro': 'id-b',
    })
  })
})

describe('createCloudBackend: createFile()', () => {
  it('creates the file under a generated id', async () => {
    const { backend, looper, server } = createTestBackend()
    const state: FileStoreState = {
      active: DEMO_FILE_NAMES[0] ?? '',
      userFiles: {},
      bin: {},
      fileIds: {},
    }
    const nextState = await backend.createFile(state, undefined, 'untitled')
    await looper.whenIdle()

    const created = server.fileAt('untitled.poentaro')
    expect(created?.id).toBe(nextState.fileIds['untitled.poentaro'])
    expect(typeof created?.id).toBe('string')
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

  async function loadedAt(revision: number) {
    const harness = createTestBackend([
      { id: 'id-a', path: 'a.poentaro', content: 'old content', revision },
    ])
    await harness.backend.load()
    return harness
  }

  it('returns once the save is persisted, before it is delivered', async () => {
    const { backend, looper, server } = await loadedAt(0)
    server.hang(WRITE)

    await backend.saveContent(seededState())

    expect(looper.snapshot().summary.totalPending).toBe(1)
    expect(backend.status()).toBe('saving')
  })

  it('writes at the tracked revision and advances it on success', async () => {
    const { backend, looper, server } = await loadedAt(3)

    await backend.saveContent(seededState())
    await looper.whenIdle()

    expect(server.lastCall()).toEqual({
      method: WRITE,
      args: ['id-a', 3, 'new content'],
    })
    expect(backend.status()).toBe('idle')

    await backend.saveContent(seededState())
    await looper.whenIdle()
    expect(server.lastCall()?.args[1]).toBe(4)
  })

  it('writes at revision 0 a file never seen via load()', async () => {
    const { backend, looper, server } = createTestBackend([
      { id: 'id-a', path: 'a.poentaro', content: '' },
    ])

    await backend.saveContent(seededState())
    await looper.whenIdle()

    expect(server.lastCall()?.args[1]).toBe(0)
  })

  it('on a revision conflict, reports the conflict and keeps the message', async () => {
    const { backend, looper, server } = await loadedAt(3)
    server.failNext(WRITE, {
      kind: 'error',
      error: {
        kind: 'conflictAtRevisionContent',
        revision: 7,
        content: 'theirs',
      },
    })

    await backend.saveContent(seededState())
    await looper.whenIdle()

    expect(backend.status()).toBe('error')
    expect(backend.lastError()).toEqual({
      kind: 'conflict',
      currentRevision: 7,
    })
    expect(looper.snapshot().summary.hasUnsynced).toBe(true)
  })

  it('an ended session classifies as an auth error and keeps the message', async () => {
    const { backend, looper, server } = await loadedAt(0)
    server.failNext(WRITE, new SessionEnded())

    await backend.saveContent(seededState())
    await looper.whenIdle()

    expect(backend.status()).toBe('error')
    expect(backend.lastError()).toEqual({ kind: 'auth' })
    expect(looper.snapshot().summary.hasUnsynced).toBe(true)
  })

  it('reports the session as ended when a save is refused as unauthorized', async () => {
    const onSessionEnded = vi.fn()
    const { backend, looper, server } = createTestBackend([], {
      onSessionEnded,
    })
    server.failNext(WRITE, new SessionEnded())

    await backend.saveContent(seededState())
    await looper.whenIdle()

    expect(onSessionEnded).toHaveBeenCalledTimes(1)
  })

  it('does not report the session as ended for any other failure', async () => {
    const onSessionEnded = vi.fn()
    const { backend, looper, server } = createTestBackend([], {
      onSessionEnded,
    })
    server.failNext(
      WRITE,
      new WorkerRequestError(new Response('', { status: 500 }), null),
    )

    await backend.saveContent(seededState())
    await looper.whenIdle()

    expect(onSessionEnded).not.toHaveBeenCalled()
  })

  it('classifies an unexpected failure status as offline', async () => {
    const { backend, looper, server } = await loadedAt(0)
    server.failNext(
      WRITE,
      new WorkerRequestError(new Response('', { status: 500 }), null),
    )

    await backend.saveContent(seededState())
    await looper.whenIdle()

    expect(backend.status()).toBe('offline')
    expect(backend.lastError()).toEqual({ kind: 'network' })
  })
})

describe('createCloudBackend: base snapshots', () => {
  const fileA = (content: string) => [
    { id: 'id-a', path: 'a.poentaro', content, revision: 1 },
  ]

  it('records each listed file text as its base on load', async () => {
    const { backend } = createTestBackend(fileA('1 2 3'))
    await backend.load()
    expect(await backend.bases().readBase('id-a')).toBe('1 2 3')
  })

  it('keeps the existing base of a file with unsent messages', async () => {
    const { backend, bases, looper, server } = createTestBackend(
      fileA('server text'),
    )
    await bases.recordBase('id-a', 'old base')
    // The save never completes, so it stays unsent while the listing arrives.
    server.hang(WRITE)
    await looper.start()
    await looper.enqueue('id-a', {
      tag: 'save-content',
      val: { content: 'local edit' },
    })
    await backend.load()
    expect(await bases.readBase('id-a')).toBe('old base')
  })

  it('records the acknowledged text as the base', async () => {
    const { bases, looper } = createTestBackend([
      { id: 'id-a', path: 'a.poentaro', content: '' },
    ])
    await looper.start()
    await looper.enqueue('id-a', {
      tag: 'save-content',
      val: { content: 'saved text' },
    })
    await looper.whenIdle()
    expect(await bases.readBase('id-a')).toBe('saved text')
  })
})
