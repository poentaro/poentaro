import { describe, expect, it } from 'vitest'
import { type FileStoreState, mergeBackendResult } from './fileStore'

function makeState(overrides: Partial<FileStoreState>): FileStoreState {
  return {
    active: 'a.poentaro',
    userFiles: {},
    bin: {},
    fileIds: {},
    ...overrides,
  }
}

describe('mergeBackendResult', () => {
  it('preserves a concurrent edit to an unrelated file made after base was read', () => {
    const base = makeState({
      active: 'a.poentaro',
      userFiles: { 'a.poentaro': 'original a', 'b.poentaro': 'original b' },
      fileIds: { 'a.poentaro': 'id-a', 'b.poentaro': 'id-b' },
    })
    // createFile's async result: adds 'untitled.poentaro', computed from `base`.
    const next = makeState({
      active: 'untitled.poentaro',
      userFiles: {
        'a.poentaro': 'original a',
        'b.poentaro': 'original b',
        'untitled.poentaro': 'template',
      },
      fileIds: {
        'a.poentaro': 'id-a',
        'b.poentaro': 'id-b',
        'untitled.poentaro': 'id-new',
      },
    })
    // While createFile was in flight, the user edited 'b.poentaro'.
    const prev = makeState({
      active: 'b.poentaro',
      userFiles: { 'a.poentaro': 'original a', 'b.poentaro': 'edited b' },
      fileIds: { 'a.poentaro': 'id-a', 'b.poentaro': 'id-b' },
    })

    const result = mergeBackendResult(prev, base, next)

    expect(result.userFiles['b.poentaro']).toBe('edited b')
    expect(result.userFiles['untitled.poentaro']).toBe('template')
    expect(result.active).toBe('untitled.poentaro')
    expect(result.fileIds['untitled.poentaro']).toBe('id-new')
  })

  it('carries a live edit to the renamed file into the new name instead of the stale snapshot', () => {
    const base = makeState({
      active: 'old.poentaro',
      userFiles: { 'old.poentaro': 'original' },
      fileIds: { 'old.poentaro': 'id-1' },
    })
    const next = makeState({
      active: 'new.poentaro',
      userFiles: { 'new.poentaro': 'original' },
      fileIds: { 'new.poentaro': 'id-1' },
    })
    // User kept typing into 'old.poentaro' while the rename was in flight.
    const prev = makeState({
      active: 'old.poentaro',
      userFiles: { 'old.poentaro': 'original plus more' },
      fileIds: { 'old.poentaro': 'id-1' },
    })

    const result = mergeBackendResult(prev, base, next)

    expect(result.userFiles['new.poentaro']).toBe('original plus more')
    expect(result.userFiles['old.poentaro']).toBeUndefined()
    expect(result.active).toBe('new.poentaro')
  })

  it('does not force-switch active if the user already navigated away during the await', () => {
    const base = makeState({
      active: 'a.poentaro',
      userFiles: { 'a.poentaro': 'content a' },
    })
    const next = makeState({
      active: 'untitled.poentaro',
      userFiles: { 'a.poentaro': 'content a', 'untitled.poentaro': 'template' },
      fileIds: { 'untitled.poentaro': 'id-new' },
    })
    const prev = makeState({
      active: 'b.poentaro',
      userFiles: { 'a.poentaro': 'content a', 'b.poentaro': 'content b' },
    })

    const result = mergeBackendResult(prev, base, next)

    expect(result.active).toBe('untitled.poentaro')
    expect(result.userFiles['b.poentaro']).toBe('content b')
  })

  it('moves content from userFiles to bin on delete without losing unrelated edits', () => {
    const base = makeState({
      active: 'a.poentaro',
      userFiles: { 'a.poentaro': 'content a', 'b.poentaro': 'content b' },
    })
    const next = makeState({
      active: 'b.poentaro',
      userFiles: { 'b.poentaro': 'content b' },
      bin: { 'a.poentaro': 'content a' },
    })
    const prev = makeState({
      active: 'b.poentaro',
      userFiles: { 'a.poentaro': 'content a', 'b.poentaro': 'edited b' },
    })

    const result = mergeBackendResult(prev, base, next)

    expect(result.userFiles['a.poentaro']).toBeUndefined()
    expect(result.bin['a.poentaro']).toBe('content a')
    expect(result.userFiles['b.poentaro']).toBe('edited b')
  })
})
