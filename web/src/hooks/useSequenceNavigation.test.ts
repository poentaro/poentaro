import { describe, expect, it } from 'vitest'
import type { MeasureSpan, SequenceEntry } from '../types'
import {
  computeSequenceSelectionLineRanges,
  envelopeOfLineRanges,
} from './useSequenceNavigation'

// A doc that declares sections in order a, b, c (one measure each), each
// measure spanning two source lines.
const measureSpans: MeasureSpan[] = [
  { startLine: 1, endLine: 2 } as MeasureSpan, // a
  { startLine: 3, endLine: 4 } as MeasureSpan, // b
  { startLine: 5, endLine: 6 } as MeasureSpan, // c
]

function entry(label: string, measureIndex: number): SequenceEntry {
  return {
    label,
    startMeasureIndex: measureIndex,
    endMeasureIndex: measureIndex,
  }
}

describe('computeSequenceSelectionLineRanges', () => {
  it('covers a single entry with its own line range', () => {
    const sequenceEntries = [entry('c', 2), entry('a', 0)]
    expect(
      computeSequenceSelectionLineRanges(sequenceEntries, measureSpans, 0, 0),
    ).toEqual([{ startLine: 5, endLine: 6 }])
  })

  it('returns one range per entry for an in-order chain (a, b)', () => {
    const sequenceEntries = [entry('a', 0), entry('b', 1)]
    expect(
      computeSequenceSelectionLineRanges(sequenceEntries, measureSpans, 0, 1),
    ).toEqual([
      { startLine: 1, endLine: 2 },
      { startLine: 3, endLine: 4 },
    ])
  })

  // Regression test: a chain referencing sections out of document order
  // (`c, a` when the doc declares `a, b, c`) previously collapsed into a
  // single Monaco range spanning from "a" through "c", which also swept up
  // "b" even though it isn't part of the chain. Each selected entry now
  // gets its own disjoint line range, so "a" and "c" are each selected on
  // their own and "b" (sitting between them in the source) is excluded.
  it('returns disjoint ranges for an out-of-document-order chain (c, a), excluding what lies between them', () => {
    const sequenceEntries = [entry('c', 2), entry('a', 0)]
    expect(
      computeSequenceSelectionLineRanges(sequenceEntries, measureSpans, 0, 1),
    ).toEqual([
      { startLine: 5, endLine: 6 }, // "c" only
      { startLine: 1, endLine: 2 }, // "a" only, "b" (lines 3-4) excluded
    ])
  })

  it('skips an index with no matching entry rather than resolving it', () => {
    const sequenceEntries = [entry('a', 0)]
    expect(
      computeSequenceSelectionLineRanges(sequenceEntries, measureSpans, 0, 1),
    ).toEqual([{ startLine: 1, endLine: 2 }])
  })

  it('skips an entry referencing a measure index with no span', () => {
    const sequenceEntries = [entry('missing', 99)]
    expect(
      computeSequenceSelectionLineRanges(sequenceEntries, measureSpans, 0, 0),
    ).toEqual([])
  })

  // A repeated label (e.g. a chain opening and closing on the same
  // section, `Intro, A, B, Intro`) resolves every occurrence to the same
  // written measure — and therefore the same line range — since `sequence`
  // replays already-written measures rather than duplicating them in the
  // source. Both occurrences must still come back as their own entry in the
  // result, not be deduplicated away.
  it('resolves a repeated label to the same line range for every occurrence', () => {
    const sequenceEntries = [entry('c', 2), entry('a', 0), entry('c', 2)]
    expect(
      computeSequenceSelectionLineRanges(sequenceEntries, measureSpans, 0, 2),
    ).toEqual([
      { startLine: 5, endLine: 6 }, // "c", first occurrence
      { startLine: 1, endLine: 2 }, // "a"
      { startLine: 5, endLine: 6 }, // "c", repeated occurrence, same lines
    ])
  })
})

describe('envelopeOfLineRanges', () => {
  it('returns null for an empty list', () => {
    expect(envelopeOfLineRanges([])).toBeNull()
  })

  it('spans the min start line to the max end line across all ranges', () => {
    expect(
      envelopeOfLineRanges([
        { startLine: 5, endLine: 6 },
        { startLine: 1, endLine: 2 },
      ]),
    ).toEqual({ startLine: 1, endLine: 6 })
  })
})
