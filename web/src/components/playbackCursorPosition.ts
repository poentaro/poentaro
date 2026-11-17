import type { NoteTiming } from '../poentaroWasm'

export interface ActiveNote {
  sourcePartIndex: number
  noteId: number
}

/**
 * Groups `noteTimings` by `sourcePartIndex`, sorting each part's list
 * ascending by `startS` (already the order `note_timings_seconds` produces
 * per part, but sorted defensively so `resolveActiveNotes`'s binary search
 * doesn't depend on that).
 */
export function groupNoteTimingsByPart(
  noteTimings: NoteTiming[],
): Map<number, NoteTiming[]> {
  const byPart = new Map<number, NoteTiming[]>()
  for (const timing of noteTimings) {
    const list = byPart.get(timing.sourcePartIndex)
    if (list) {
      list.push(timing)
    } else {
      byPart.set(timing.sourcePartIndex, [timing])
    }
  }
  for (const list of byPart.values()) {
    list.sort((a, b) => a.startS - b.startS)
  }
  return byPart
}

/**
 * Binary search for the last timing whose `startS <= t` in `timings`
 * (must be sorted ascending by `startS`). Returns `-1` when `t` is before
 * every timing's start.
 */
function findLastTimingStartingAtOrBefore(
  timings: NoteTiming[],
  t: number,
): number {
  let lo = 0
  let hi = timings.length - 1
  let result = -1
  while (lo <= hi) {
    const mid = (lo + hi) >> 1
    const midTiming = timings[mid]
    if (midTiming === undefined) break
    if (midTiming.startS <= t) {
      result = mid
      lo = mid + 1
    } else {
      hi = mid - 1
    }
  }
  return result
}

/**
 * The active `(sourcePartIndex, noteId)` per part at time `t`, given
 * `timingsByPart` (see `groupNoteTimingsByPart`). A part with no note
 * sounding at `t` — before its first note starts, past its last note's end,
 * or in a gap between timings — is omitted from the result rather than
 * falling back to a neighboring note, since there is nothing correct to
 * highlight there.
 */
export function resolveActiveNotes(
  t: number,
  timingsByPart: Map<number, NoteTiming[]>,
): ActiveNote[] {
  const active: ActiveNote[] = []
  for (const [sourcePartIndex, timings] of timingsByPart) {
    const index = findLastTimingStartingAtOrBefore(timings, t)
    if (index === -1) continue
    const timing = timings[index]
    if (timing === undefined || t >= timing.endS) continue
    active.push({ sourcePartIndex, noteId: timing.noteId })
  }
  return active
}
