import { afterEach, beforeAll, describe, expect, it, vi } from 'vitest'
import type { Message, SendRequest } from '../../poentaroWasm'
import { SessionEnded } from '../../protocol/transport'
import { WorkerRequestError } from '../../syncedShare/workerClient'
import { installRealWasm } from '../cloudBackendTestHelpers'
import { createFakeAppFiles, type FakeFile } from '../fakeAppFiles'
import { sendMessage } from './sendMessage'

beforeAll(installRealWasm)

afterEach(() => {
  vi.useRealTimers()
})

function requestFor(message: Message, expectedRevision = 3n): SendRequest {
  return {
    fileId: 'file-1',
    messageId: 'message-1',
    message,
    expectedRevision,
  }
}

const saveContent = (content: string): Message => ({
  tag: 'save-content',
  val: { content },
})

const fileOne: FakeFile = {
  id: 'file-1',
  path: 'a.poentaro',
  content: 'server text',
  revision: 3,
}

const WRITE = 'files/write _ at revision _ content _'

function send(message: Message, seed: FakeFile[] = [fileOne]) {
  const server = createFakeAppFiles(seed)
  return {
    server,
    result: sendMessage(requestFor(message), { app: server.app }),
  }
}

/** Sends a save to a server whose next write answers `failure`. */
function saveFailingWith(failure: Parameters<typeof server.failNext>[1]) {
  const server = createFakeAppFiles([fileOne])
  server.failNext(WRITE, failure)
  return sendMessage(requestFor(saveContent('x')), { app: server.app })
}
const server = createFakeAppFiles()

describe('sendMessage success', () => {
  it('saves content at the recorded revision and returns the new one', async () => {
    const { server, result } = send(saveContent('1 2 3'))
    expect(await result).toEqual({ tag: 'ok', val: { revision: 4n } })
    expect(server.lastCall()).toEqual({
      method: WRITE,
      args: ['file-1', 3, '1 2 3'],
    })
  })

  it('creates a file with the lane id', async () => {
    const { server, result } = send(
      { tag: 'create-file', val: { name: 'b.poentaro', content: 'x' } },
      [],
    )
    expect(await result).toEqual({ tag: 'ok', val: { revision: 0n } })
    expect(server.file('file-1')).toMatchObject({
      path: 'b.poentaro',
      content: 'x',
    })
  })

  it('takes a create whose earlier answer was lost as made', async () => {
    const { result } = send({
      tag: 'create-file',
      val: { name: 'a.poentaro', content: 'server text' },
    })
    expect(await result).toEqual({ tag: 'ok', val: { revision: 3n } })
  })

  it.each([
    [{ tag: 'rename-file', val: { to: 'b.poentaro' } } as Message],
    [{ tag: 'trash-file' } as Message],
  ])('keeps the recorded revision for %j', async (message) => {
    const { result } = send(message)
    expect(await result).toEqual({ tag: 'ok', val: { revision: 3n } })
  })

  it('restores, and takes a file already back as restored', async () => {
    const trashed = { ...fileOne, trashedAt: 5 }
    expect(await send({ tag: 'restore-file' }, [trashed]).result).toEqual({
      tag: 'ok',
      val: { revision: 3n },
    })
    expect(await send({ tag: 'restore-file' }).result).toEqual({
      tag: 'ok',
      val: { revision: 3n },
    })
  })

  it('takes trashing a file that is gone as done', async () => {
    expect(await send({ tag: 'trash-file' }, []).result).toEqual({
      tag: 'ok',
      val: { revision: 3n },
    })
  })
})

describe('sendMessage failures', () => {
  it('treats a network error as transient', async () => {
    const result = await saveFailingWith(new TypeError('Failed to fetch'))
    expect(result.tag).toBe('transient')
  })

  it('gives up waiting after the timeout and reports transient', async () => {
    vi.useFakeTimers()
    const calls = await server.app()
    const pending = sendMessage(requestFor(saveContent('x')), {
      app: async () => ({
        ...calls,
        files: {
          ...calls.files,
          writeAtRevisionContent: () => new Promise(() => undefined),
        },
      }),
      timeoutMs: 15_000,
    })
    await vi.advanceTimersByTimeAsync(15_000)
    expect((await pending).tag).toBe('transient')
  })

  it('treats a server failure as transient', async () => {
    const result = await saveFailingWith({ kind: 'internalError' })
    expect(result.tag).toBe('transient')
  })

  it("treats a failure status as transient, naming the request's trace ID", async () => {
    const response = new Response('', {
      status: 503,
      headers: { 'X-Trace-Id': '0123456789abcdef' },
    })
    const result = await saveFailingWith(new WorkerRequestError(response, null))
    expect(result).toEqual({
      tag: 'transient',
      val: { reason: 'status 503, trace ID 0123456789abcdef' },
    })
  })

  it('maps an ended session to unauthorized', async () => {
    const result = await saveFailingWith(new SessionEnded())
    expect(result).toEqual({ tag: 'unauthorized' })
  })

  it("names the refusing response's trace ID in a permanent reason", async () => {
    const empty = createFakeAppFiles([])
    const result = await sendMessage(requestFor(saveContent('x')), {
      app: async (options) => {
        options.onTraceId?.('0123456789abcdef')
        return empty.app()
      },
    })
    expect(result).toEqual({
      tag: 'permanent',
      val: { status: 404, reason: 'missing, trace ID 0123456789abcdef' },
    })
  })

  it('is permanent to save a file in the trash', async () => {
    const { result } = send(saveContent('x'), [{ ...fileOne, trashedAt: 1 }])
    expect(await result).toEqual({
      tag: 'permanent',
      val: { status: 410, reason: 'trashed' },
    })
  })
})

