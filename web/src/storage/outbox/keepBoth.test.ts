import { describe, expect, it } from 'vitest'
import type { FileStoreState } from '../../fileStore'
import { loadedFiles, loadedText, NOT_LOADED } from '../../fileStore'
import { createTestBackend } from '../cloudBackendTestHelpers'
import { keepBoth } from './keepBoth'

const state: FileStoreState = {
  active: 'a.poentaro',
  userFiles: loadedFiles({ 'a.poentaro': 'local text' }),
  bin: loadedFiles({}),
  fileIds: { 'a.poentaro': 'id-a' },
}

async function run() {
  const harness = createTestBackend([
    { id: 'id-a', path: 'a.poentaro', content: 'server text', revision: 4 },
  ])
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
    expect(loadedText(next, 'a (conflicted copy 2026-10-10).poentaro')).toBe(
      'local text',
    )
  })

  it('queues the creation of the copy with the local text', async () => {
    const { harness } = await run()
    expect(
      harness.server.fileAt('a (conflicted copy 2026-10-10).poentaro')?.content,
    ).toBe('local text')
    expect(harness.looper.snapshot().summary.hasUnsynced).toBe(false)
  })

  it('reverts the original to the server version and keeps it open', async () => {
    const { next } = await run()
    expect(loadedText(next, 'a.poentaro')).toBe('server text')
    expect(next.active).toBe('a.poentaro')
  })

  it('reads the text of a file that was never loaded before copying it', async () => {
    const harness = createTestBackend([
      { id: 'id-a', path: 'a.poentaro', content: 'server text', revision: 4 },
    ])
    const next = await keepBoth({
      backend: harness.backend,
      state: { ...state, userFiles: { 'a.poentaro': NOT_LOADED } },
      fileId: 'id-a',
      today: '2026-10-10',
    })
    expect(loadedText(next, 'a (conflicted copy 2026-10-10).poentaro')).toBe(
      'server text',
    )
  })
})
