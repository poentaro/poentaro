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
  it('lists every file without reading any text', async () => {
    const { backend, server } = createTestBackend([
      { id: 'id-a', path: 'a.poentaro', content: 'a' },
      { id: 'id-b', path: 'b.poentaro', content: 'b' },
    ])
    const state = await backend.load()
    expect(Object.keys(state.userFiles)).toEqual(['a.poentaro', 'b.poentaro'])
    expect(server.callsTo('read file _')).toHaveLength(0)
  })
})

describe('createCloudBackend: file text cache', () => {
  const seed = [{ id: 'id-a', path: 'a.poentaro', content: '1 2', revision: 4 }]

  it('remembers the text of a file it fetched, at its revision', async () => {
    const { backend, texts } = createTestBackend(seed)
    await backend.fetchServerFile('id-a')
    expect(await texts.read('id-a', 4)).toBe('1 2')
    expect(await texts.read('id-a', 5)).toBeUndefined()
  })

  it('remembers the text the server acknowledged at the new revision', async () => {
    const { texts, looper } = createTestBackend([
      { id: 'id-a', path: 'a.poentaro', content: '1 2' },
    ])
    await looper.start()
    await looper.enqueue('id-a', {
      tag: 'save-content',
      val: { content: 'saved' },
    })
    await looper.whenIdle()
    expect(await texts.read('id-a', 1)).toBe('saved')
  })

  it("remembers the server's text from a conflict answer", async () => {
    const { texts, looper, server } = createTestBackend(seed)
    server.failNext('write file _ at revision _ content _', {
      kind: 'error',
      error: {
        kind: 'conflict at revision content',
        revision: 7,
        content: 'theirs',
      },
    })
    await looper.start()
    await looper.enqueue('id-a', {
      tag: 'save-content',
      val: { content: 'mine' },
    })
    await looper.whenIdle()
    expect(await texts.read('id-a', 7)).toBe('theirs')
  })
})