describe('sendMessage revision conflict', () => {
  const moved = { ...fileOne, revision: 9 }

  it('reports a conflict when the server text differs', async () => {
    const { result } = send(saveContent('mine'), [moved])
    expect(await result).toEqual({
      tag: 'conflict',
      val: { currentRevision: 9n },
    })
  })

  it('treats equal server text as a lost acknowledgement', async () => {
    const { result } = send(saveContent('server text'), [moved])
    expect(await result).toEqual({ tag: 'ok', val: { revision: 9n } })
  })
})

describe('sendMessage folder move', () => {
  const folderMove = (from: string, to: string): Message => ({
    tag: 'rename-files',
    val: {
      moves: [{ fileId: 'a', to: `${to}/a.poentaro` }],
      folder: { from, to },
    },
  })
  const inOld: FakeFile = { id: 'a', path: 'Old/a.poentaro', content: '' }

  it('moves the folder as one call', async () => {
    const { server, result } = send(folderMove('Old', 'New'), [inOld])
    expect(await result).toEqual({ tag: 'ok', val: { revision: 3n } })
    expect(server.calls).toEqual([
      { method: 'folders/move _ to _', args: ['Old', 'New'] },
    ])
    expect(server.file('a')?.path).toBe('New/a.poentaro')
  })

  it('takes a folder already moved as moved', async () => {
    const { result } = send(folderMove('Old', 'New'), [
      { ...inOld, path: 'New/a.poentaro' },
    ])
    expect(await result).toEqual({ tag: 'ok', val: { revision: 3n } })
  })

  it('adjusts a destination folder someone else made', async () => {
    const { result } = send(folderMove('Old', 'New'), [
      inOld,
      { id: 'b', path: 'New/b.poentaro', content: '' },
    ])
    expect(await result).toEqual({
      tag: 'names-taken',
      val: { adjustments: [{ rejected: 'New', replacement: 'New 2' }] },
    })
  })
})

describe('sendMessage folder move race', () => {
  it('names a file taken inside the destination on its own', async () => {
    const server = createFakeAppFiles([
      { id: 'a', path: 'Old/a.poentaro', content: '' },
    ])
    server.failNext('folders/move _ to _', {
      kind: 'error',
      error: { kind: 'destinationsTaken', paths: ['New/a.poentaro'] },
    })
    const result = await sendMessage(
      requestFor({
        tag: 'rename-files',
        val: {
          moves: [{ fileId: 'a', to: 'New/a.poentaro' }],
          folder: { from: 'Old', to: 'New' },
        },
      }),
      { app: server.app },
    )
    expect(result).toEqual({
      tag: 'names-taken',
      val: {
        adjustments: [
          { rejected: 'New/a.poentaro', replacement: 'New/a 2.poentaro' },
        ],
      },
    })
  })
})

describe('sendMessage batch queued without its folder', () => {
  const batch = (...moves: Array<{ fileId: string; to: string }>): Message => ({
    tag: 'rename-files',
    val: { moves },
  })

  it('moves each file, skipping one gone since', async () => {
    const { server, result } = send(
      batch(
        { fileId: 'a', to: 'New/a.poentaro' },
        { fileId: 'gone', to: 'New/b.poentaro' },
      ),
      [{ id: 'a', path: 'Old/a.poentaro', content: '' }],
    )
    expect(await result).toEqual({ tag: 'ok', val: { revision: 3n } })
    expect(server.file('a')?.path).toBe('New/a.poentaro')
  })

  it('adjusts each taken destination, dropping a folder marker', async () => {
    const { result } = send(
      batch(
        { fileId: 'a', to: 'New/a.poentaro' },
        { fileId: 'k', to: 'New/.keep' },
      ),
      [
        { id: 'a', path: 'Old/a.poentaro', content: '' },
        { id: 'k', path: 'Old/.keep', content: '' },
        { id: 'x', path: 'New/a.poentaro', content: '' },
        { id: 'y', path: 'New/.keep', content: '' },
      ],
    )
    expect(await result).toEqual({
      tag: 'names-taken',
      val: {
        adjustments: [
          { rejected: 'New/a.poentaro', replacement: 'New/a 2.poentaro' },
          { rejected: 'New/.keep', replacement: undefined },
        ],
      },
    })
  })
})

describe('sendMessage taken path', () => {
  const holder: FakeFile = {
    id: 'other',
    path: 'Hymns/Advent/O Come.poentaro',
    content: '',
  }

  it('suggests a new name for a create', async () => {
    const { result } = send(
      {
        tag: 'create-file',
        val: { name: 'Hymns/Advent/O Come.poentaro', content: '' },
      },
      [holder],
    )
    expect(await result).toEqual({
      tag: 'name-taken',
      val: { suggestedName: 'Hymns/Advent/O Come 2.poentaro' },
    })
  })

  it('suggests a new name for a rename, keeping the folder', async () => {
    const { result } = send(
      { tag: 'rename-file', val: { to: 'Hymns/Advent/O Come.poentaro' } },
      [fileOne, holder],
    )
    expect(await result).toEqual({
      tag: 'name-taken',
      val: { suggestedName: 'Hymns/Advent/O Come 2.poentaro' },
    })
  })

  it("takes a folder's marker that exists already as the folder", async () => {
    const { result } = send(
      { tag: 'create-file', val: { name: 'Hymns/.keep', content: '' } },
      [{ id: 'other', path: 'Hymns/.keep', content: '' }],
    )
    expect(await result).toEqual({ tag: 'ok', val: { revision: 3n } })
  })
})
