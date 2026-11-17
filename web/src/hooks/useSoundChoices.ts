import { useEffect, useState } from 'react'
import { poentaroWasm, type SoundChoices } from '../poentaroWasm'
import { GM_INSTRUMENTS } from '../utils/gmInstruments'
import { ensureWasmInit } from '../wasmInit'

let cachedSoundChoices: SoundChoices | null = null

function soundChoices(): SoundChoices {
  cachedSoundChoices ??= poentaroWasm().listSoundChoices(GM_INSTRUMENTS)
  return cachedSoundChoices
}

/** Every sound the sound picker offers, labelled by Rust. `null` while
 * `enabled` is false (so a closed picker costs nothing) or until the
 * main-thread wasm component is ready. */
export function useSoundChoices(enabled: boolean): SoundChoices | null {
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

  return enabled && wasmReady ? soundChoices() : null
}
