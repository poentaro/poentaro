import { describe, expect, it } from 'vitest'
import { DEMO_FILE_NAMES, type FileStoreState } from '../fileStore'
import { createTestBackend } from './cloudBackendTestHelpers'

const CREATE = 'files/create _ at _ content _'
const WRITE = 'files/write _ at revision _ content _'

const emptyState: FileStoreState = {
  active: DEMO_FILE_NAMES[0] ?? '',
  userFiles: {},
  bin: {},
  fileIds: {},
}

const stateWithFile: FileStoreState = {
  active: 'a.poentaro',
  userFiles: { 'a.poentaro': 'content' },
  bin: { 'binned.poentaro': 'old' },
  fileIds: { 'a.poentaro': 'id-a', 'binned.poentaro': 'id-binned' },
}

/** A backend whose server never answers. */
function unanswered() {
  const harness = createTestBackend()
  harness.server.hang()
  return harness
}

function queuedTags(
  looper: ReturnType<typeof createTestBackend>['looper'],
  fileId: string,
): string[] {
  const lane = looper.snapshot().queue.lanes.find((l) => l.fileId === fileId)
  return lane?.messages.map(({ message }) => message.tag) ?? []
}

describe('createCloudBackend: structural operations are applied locally at once', () => {
  it('createFile returns the new file and queues a create-file message', async () => {
    const { backend, looper } = unanswered()

    const next = await backend.createFile(emptyState, undefined, 'untitled')

    expect(next.active).toBe('untitled.poentaro')
    const id = next.fileIds['untitled.poentaro'] ?? ''
    expect(queuedTags(looper, id)).toEqual(['create-file'])
  })

  it('renameFile returns the renamed state and queues a rename-file message', async () => {
    const { backend, looper } = unanswered()

    const next = await backend.renameFile(
      stateWithFile,
      'a.poentaro',
      'b.poentaro',
    )

    expect(next.userFiles).toHaveProperty('b.poentaro')
    expect(queuedTags(looper, 'id-a')).toEqual(['rename-file'])
  })

  it('deleteFile moves the file to the bin and queues a trash-file message', async () => {
    const { backend, looper } = unanswered()

    const next = await backend.deleteFile(stateWithFile, 'a.poentaro')

    expect(next.bin).toHaveProperty('a.poentaro')
    expect(queuedTags(looper, 'id-a')).toEqual(['trash-file'])
  })

  it('restoreFile brings the file back and queues a restore-file message', async () => {
    const { backend, looper } = unanswered()

    const next = await backend.restoreFile(stateWithFile, 'binned.poentaro')

    expect(next.userFiles).toHaveProperty('binned.poentaro')
    expect(queuedTags(looper, 'id-binned')).toEqual(['restore-file'])
  })
})

describe('createCloudBackend: ordering within a file', () => {
  async function createThenEditOffline() {
    const harness = createTestBackend()
    harness.server.failNext(CREATE, new TypeError('Failed to fetch'))
    const created = await harness.backend.createFile(
      emptyState,
      undefined,
      'untitled',
    )
    await harness.backend.saveContent({
      ...created,
      userFiles: { ...created.userFiles, 'untitled.poentaro': 'edited' },
    })
    await harness.looper.whenIdle()
    return { ...harness, id: created.fileIds['untitled.poentaro'] ?? '' }
  }

  it('keeps a save behind the offline creation of the same file', async () => {
    const { backend, looper, id } = await createThenEditOffline()

    expect(queuedTags(looper, id)[0]).toBe('create-file')
    expect(backend.status()).toBe('offline')
  })

  it('sends the create before the save once the network is back', async () => {
    const { backend, looper, server, wake, advance, id } =
      await createThenEditOffline()

    advance(120_000)
    wake()
    await looper.whenIdle()

    expect(server.calls.map(({ method }) => method)).toEqual([
      CREATE,
      CREATE,
      WRITE,
    ])
    expect(server.lastCall()?.args).toEqual([id, 0, 'edited'])
    expect(backend.status()).toBe('idle')
  })
})

describe('createCloudBackend: a name collision found at delivery time', () => {
  it('renames the local file to the name the outbox picked', async () => {
    const { backend, looper, server } = createTestBackend([
      { id: 'other', path: 'untitled.poentaro', content: '' },
    ])
    const renames: Array<{ from: string; to: string }> = []
    backend.onLocalRename(({ from, to }) => renames.push({ from, to }))

    await backend.createFile(emptyState, undefined, 'untitled')
    await looper.whenIdle()

    expect(server.callsTo(CREATE).map(({ args }) => args[1])).toEqual([
      'untitled.poentaro',
      'untitled 2.poentaro',
    ])
    expect(renames).toEqual([
      { from: 'untitled.poentaro', to: 'untitled 2.poentaro' },
    ])
    expect(backend.status()).toBe('idle')
  })

  it('renames the local file after a rename collides', async () => {
    const { backend, looper, server } = createTestBackend([
      { id: 'id-a', path: 'a.poentaro', content: 'content' },
      { id: 'other', path: 'b.poentaro', content: '' },
    ])
    const renames: Array<{ from: string; to: string }> = []
    backend.onLocalRename(({ from, to }) => renames.push({ from, to }))

    await backend.renameFile(stateWithFile, 'a.poentaro', 'b.poentaro')
    await looper.whenIdle()

    expect(server.file('id-a')?.path).toBe('b 2.poentaro')
    expect(renames).toEqual([{ from: 'b.poentaro', to: 'b 2.poentaro' }])
  })
})
