import type { NoteLabels } from '../poentaroWasm'
import type { Notation } from './messages'

const NOTE_LABELS: Record<Notation, NoteLabels> = {
  numbered: 'degrees',
  violin: 'degrees',
  cello: 'degrees',
  abc: 'letter-names',
  guitar: 'guitar-chords',
  harmonica: 'harmonica-tabs',
}

/** What the numbered layout's note heads read as for the preview's notation:
 * the ABC view swaps each degree for its letter name, the guitar view
 * follows each degree with its letter name (`1 (C)`) and draws a chord
 * diagram at each chord symbol, and the harmonica
 * view swaps each degree for its harmonica tab. */
export function noteLabelsFor(notation: Notation): NoteLabels {
  return NOTE_LABELS[notation]
}
