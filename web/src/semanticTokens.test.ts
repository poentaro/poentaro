import { describe, expect, it } from 'vitest'
import type { HighlightKind, HighlightToken } from './poentaroWasm'
import { encodeSemanticTokens } from './semanticTokens'

const tokenTypes: HighlightKind[] = ['directive-key', 'lyric-reading']

function tokenAt(
  source: string,
  kind: HighlightKind,
  text: string,
  fromByte = 0,
): HighlightToken {
  const bytes = new TextEncoder().encode(source)
  const needle = new TextEncoder().encode(text)
  const start = bytes.findIndex(
    (_, index) =>
      index >= fromByte &&
      needle.every((byte, offset) => bytes[index + offset] === byte),
  )
  return { kind, span: { start, end: start + needle.length } }
}

describe('encodeSemanticTokens', () => {
  it('encodes positions relative to the previous token', () => {
    const source = 'break\nbpm=92 key=C4\n[M] 1\n'
    const tokens = [
      tokenAt(source, 'directive-key', 'break'),
      tokenAt(source, 'directive-key', 'bpm'),
      tokenAt(source, 'directive-key', 'key'),
    ]
    expect([...encodeSemanticTokens(source, tokens, tokenTypes)]).toEqual(
      [
        [0, 0, 5, 0, 0],
        [1, 0, 3, 0, 0],
        [0, 7, 3, 0, 0],
      ].flat(),
    )
  })

  it('counts columns in UTF-16 units after multi-byte text', () => {
    const source = '[M] 1 2\n[L] 歌😀 行(háng)\n'
    const tokens = [tokenAt(source, 'lyric-reading', '(háng)')]
    expect([...encodeSemanticTokens(source, tokens, tokenTypes)]).toEqual([
      1,
      '[L] 歌😀 行'.length,
      '(háng)'.length,
      1,
      0,
    ])
  })
})
