import { describe, expect, it } from 'vitest'
import { createTestBackend } from './cloudBackendTestHelpers'

describe('createCloudBackend: fetchServerFile()', () => {
  const seed = [{ id: 'id-a', path: 'a.poentaro', content: '1 2', revision: 4 }]

  it('returns the revision and content of the file', async () => {
    const { backend } = createTestBackend(seed)
    expect(await backend.fetchServerFile('id-a')).toEqual({
      revision: 4,
      content: '1 2',
    })
  })

  it('returns undefined for a file the server lacks', async () => {
    const { backend } = createTestBackend(seed)
    expect(await backend.fetchServerFile('nope')).toBeUndefined()
  })
})

describe('createCloudBackend: load()', () => {
  it('reads every file, leaving out one deleted after the list', async () => {
    const { backend, server } = createTestBackend([
      { id: 'id-a', path: 'a.poentaro', content: 'a' },
      { id: 'id-b', path: 'b.poentaro', content: 'b' },
    ])
    server.failNext('files/read _', {
      kind: 'error',
      error: { kind: 'missing' },
    })
    const state = await backend.load()
    expect(Object.keys(state.userFiles)).toEqual(['b.poentaro'])
    expect(server.callsTo('files/read _')).toHaveLength(2)
  })
})
