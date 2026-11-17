import type { StaffClef } from '../poentaroWasm'
import type { Notation } from './messages'

const STAFF_CLEFS: Partial<Record<Notation, StaffClef>> = {
  violin: 'violin',
  cello: 'cello',
}

/** The instrument a staff view is engraved for (which clef every staff
 * gets), or `undefined` for the views drawn by the numbered layout. */
export function staffClefFor(notation: Notation): StaffClef | undefined {
  return STAFF_CLEFS[notation]
}
