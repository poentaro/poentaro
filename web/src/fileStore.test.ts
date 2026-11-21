import { describe, expect, it } from 'vitest'
import {
  type FileStoreState,
  loadedBinText,
  loadedFiles,
  loadedText,
  mergeBackendResult,
  NOT_LOADED,
} from './fileStore'

function makeState(overrides: Partial<FileStoreState>): FileStoreState {
  return {
    active: 'a.poentaro',
    userFiles: loadedFiles({}),
    bin: loadedFiles({}),
    fileIds: {},
    ...overrides,
  }
}

describe('mergeBackendResult', () => {
  it('preserves a concurrent edit to an unrelated file made after base was read', () => {
    const base = makeState({
      active: 'a.poentaro',
      userFiles: loadedFiles({
        'a.poentaro': 'original a',
        'b.poentaro': 'original b',
      }),
      fileIds: { 'a.poentaro': 'id-a', 'b.poentaro': 'id-b' },
    })
    // createFile's async result: adds 'untitled.poentaro', computed from `base`.
    const next = makeState({
      active: 'untitled.poentaro',
      userFiles: loadedFiles({
        'a.poentaro': 'original a',
        'b.poentaro': 'original b',
        'untitled.poentaro': 'template',
      }),
      fileIds: {
        'a.poentaro': 'id-a',
        'b.poentaro': 'id-b',
        'untitled.poentaro': 'id-new',
      },
    })
    // While createFile was in flight, the user edited 'b.poentaro'.
    const prev = makeState({
      active: 'b.poentaro',
      userFiles: loadedFiles({
        'a.poentaro': 'original a',
        'b.poentaro': 'edited b',
      }),
      fileIds: { 'a.poentaro': 'id-a', 'b.poentaro': 'id-b' },
    })

    const result = mergeBackendResult(prev, base, next)

    expect(loadedText(result, 'b.poentaro')).toBe('edited b')
    expect(loadedText(result, 'untitled.poentaro')).toBe('template')
    expect(result.active).toBe('untitled.poentaro')
    expect(result.fileIds['untitled.poentaro']).toBe('id-new')
  })

  it('carries a live edit to the renamed file into the new name instead of the stale snapshot', () => {
    const base = makeState({
      active: 'old.poentaro',
      userFiles: loadedFiles({ 'old.poentaro': 'original' }),
      fileIds: { 'old.poentaro': 'id-1' },
    })
    const next = makeState({
      active: 'new.poentaro',
      userFiles: loadedFiles({ 'new.poentaro': 'original' }),
      fileIds: { 'new.poentaro': 'id-1' },
    })
    // User kept typing into 'old.poentaro' while the rename was in flight.
    const prev = makeState({
      active: 'old.poentaro',
      userFiles: loadedFiles({ 'old.poentaro': 'original plus more' }),
      fileIds: { 'old.poentaro': 'id-1' },
    })

    const result = mergeBackendResult(prev, base, next)

    expect(loadedText(result, 'new.poentaro')).toBe('original plus more')
    expect(result.userFiles['old.poentaro']).toBeUndefined()
    expect(result.active).toBe('new.poentaro')
  })

  it('does not force-switch active if the user already navigated away during the await', () => {
    const base = makeState({
      active: 'a.poentaro',
      userFiles: loadedFiles({ 'a.poentaro': 'content a' }),
    })
    const next = makeState({
      active: 'untitled.poentaro',
      userFiles: loadedFiles({
        'a.poentaro': 'content a',
        'untitled.poentaro': 'template',
      }),
      fileIds: { 'untitled.poentaro': 'id-new' },
    })
    const prev = makeState({
      active: 'b.poentaro',
      userFiles: loadedFiles({
        'a.poentaro': 'content a',
        'b.poentaro': 'content b',
      }),
    })

    const result = mergeBackendResult(prev, base, next)

    expect(result.active).toBe('untitled.poentaro')
    expect(loadedText(result, 'b.poentaro')).toBe('content b')
  })

  it('moves content from userFiles to bin on delete without losing unrelated edits', () => {
    const base = makeState({
      active: 'a.poentaro',
      userFiles: loadedFiles({
        'a.poentaro': 'content a',
        'b.poentaro': 'content b',
      }),
    })
    const next = makeState({
      active: 'b.poentaro',
      userFiles: loadedFiles({ 'b.poentaro': 'content b' }),
      bin: loadedFiles({ 'a.poentaro': 'content a' }),
    })
    const prev = makeState({
      active: 'b.poentaro',
      userFiles: loadedFiles({
        'a.poentaro': 'content a',
        'b.poentaro': 'edited b',
      }),
    })

    const result = mergeBackendResult(prev, base, next)

    expect(result.userFiles['a.poentaro']).toBeUndefined()
    expect(loadedBinText(result, 'a.poentaro')).toBe('content a')
    expect(loadedText(result, 'b.poentaro')).toBe('edited b')
  })

  it('finds the renamed file by id when two files hold the same text', () => {
    const base = makeState({
      active: 'a.poentaro',
      userFiles: loadedFiles({ 'a.poentaro': 'same', 'b.poentaro': 'same' }),
      fileIds: { 'a.poentaro': 'id-a', 'b.poentaro': 'id-b' },
    })
    const next = makeState({
      active: 'renamed.poentaro',
      userFiles: loadedFiles({
        'renamed.poentaro': 'same',
        'b.poentaro': 'same',
      }),
      fileIds: { 'renamed.poentaro': 'id-a', 'b.poentaro': 'id-b' },
    })
    // Both files were edited while the rename of `a` was in flight.
    const prev = makeState({
      active: 'a.poentaro',
      userFiles: loadedFiles({
        'a.poentaro': 'edited a',
        'b.poentaro': 'edited b',
      }),
      fileIds: { 'a.poentaro': 'id-a', 'b.poentaro': 'id-b' },
    })

    const result = mergeBackendResult(prev, base, next)

    expect(loadedText(result, 'renamed.poentaro')).toBe('edited a')
    expect(loadedText(result, 'b.poentaro')).toBe('edited b')
  })

  it('keeps a file that is not loaded as not loaded when it is renamed', () => {
    const base = makeState({
      userFiles: { 'a.poentaro': NOT_LOADED },
      fileIds: { 'a.poentaro': 'id-a' },
    })
    const next = makeState({
      userFiles: { 'b.poentaro': NOT_LOADED },
      fileIds: { 'b.poentaro': 'id-a' },
    })

    const result = mergeBackendResult(base, base, next)

    expect(result.userFiles).toEqual({ 'b.poentaro': NOT_LOADED })
  })
})
