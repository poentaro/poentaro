import { describe, expect, it } from 'vitest'
import { type FileStoreState, loadedText } from '../fileStore'
import { createTestBackend } from './cloudBackendTestHelpers'

const READ = 'read file _'
const WRITE = 'write file _ at revision _ content _'

const seed = [
  { id: 'id-a', path: 'a.poentaro', content: 'A text', revision: 2 },
  { id: 'id-b', path: 'b.poentaro', content: 'B text', revision: 1 },
  { id: 'id-t', path: 't.poentaro', content: 'T text', trashedAt: 5 },
]

async function loaded() {
  const harness = createTestBackend(seed)
  const state = await harness.backend.load()
  return { ...harness, state }
}

describe('createCloudBackend: ensureLoaded()', () => {
  it('reads one file and leaves the others not loaded', async () => {
    const { backend, server, state } = await loaded()
    const next = await backend.ensureLoaded(state, 'a.poentaro')
    expect(loadedText(next, 'a.poentaro')).toBe('A text')
    expect(loadedText(next, 'b.poentaro')).toBeUndefined()
    expect(server.callsTo(READ)).toHaveLength(1)
  })

  it('loads a file in the bin', async () => {
    const { backend, state } = await loaded()
    const next = await backend.ensureLoaded(state, 't.poentaro')
    expect(next.bin['t.poentaro']).toEqual({ tag: 'loaded', val: 'T text' })
  })

  it('does not read a file that is loaded already', async () => {
    const { backend, server, state } = await loaded()
    const next = await backend.ensureLoaded(state, 'a.poentaro')
    await backend.ensureLoaded(next, 'a.poentaro')
    expect(server.callsTo(READ)).toHaveLength(1)
  })

  it('shares one read between calls for the same file', async () => {
    const { backend, server, state } = await loaded()
    await Promise.all([
      backend.ensureLoaded(state, 'a.poentaro'),
      backend.ensureLoaded(state, 'a.poentaro'),
    ])
    expect(server.callsTo(READ)).toHaveLength(1)
  })

  it('reads from the on-device cache when the revision is unchanged', async () => {
    const { backend, server, texts, state } = await loaded()
    await texts.write('id-a', 2, 'cached text')
    const next = await backend.ensureLoaded(state, 'a.poentaro')
    expect(loadedText(next, 'a.poentaro')).toBe('cached text')
    expect(server.callsTo(READ)).toHaveLength(0)
  })

  it('reads again when the cached revision is older', async () => {
    const { backend, server, texts, state } = await loaded()
    await texts.write('id-a', 1, 'stale text')
    const next = await backend.ensureLoaded(state, 'a.poentaro')
    expect(loadedText(next, 'a.poentaro')).toBe('A text')
    expect(server.callsTo(READ)).toHaveLength(1)
  })

  it('shows unsent edits instead of the server text, without a read', async () => {
    const harness = createTestBackend(seed)
    harness.server.hang(WRITE)
    await harness.looper.start()
    await harness.looper.enqueue('id-a', {
      tag: 'save-content',
      val: { content: 'unsent edit' },
    })
    const state = await harness.backend.load()
    expect(loadedText(state, 'a.poentaro')).toBe('unsent edit')
    expect(harness.server.callsTo(READ)).toHaveLength(0)
  })

  it('rejects for a file the server no longer has', async () => {
    const { backend, server, state } = await loaded()
    server.failNext(READ, { kind: 'error', error: { kind: 'missing' } })
    await expect(backend.ensureLoaded(state, 'a.poentaro')).rejects.toThrow()
  })

  it('returns the state for an unknown name', async () => {
    const { backend, state } = await loaded()
    const same: FileStoreState = await backend.ensureLoaded(state, 'nope')
    expect(same).toBe(state)
  })

  it('still loads the text of a file renamed before it was opened', async () => {
    const { backend, server, state } = await loaded()
    const renamed = await backend.renameFile(state, 'b.poentaro', 'c.poentaro')
    expect(renamed.userFiles['c.poentaro']).toEqual({ tag: 'not-loaded' })
    const next = await backend.ensureLoaded(renamed, 'c.poentaro')
    expect(loadedText(next, 'c.poentaro')).toBe('B text')
    expect(server.callsTo(READ)).toHaveLength(1)
  })
})
