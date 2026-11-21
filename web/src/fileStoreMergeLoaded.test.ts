import { describe, expect, it } from 'vitest'
import {
  type FileStoreState,
  loadedFiles,
  mergeLoadedTexts,
  NOT_LOADED,
} from './fileStore'

const unloaded: FileStoreState = {
  active: 'a.poentaro',
  userFiles: { 'a.poentaro': NOT_LOADED, 'b.poentaro': NOT_LOADED },
  bin: { 't.poentaro': NOT_LOADED },
  fileIds: { 'a.poentaro': 'id-a', 'b.poentaro': 'id-b', 't.poentaro': 'id-t' },
}

describe('mergeLoadedTexts', () => {
  it('fills a file that is still not loaded and leaves the others', () => {
    const loaded = {
      ...unloaded,
      userFiles: {
        ...unloaded.userFiles,
        ...loadedFiles({ 'a.poentaro': 'A' }),
      },
    }
    const merged = mergeLoadedTexts(unloaded, loaded)
    expect(merged.userFiles['a.poentaro']).toEqual({ tag: 'loaded', val: 'A' })
    expect(merged.userFiles['b.poentaro']).toEqual(NOT_LOADED)
  })

  it('fills a file in the bin', () => {
    const loaded = { ...unloaded, bin: loadedFiles({ 't.poentaro': 'T' }) }
    expect(mergeLoadedTexts(unloaded, loaded).bin['t.poentaro']).toEqual({
      tag: 'loaded',
      val: 'T',
    })
  })

  it('keeps an edit made while the text was being read', () => {
    const edited = {
      ...unloaded,
      userFiles: {
        ...unloaded.userFiles,
        ...loadedFiles({ 'a.poentaro': 'edited' }),
      },
    }
    const loaded = {
      ...unloaded,
      userFiles: {
        ...unloaded.userFiles,
        ...loadedFiles({ 'a.poentaro': 'server' }),
      },
    }
    expect(mergeLoadedTexts(edited, loaded).userFiles['a.poentaro']).toEqual({
      tag: 'loaded',
      val: 'edited',
    })
  })

  it('finds the file by id after a rename', () => {
    const renamed: FileStoreState = {
      ...unloaded,
      active: 'z.poentaro',
      userFiles: { 'z.poentaro': NOT_LOADED },
      fileIds: { 'z.poentaro': 'id-a' },
    }
    const loaded = {
      ...unloaded,
      userFiles: loadedFiles({ 'a.poentaro': 'A' }),
    }
    expect(mergeLoadedTexts(renamed, loaded).userFiles['z.poentaro']).toEqual({
      tag: 'loaded',
      val: 'A',
    })
  })
})
