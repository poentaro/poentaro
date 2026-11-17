import type { RefObject } from 'react'
import { useCallback, useRef, useState } from 'react'
import type { WorkerRequest } from '../worker/poentaro.worker'

interface UseInstrumentPreviewParams {
  workerRef: RefObject<Worker | null>
}

/** Manages previewing an instrument or percussion sound via the worker's audio playback. */
export function useInstrumentPreview({
  workerRef,
}: UseInstrumentPreviewParams) {
  const [previewAudioPlaying, setPreviewAudioPlaying] = useState(false)
  const previewAudioRequestIdRef = useRef(0)
  const latestPreviewAudioIdRef = useRef(0)
  const currentPreviewAudioRef = useRef<HTMLAudioElement | null>(null)

  // biome-ignore lint/correctness/useExhaustiveDependencies: workerRef is a stable ref passed in as a param
  const previewInstrument = useCallback((programNumber: number) => {
    const worker = workerRef.current
    if (!worker) return
    const id = ++previewAudioRequestIdRef.current
    latestPreviewAudioIdRef.current = id
    worker.postMessage({
      type: 'previewInstrument',
      id,
      programNumber,
    } satisfies WorkerRequest)
  }, [])

  // biome-ignore lint/correctness/useExhaustiveDependencies: workerRef is a stable ref passed in as a param
  const previewPercussion = useCallback((key: number) => {
    const worker = workerRef.current
    if (!worker) return
    const id = ++previewAudioRequestIdRef.current
    latestPreviewAudioIdRef.current = id
    worker.postMessage({
      type: 'previewPercussion',
      id,
      key,
    } satisfies WorkerRequest)
  }, [])

  const stopPreviewInstrument = useCallback(() => {
    if (currentPreviewAudioRef.current) {
      currentPreviewAudioRef.current.pause()
    }
  }, [])

  return {
    previewAudioPlaying,
    setPreviewAudioPlaying,
    previewInstrument,
    previewPercussion,
    stopPreviewInstrument,
    latestPreviewAudioIdRef,
    currentPreviewAudioRef,
  }
}
