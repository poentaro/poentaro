import { describe, expect, it } from 'vitest'
import type { HighlightKind, HighlightToken } from './poentaroWasm'
import { encodeSemanticTokens } from './semanticTokens'

const tokenTypes: HighlightKind[] = [
  'section-header',
  'metadata-key',
  'part-kind',
  'directive-key',
]

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
    const source = 'score\nbpm=92 key=C4\n[M] 1\n'
    const tokens = [
      tokenAt(source, 'section-header', 'score'),
      tokenAt(source, 'directive-key', 'bpm'),
      tokenAt(source, 'directive-key', 'key'),
    ]
    expect([...encodeSemanticTokens(source, tokens, tokenTypes)]).toEqual(
      [
        [0, 0, 5, 0, 0],
        [1, 0, 3, 3, 0],
        [0, 7, 3, 3, 0],
      ].flat(),
    )
  })

  it('counts columns in UTF-16 units after multi-byte text', () => {
    const source = 'metadata {\ntitle "歌😀", height 30 }\n'
    const tokens = [tokenAt(source, 'metadata-key', 'height')]
    expect([...encodeSemanticTokens(source, tokens, tokenTypes)]).toEqual([
      1,
      'title "歌😀", '.length,
      'height'.length,
      1,
      0,
    ])
  })
})
