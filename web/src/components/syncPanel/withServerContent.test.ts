import { describe, expect, it } from 'vitest'
import type { FileStoreState } from '../../fileStore'
import { withServerContent } from './withServerContent'

const store: FileStoreState = {
  active: 'a.poentaro',
  userFiles: { 'a.poentaro': 'mine' },
  bin: {},
  fileIds: { 'a.poentaro': 'id-a' },
}

describe('withServerContent', () => {
  it('replaces the text of the file with that id', () => {
    expect(withServerContent(store, 'id-a', 'theirs').userFiles).toEqual({
      'a.poentaro': 'theirs',
    })
  })

  it('leaves the store alone for an unknown id', () => {
    expect(withServerContent(store, 'id-z', 'theirs')).toBe(store)
  })
})
