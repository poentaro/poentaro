import { describe, expect, it } from 'vitest'
import {
  baseNameFromActiveFile,
  midiFilenameFromActiveFile,
  mp3FilenameFromActiveFile,
  pdfFilenameFromActiveFile,
  wavFilenameFromActiveFile,
  zipFilenameFromActiveFile,
} from './workerHelpers'

describe('export filenames', () => {
  it('drop the .poentaro extension', () => {
    expect(baseNameFromActiveFile('song.poentaro')).toBe('song')
    expect(pdfFilenameFromActiveFile('song.poentaro')).toBe('song.pdf')
  })

  it('keep a name that has no .poentaro extension', () => {
    expect(baseNameFromActiveFile('song')).toBe('song')
    expect(midiFilenameFromActiveFile('song')).toBe('song.mid')
  })

  it('drop the folders the file lives in', () => {
    const path = 'hymns/advent/song.poentaro'
    expect(baseNameFromActiveFile(path)).toBe('song')
    expect(pdfFilenameFromActiveFile(path)).toBe('song.pdf')
    expect(midiFilenameFromActiveFile(path)).toBe('song.mid')
    expect(wavFilenameFromActiveFile(path)).toBe('song.wav')
    expect(mp3FilenameFromActiveFile(path)).toBe('song.mp3')
    expect(zipFilenameFromActiveFile(path, [])).toBe('song.zip')
  })

  it('still mark the enabled parts', () => {
    expect(
      pdfFilenameFromActiveFile('hymns/song.poentaro', ['Soprano', 'Alto']),
    ).toBe('song (Soprano, Alto).pdf')
    expect(zipFilenameFromActiveFile('hymns/song.poentaro', ['split'])).toBe(
      'song (split).zip',
    )
  })
})
