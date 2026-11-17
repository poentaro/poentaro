import { useEffect, useMemo, useState } from 'react'
import { type MetadataFields, poentaroWasm } from '../poentaroWasm'
import { ensureWasmInit } from '../wasmInit'

/** The field values of the preset called `presetName`, or of `metadata`
 * when `null`, parsed by the Rust metadata parser. `null` while `enabled` is false (so a closed Edit Metadata modal
 * costs nothing per keystroke) or until the main-thread wasm component is
 * ready. */
export function useMetadataFields(
  source: string,
  presetName: string | null,
  enabled: boolean,
): MetadataFields | null {
  const [wasmReady, setWasmReady] = useState(false)

  useEffect(() => {
    if (!enabled) return
    let cancelled = false
    void ensureWasmInit().then(() => {
      if (!cancelled) setWasmReady(true)
    })
    return () => {
      cancelled = true
    }
  }, [enabled])

  return useMemo(
    () =>
      enabled && wasmReady
        ? poentaroWasm().parseMetadataFields(source, presetName ?? undefined)
        : null,
    [enabled, wasmReady, source, presetName],
  )
}
