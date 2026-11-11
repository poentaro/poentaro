import { useEffect, useState } from 'react'
import {
  type PartSettingLimits,
  poentaroWasm,
  type RangeInstrumentChoice,
  type RangePitchChoice,
} from '../poentaroWasm'
import { ensureWasmInit } from '../wasmInit'

/** What the Edit Parts modal may set a part to, owned by Rust. */
export interface PartSettingOptions {
  limits: PartSettingLimits
  rangeInstruments: RangeInstrumentChoice[]
  rangePitches: RangePitchChoice[]
}

let cachedPartSettingOptions: PartSettingOptions | null = null

function partSettingOptions(): PartSettingOptions {
  cachedPartSettingOptions ??= {
    limits: poentaroWasm().getPartSettingLimits(),
    rangeInstruments: poentaroWasm().listRangeInstruments(),
    rangePitches: poentaroWasm().listRangePitches(),
  }
  return cachedPartSettingOptions
}

/** The valid volume and octave-offset ranges of a part, the instruments
 * it can name, and the pitches its range can span. `null` while `enabled` is false (so a closed modal costs
 * nothing) or until the main-thread wasm component is ready. */
export function usePartSettingOptions(
  enabled: boolean,
): PartSettingOptions | null {
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

  return enabled && wasmReady ? partSettingOptions() : null
}
