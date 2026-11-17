import { describe, expect, it } from 'vitest'
import type { Symbol as WasmSymbol } from './poentaroWasm'
import { symbolAtByteOffset } from './renameSymbol'

function makeSymbol(name: string, spans: Array<[number, number]>): WasmSymbol {
  return {
    name,
    kind: 'abbreviation',
    occurrences: spans.map(([start, end]) => ({
      span: { start, end },
      hitSpan: { start, end },
      role: 'declaration',
    })),
  }
}

function makeSymbolWithHitSpan(
  name: string,
  span: [number, number],
  hitSpan: [number, number],
): WasmSymbol {
  return {
    name,
    kind: 'section-label',
    occurrences: [
      {
        span: { start: span[0], end: span[1] },
        hitSpan: { start: hitSpan[0], end: hitSpan[1] },
        role: 'declaration',
      },
    ],
  }
}

describe('symbolAtByteOffset', () => {
  it('finds the symbol whose occurrence contains the offset', () => {
    const symbols = [makeSymbol('S', [[10, 11]]), makeSymbol('A', [[20, 21]])]
    expect(symbolAtByteOffset(symbols, 10)?.name).toBe('S')
    expect(symbolAtByteOffset(symbols, 20)?.name).toBe('A')
  })

  it('treats the span end as inclusive, so a caret resting right after the symbol still hits', () => {
    const symbols = [makeSymbol('S', [[10, 11]])]
    expect(symbolAtByteOffset(symbols, 11)?.name).toBe('S')
  })

  it('returns null just past the inclusive end', () => {
    const symbols = [makeSymbol('S', [[10, 11]])]
    expect(symbolAtByteOffset(symbols, 12)).toBeNull()
  })

  it('returns null when no occurrence contains the offset', () => {
    const symbols = [makeSymbol('S', [[10, 11]])]
    expect(symbolAtByteOffset(symbols, 5)).toBeNull()
  })

  it('hit-tests against hit_span, not span, so a caret anywhere in label="C" hits the declaration', () => {
    // span covers just the quoted text "C" (bytes 17-18); hit_span covers
    // the whole `label="C"` token (bytes 10-19).
    const symbols = [makeSymbolWithHitSpan('C', [17, 18], [10, 19])]
    expect(symbolAtByteOffset(symbols, 10)?.name).toBe('C') // start of `label=`
    expect(symbolAtByteOffset(symbols, 15)?.name).toBe('C') // inside `label=`
    expect(symbolAtByteOffset(symbols, 19)?.name).toBe('C') // just after closing quote
    expect(symbolAtByteOffset(symbols, 20)).toBeNull() // past the token
  })
})
