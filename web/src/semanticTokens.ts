import type { HighlightKind, HighlightToken, Span } from './poentaroWasm'

/** Where a UTF-8 byte offset falls, as Monaco counts it: 0-indexed line and
 * UTF-16 column. */
interface LinePosition {
  line: number
  character: number
}

/**
 * Resolves the start and end of every wasm UTF-8 byte span to line
 * positions, keyed by byte offset, in one pass over `source`. `source` must
 * be LF-only, as the Rust side measures it.
 */
export function spanPositions(
  source: string,
  spans: Span[],
): Map<number, LinePosition> {
  const wanted = [
    ...new Set(spans.flatMap(({ start, end }) => [start, end])),
  ].sort((a, b) => a - b)
  const positions = new Map<number, LinePosition>()
  const encoder = new TextEncoder()
  const codePoints = source[Symbol.iterator]()
  let bytePos = 0
  let line = 0
  let character = 0
  for (const target of wanted) {
    while (bytePos < target) {
      const step = codePoints.next()
      if (step.done) break
      bytePos += encoder.encode(step.value).length
      if (step.value === '\n') {
        line += 1
        character = 0
      } else {
        character += step.value.length
      }
    }
    positions.set(target, { line, character })
  }
  return positions
}

/**
 * Encodes wasm highlight tokens in Monaco's semantic-tokens format: five
 * integers per token (delta line, delta start column, length, token type
 * index, modifier bits), each position relative to the previous token's.
 */
export function encodeSemanticTokens(
  source: string,
  tokens: HighlightToken[],
  tokenTypes: readonly HighlightKind[],
): Uint32Array {
  const positions = spanPositions(
    source,
    tokens.map(({ span }) => span),
  )
  let previous: LinePosition = { line: 0, character: 0 }
  const data = tokens.flatMap(({ kind, span }) => {
    const start = positions.get(span.start)
    const end = positions.get(span.end)
    if (!start || !end) return []
    const deltaLine = start.line - previous.line
    const deltaCharacter =
      deltaLine === 0 ? start.character - previous.character : start.character
    previous = start
    return [
      deltaLine,
      deltaCharacter,
      end.character - start.character,
      tokenTypes.indexOf(kind),
      0,
    ]
  })
  return new Uint32Array(data)
}
