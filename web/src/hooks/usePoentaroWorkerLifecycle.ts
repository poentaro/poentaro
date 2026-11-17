import type { RefObject } from 'react'
import { useEffect } from 'react'
import type { FontBytesByFamily } from '../poentaroWasm'
import { ensureWasmModule } from '../wasmInit'
import type { WorkerRequest } from '../worker/poentaro.worker'
import {
  createWorkerMessageHandler,
  type WorkerMessageHandlerDeps,
} from './usePoentaroWorkerMessageHandler'

export interface PoentaroWorkerLifecycleDeps extends WorkerMessageHandlerDeps {
  workerRef: RefObject<Worker | null>
  wavUrlRef: RefObject<string | null>
  mp3UrlRef: RefObject<string | null>
  measureWavUrlRef: RefObject<string | null>
  cursorOffsetTimerRef: RefObject<number | null>
  soundfontBytes: Uint8Array | null
  fontBytes: FontBytesByFamily | null
}

/** Creates and tears down the render worker, wires up its message handler, and forwards
 * the soundfont/PDF fonts to it once they've loaded. */
export function usePoentaroWorkerLifecycle(deps: PoentaroWorkerLifecycleDeps) {
  const {
    workerRef,
    wavUrlRef,
    mp3UrlRef,
    measureWavUrlRef,
    cursorOffsetTimerRef,
  } = deps

  // biome-ignore lint/correctness/useExhaustiveDependencies: deps are stable refs/setters from sibling hooks
  useEffect(() => {
    const worker = new Worker(
      new URL('../worker/poentaro.worker.ts', import.meta.url),
      { type: 'module' },
    )
    workerRef.current = worker

    ensureWasmModule()
      .then((module) => {
        if (workerRef.current === worker) {
          worker.postMessage({
            type: 'wasmModule',
            module,
          } satisfies WorkerRequest)
        }
      })
      .catch(() => {})

    worker.onmessage = createWorkerMessageHandler(deps)

    return () => {
      worker.terminate()
      workerRef.current = null
      if (wavUrlRef.current) {
        URL.revokeObjectURL(wavUrlRef.current)
        wavUrlRef.current = null
      }
      if (mp3UrlRef.current) {
        URL.revokeObjectURL(mp3UrlRef.current)
        mp3UrlRef.current = null
      }
      if (measureWavUrlRef.current) {
        URL.revokeObjectURL(measureWavUrlRef.current)
        measureWavUrlRef.current = null
      }
      if (cursorOffsetTimerRef.current !== null) {
        window.clearTimeout(cursorOffsetTimerRef.current)
      }
    }
  }, [deps.setNextWavUrl, deps.setNextMp3Url, deps.setNextMeasureWavUrl])

  useEffect(() => {
    const worker = workerRef.current
    if (!worker || !deps.soundfontBytes) return
    worker.postMessage({
      type: 'loadSoundfont',
      soundfont: deps.soundfontBytes.buffer as ArrayBuffer,
    } satisfies WorkerRequest)
  }, [deps.soundfontBytes, workerRef])

  useEffect(() => {
    const worker = workerRef.current
    if (!worker || !deps.fontBytes) return
    worker.postMessage({
      type: 'loadPdfFonts',
      fonts: deps.fontBytes,
    } satisfies WorkerRequest)
  }, [deps.fontBytes, workerRef])
}
