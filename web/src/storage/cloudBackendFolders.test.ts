import { describe, expect, it } from 'vitest'
import type { FileStoreState } from '../fileStore'
import { createTestBackend, fetchMock } from './cloudBackendTestHelpers'

const state: FileStoreState = {
  active: 'Scratch.poentaro',
  userFiles: {
    'Hymns/Advent/O Come.poentaro': 'a',
    'Hymns/Amazing.poentaro': 'b',
    'Scratch.poentaro': 'c',
  },
  bin: {},
  fileIds: {
    'Hymns/Advent/O Come.poentaro': 'id-come',
    'Hymns/Amazing.poentaro': 'id-amazing',
    'Scratch.poentaro': 'id-scratch',
  },
}

function neverAnswer() {
  fetchMock.mockImplementation(() => new Promise(() => undefined))
}

function queued(
  looper: ReturnType<typeof createTestBackend>['looper'],
  fileId: string,
) {
  const lane = looper.snapshot().queue.lanes.find((l) => l.fileId === fileId)
  return lane?.messages.map(({ message }) => message) ?? []
}

/** Every message on the batch lanes (a folder move's single message). */
function batches(looper: ReturnType<typeof createTestBackend>['looper']) {
  return looper
    .snapshot()
    .queue.lanes.filter((lane) => lane.fileId.startsWith('batch:'))
    .flatMap((lane) => lane.messages.map(({ message }) => message))
}

describe('createCloudBackend: folder operations', () => {
  it('moveFolder queues one batch rename covering every file inside', async () => {
    const { backend, looper } = createTestBackend()
    neverAnswer()

    const next = await backend.moveFolder(state, 'Hymns', 'Songs')

    expect(Object.keys(next.userFiles).sort()).toEqual([
      'Scratch.poentaro',
      'Songs/Advent/O Come.poentaro',
      'Songs/Amazing.poentaro',
    ])
    expect(batches(looper)).toEqual([
      {
        tag: 'rename-files',
        val: {
          moves: [
            { fileId: 'id-come', to: 'Songs/Advent/O Come.poentaro' },
            { fileId: 'id-amazing', to: 'Songs/Amazing.poentaro' },
          ],
        },
      },
    ])
    expect(queued(looper, 'id-come')).toEqual([])
    expect(queued(looper, 'id-amazing')).toEqual([])
    expect(queued(looper, 'id-scratch')).toEqual([])
  })

  it('moveFolder onto an existing path changes and queues nothing', async () => {
    const { backend, looper } = createTestBackend()
    neverAnswer()

    const next = await backend.moveFolder(state, 'Hymns', 'Scratch.poentaro')

    expect(next).toBe(state)
    expect(batches(looper)).toEqual([])
  })

  it('deleteFolder trashes every file inside on its own', async () => {
    const { backend, looper } = createTestBackend()
    neverAnswer()

    const next = await backend.deleteFolder(state, 'Hymns')

    expect(Object.keys(next.bin).sort()).toEqual([
      'Hymns/Advent/O Come.poentaro',
      'Hymns/Amazing.poentaro',
    ])
    expect(queued(looper, 'id-come')).toEqual([{ tag: 'trash-file' }])
    expect(queued(looper, 'id-amazing')).toEqual([{ tag: 'trash-file' }])
    expect(queued(looper, 'id-scratch')).toEqual([])
  })
})
