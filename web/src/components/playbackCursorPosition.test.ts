import { describe, expect, it } from 'vitest'
import type { NoteTiming } from '../poentaroWasm'
import {
  groupNoteTimingsByPart,
  resolveActiveNotes,
} from './playbackCursorPosition'

function timing(
  sourcePartIndex: number,
  noteId: number,
  startS: number,
  endS: number,
): NoteTiming {
  return {
    sourcePartIndex: sourcePartIndex,
    noteId: noteId,
    startS: startS,
    endS: endS,
  }
}

describe('groupNoteTimingsByPart', () => {
  it('groups by sourcePartIndex and sorts each group by startS', () => {
    const timings = [timing(0, 1, 1, 2), timing(1, 0, 0, 1), timing(0, 0, 0, 1)]
    const byPart = groupNoteTimingsByPart(timings)
    expect([...byPart.keys()].sort()).toEqual([0, 1])
    expect(byPart.get(0)?.map((t) => t.noteId)).toEqual([0, 1])
    expect(byPart.get(1)?.map((t) => t.noteId)).toEqual([0])
  })
})

describe('resolveActiveNotes', () => {
  const timingsByPart = groupNoteTimingsByPart([
    timing(0, 0, 0, 1),
    timing(0, 1, 1, 2),
    timing(0, 2, 2, 3),
    timing(1, 0, 0, 5),
  ])

  it('finds the active note per part at a given time', () => {
    expect(resolveActiveNotes(0.5, timingsByPart)).toEqual([
      { sourcePartIndex: 0, noteId: 0 },
      { sourcePartIndex: 1, noteId: 0 },
    ])
  })

  it('picks up the boundary note when t lands exactly on a start time', () => {
    expect(resolveActiveNotes(1, timingsByPart)).toEqual([
      { sourcePartIndex: 0, noteId: 1 },
      { sourcePartIndex: 1, noteId: 0 },
    ])
  })

  it('omits a part with no note active before its first timing starts', () => {
    const byPart = groupNoteTimingsByPart([timing(0, 0, 1, 2)])
    expect(resolveActiveNotes(0.5, byPart)).toEqual([])
  })

  it('omits a part with no note active past its last timing ends', () => {
    expect(resolveActiveNotes(3.5, timingsByPart)).toEqual([
      { sourcePartIndex: 1, noteId: 0 },
    ])
  })

  it('omits a part entirely when it has no timings at all in range', () => {
    expect(resolveActiveNotes(10, timingsByPart)).toEqual([])
  })

  it('returns an empty array for an empty timing set', () => {
    expect(resolveActiveNotes(0, groupNoteTimingsByPart([]))).toEqual([])
  })
})
