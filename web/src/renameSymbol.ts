import {
  poentaroWasm,
  type SymbolKind,
  type Symbol as WasmSymbol,
} from './poentaroWasm'
import { GM_INSTRUMENTS } from './utils/gmInstruments'
import { ensureWasmInit } from './wasmInit'

/** Every renamable symbol (part/group abbreviation, section label) in `source`. */
export async function listRenameSymbols(source: string): Promise<WasmSymbol[]> {
  await ensureWasmInit()
  const result = poentaroWasm().listSymbols(source, GM_INSTRUMENTS)
  return result.tag === 'ok' ? result.val.symbols : []
}

/**
 * Whether `byteOffset` falls within `span`. The end is inclusive: a caret
 * resting right after the last character of a symbol (the natural position
 * after clicking/typing it) still counts, which matters most for
 * single-character symbols where that's the only spot after the symbol at
 * all.
 */
function spanContainsOffset(
  span: { start: number; end: number },
  byteOffset: number,
): boolean {
  return byteOffset >= span.start && byteOffset <= span.end
}

/** The symbol (if any) with an occurrence spanning `byteOffset`. */
export function symbolAtByteOffset(
  symbols: WasmSymbol[],
  byteOffset: number,
): WasmSymbol | null {
  for (const symbol of symbols) {
    const hit = symbol.occurrences.some((occurrence) =>
      spanContainsOffset(occurrence.hitSpan, byteOffset),
    )
    if (hit) return symbol
  }
  return null
}

/** The occurrence of `symbol` (if any) spanning `byteOffset`. */
export function occurrenceAtByteOffset(symbol: WasmSymbol, byteOffset: number) {
  return symbol.occurrences.find((occurrence) =>
    spanContainsOffset(occurrence.hitSpan, byteOffset),
  )
}

export interface RenameTextEdit {
  start: number
  end: number
  replacement: string
}

/** Byte-offset text edits renaming every occurrence of `oldName` (of `kind`) to `newName`. */
export async function renameSymbolEdits(
  source: string,
  kind: SymbolKind,
  oldName: string,
  newName: string,
): Promise<RenameTextEdit[]> {
  await ensureWasmInit()
  const result = poentaroWasm().renameSymbol(
    source,
    kind,
    oldName,
    newName,
    GM_INSTRUMENTS,
  )
  if (result.tag !== 'ok') return []
  return result.val.edits.map((edit) => ({
    start: edit.span.start,
    end: edit.span.end,
    replacement: edit.replacement,
  }))
}
