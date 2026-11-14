import { useEffect, useState } from 'react'
import { ensureWasmInit } from '../wasmInit'

/** Whether the main-thread wasm component is ready, so a component that
 * calls Rust while rendering can wait for it: the first paint, and a
 * dialog opened right away, can come before it. */
export function useWasmReady(): boolean {
  const [ready, setReady] = useState(false)
  useEffect(() => {
    let cancelled = false
    void ensureWasmInit().then(() => {
      if (!cancelled) setReady(true)
    })
    return () => {
      cancelled = true
    }
  }, [])
  return ready
}
