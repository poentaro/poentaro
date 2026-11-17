import { describe, expect, it } from 'vitest'
import type { FileStoreState } from '../fileStore'
import {
  createTestBackend,
  fetchMock,
  jsonResponse,
} from './cloudBackendTestHelpers'

const listed = (id: string, name: string, trashedAt: number | null = null) => ({
  id,
  name,
  content: '',
  revision: 1,
  trashedAt,
})

function neverAnswer() {
  fetchMock.mockImplementation(() => new Promise(() => undefined))
}

function lane(
  looper: ReturnType<typeof createTestBackend>['looper'],
  fileId: string,
) {
  return (
    looper
      .snapshot()
      .queue.lanes.find((candidate) => candidate.fileId === fileId)
      ?.messages.map(({ message }) => message) ?? []
  )
}

const emptyStore: FileStoreState = {
  active: 'a.poentaro',
  userFiles: { 'a.poentaro': 'A' },
  bin: {},
  fileIds: { 'a.poentaro': 'id-a' },
}

describe('createCloudBackend: empty folders are placeholder files', () => {
  it('load() turns active placeholders into folders and hides every placeholder row', async () => {
    fetchMock.mockResolvedValue(
      jsonResponse(200, {
        files: [
          listed('id-a', 'a.poentaro'),
          listed('id-k1', 'Empty/.keep'),
          listed('id-k2', 'Deleted/.keep', 5),
        ],
      }),
    )

    const state = await createTestBackend().backend.load()

    expect(state.userFiles).toEqual({ 'a.poentaro': '' })
    expect(state.bin).toEqual({})
    expect(state.fileIds).toEqual({ 'a.poentaro': 'id-a' })
    expect(state.folders).toEqual(['Empty'])
  })

  it('load() leaves a store without placeholders without a folders field', async () => {
    fetchMock.mockResolvedValue(
      jsonResponse(200, { files: [listed('id-a', 'a.poentaro')] }),
    )
    const state = await createTestBackend().backend.load()
    expect(state).not.toHaveProperty('folders')
  })

  it('createFolder queues an empty <folder>/.keep file', async () => {
    const { backend, looper } = createTestBackend()
    neverAnswer()

    const next = await backend.createFolder(emptyStore, undefined, 'New folder')

    expect(next.folders).toEqual(['New folder'])
    const queued = looper.snapshot().queue.lanes.flatMap((l) => l.messages)
    expect(queued.map(({ message }) => message)).toEqual([
      { tag: 'create-file', val: { name: 'New folder/.keep', content: '' } },
    ])
  })

  it('a still-queued create-folder shows after a reload', async () => {
    const { backend } = createTestBackend()
    neverAnswer()
    await backend.createFolder(emptyStore, undefined, 'New folder')
    fetchMock.mockReset()
    fetchMock.mockImplementation((request) =>
      request.url.endsWith('/files/list')
        ? Promise.resolve(
            jsonResponse(200, { files: [listed('id-a', 'a.poentaro')] }),
          )
        : new Promise(() => undefined),
    )

    const state = await backend.load()

    expect(state.folders).toEqual(['New folder'])
    expect(Object.keys(state.userFiles)).toEqual(['a.poentaro'])
  })

  it('renaming an empty folder renames its placeholder', async () => {
    fetchMock.mockResolvedValue(
      jsonResponse(200, { files: [listed('id-k', 'Old/.keep')] }),
    )
    const { backend, looper } = createTestBackend()
    const loaded = await backend.load()
    neverAnswer()

    const next = await backend.moveFolder(loaded, 'Old', 'Sub/New')

    expect(next.folders).toEqual(['Sub/New'])
    expect(
      looper
        .snapshot()
        .queue.lanes.filter((l) => l.fileId.startsWith('batch:'))
        .flatMap((l) => l.messages.map(({ message }) => message)),
    ).toEqual([
      {
        tag: 'rename-files',
        val: { moves: [{ fileId: 'id-k', to: 'Sub/New/.keep' }] },
      },
    ])
  })

  it('deleting an empty folder trashes its placeholder', async () => {
    fetchMock.mockResolvedValue(
      jsonResponse(200, { files: [listed('id-k', 'Old/.keep')] }),
    )
    const { backend, looper } = createTestBackend()
    const loaded = await backend.load()
    neverAnswer()

    const next = await backend.deleteFolder(loaded, 'Old')

    expect(next.folders ?? []).toEqual([])
    expect(next.bin).toEqual({})
    expect(lane(looper, 'id-k')).toEqual([{ tag: 'trash-file' }])
  })

  it('creating a folder whose deleted placeholder still holds the name restores it', async () => {
    fetchMock.mockResolvedValue(
      jsonResponse(200, {
        files: [
          listed('id-a', 'a.poentaro'),
          listed('id-k', 'New folder/.keep', 9),
        ],
      }),
    )
    const { backend, looper } = createTestBackend()
    const loaded = await backend.load()
    neverAnswer()

    await backend.createFolder(loaded, undefined, 'New folder')

    expect(lane(looper, 'id-k')).toEqual([{ tag: 'restore-file' }])
  })
})
