import { describe, expect, it } from 'vitest'
import { documentTitleFor } from './useDocumentTitle'

describe('documentTitleFor', () => {
  it('strips the .poentaro extension from the active file name', () => {
    expect(documentTitleFor('twinkle.poentaro')).toBe('twinkle · Poentaro')
  })

  it('leaves a name without the .poentaro extension untouched', () => {
    expect(documentTitleFor('untitled')).toBe('untitled · Poentaro')
  })
})
