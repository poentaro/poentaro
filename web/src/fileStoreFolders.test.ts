import { beforeAll, describe, expect, it } from 'vitest'
import {
  createFile,
  deleteFile,
  deleteFolder,
  duplicateFile,
  type FileStoreState,
  moveFolder,
  renameFile,
  restoreFile,
} from './fileStore'
import { deserializeFileStore } from './fileStorePersistence'
import { fileTreeRows } from './fileStoreQueries'
import { loadWasmForTests } from './testing/loadWasmForTests'

beforeAll(loadWasmForTests)

function makeState(overrides: Partial<FileStoreState>): FileStoreState {
  return {
    active: 'a.poentaro',
    userFiles: {},
    bin: {},
    fileIds: {},
    ...overrides,
  }
}

describe('file store operations delegated to Rust', () => {
  it('creates a named file inside a folder', () => {
    const state = makeState({
      userFiles: { 'Hymns/a.poentaro': 'x' },
      fileIds: { 'Hymns/a.poentaro': 'id-1' },
    })
    const next = createFile(state, 'Hymns', 'Psalm')
    expect(next.active).toBe('Hymns/Psalm.poentaro')
  })

  it('duplicates in the same folder', () => {
    const state = makeState({
      active: 'A/b.poentaro',
      userFiles: { 'A/b.poentaro': 'content' },
      fileIds: { 'A/b.poentaro': 'id-b' },
    })
    expect(duplicateFile(state).active).toBe('A/b 2.poentaro')
  })

  it('moves a file between folders keeping its id', () => {
    const state = makeState({
      userFiles: { 'a.poentaro': 'content' },
      fileIds: { 'a.poentaro': 'id-a' },
    })
    const next = renameFile(state, 'a.poentaro', 'X/Y/a')
    expect(next.userFiles).toEqual({ 'X/Y/a.poentaro': 'content' })
    expect(next.fileIds['X/Y/a.poentaro']).toBe('id-a')
  })

  it('returns the same state object when an operation is rejected', () => {
    const state = makeState({
      userFiles: { 'a.poentaro': '1', 'b.poentaro': '2' },
      fileIds: { 'a.poentaro': 'id-a', 'b.poentaro': 'id-b' },
    })
    expect(renameFile(state, 'a.poentaro', 'b.poentaro')).toBe(state)
  })

  it('moves a folder as a batch and reports every change', () => {
    const state = makeState({
      userFiles: { 'A/x.poentaro': '1', 'A/B/y.poentaro': '2' },
      fileIds: { 'A/x.poentaro': 'id-x', 'A/B/y.poentaro': 'id-y' },
      folders: ['A/Empty'],
    })
    const result = moveFolder(state, 'A', 'Z')
    expect(result.ok).toBe(true)
    if (!result.ok) return
    expect(Object.keys(result.state.userFiles).sort()).toEqual([
      'Z/B/y.poentaro',
      'Z/x.poentaro',
    ])
    expect(result.state.folders).toEqual(['Z/Empty'])
    expect(result.state.fileIds['Z/x.poentaro']).toBe('id-x')
    expect(result.changes).toHaveLength(3)
  })

  it('rejects a colliding folder move without changing anything', () => {
    const state = makeState({
      userFiles: { 'A/x.poentaro': '1', 'B/y.poentaro': '2' },
      fileIds: { 'A/x.poentaro': 'id-x', 'B/y.poentaro': 'id-y' },
    })
    const result = moveFolder(state, 'A', 'B')
    expect(result.ok).toBe(false)
    if (result.ok) return
    expect(result.failure.kind).toBe('collision')
  })

  it('deletes a folder to the bin and restores a file into the recreated folder', () => {
    const state = makeState({
      userFiles: { 'A/B/y.poentaro': '2' },
      fileIds: { 'A/B/y.poentaro': 'id-y' },
    })
    const deleted = deleteFolder(state, 'A')
    expect(deleted.userFiles).toEqual({})
    expect(deleted.bin).toEqual({ 'A/B/y.poentaro': '2' })
    const restored = restoreFile(deleted, 'A/B/y.poentaro')
    expect(restored.userFiles).toEqual({ 'A/B/y.poentaro': '2' })
    expect(restored.fileIds['A/B/y.poentaro']).toBe('id-y')
  })

  it('bins a single file under its full path', () => {
    const state = makeState({
      userFiles: { 'A/y.poentaro': '2' },
      fileIds: { 'A/y.poentaro': 'id-y' },
    })
    expect(deleteFile(state, 'A/y.poentaro').bin).toEqual({
      'A/y.poentaro': '2',
    })
  })

  it('lists the tree as sorted display rows', () => {
    const state = makeState({
      userFiles: { 'b.poentaro': '', 'A/z.poentaro': '' },
      folders: ['Empty'],
    })
    expect(
      fileTreeRows(state).map((row) => [row.path, row.depth, row.isFolder]),
    ).toEqual([
      ['A', 0, true],
      ['A/z.poentaro', 1, false],
      ['Empty', 0, true],
      ['b.poentaro', 0, false],
    ])
  })
})

describe('persistence of the optional folders field', () => {
  it('loads an old store without folders', () => {
    const state = deserializeFileStore(
      JSON.stringify({
        active: 'a.poentaro',
        userFiles: { 'a.poentaro': 'x' },
        bin: {},
        fileIds: { 'a.poentaro': 'id-a' },
      }),
    )
    expect(state.folders).toBeUndefined()
    expect(state.userFiles).toEqual({ 'a.poentaro': 'x' })
  })

  it('keeps folders and a nested active file when present', () => {
    const state = deserializeFileStore(
      JSON.stringify({
        active: 'A/a.poentaro',
        userFiles: { 'A/a.poentaro': 'x' },
        bin: {},
        fileIds: { 'A/a.poentaro': 'id-a' },
        folders: ['A/Empty', 3],
      }),
    )
    expect(state.folders).toEqual(['A/Empty'])
    expect(state.active).toBe('A/a.poentaro')
  })
})
