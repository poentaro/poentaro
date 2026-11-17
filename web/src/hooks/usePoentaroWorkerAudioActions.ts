import type { RefObject } from 'react'
import { useCallback } from 'react'
import type { WorkerRequest } from '../worker/poentaro.worker'
import { postAfterPaint } from './workerHelpers'

interface UsePoentaroWorkerAudioActionsParams {
  workerRef: RefObject<Worker | null>
  sourceRef: RefObject<string>
  enabledTracksRef: RefObject<string[] | undefined>
  wavUrlRef: RefObject<string | null>
  setWavUrl: (next: string | null) => void
  mp3UrlRef: RefObject<string | null>
  setMp3Url: (next: string | null) => void
  audioGenerating: boolean
  setAudioGenerating: (generating: boolean) => void
  audioRequestIdRef: RefObject<number>
  latestAudioIdRef: RefObject<number>
}

/** Revokes/replaces the current full-score preview WAV/MP3 URLs, and sends
 * the "generateAudio" request to the worker.
 *
 * Only one of `wavUrl`/`mp3Url` is ever non-null at a time — generating one
 * format revokes and clears the other, so `Preview` (which renders whichever
 * of the two is set) always shows the most recently generated audio rather
 * than stacking two inline players. */
export function usePoentaroWorkerAudioActions({
  workerRef,
  sourceRef,
  enabledTracksRef,
  wavUrlRef,
  setWavUrl,
  mp3UrlRef,
  setMp3Url,
  audioGenerating,
  setAudioGenerating,
  audioRequestIdRef,
  latestAudioIdRef,
}: UsePoentaroWorkerAudioActionsParams) {
  const setNextWavUrl = useCallback(
    (next: string | null) => {
      if (wavUrlRef.current) {
        URL.revokeObjectURL(wavUrlRef.current)
      }
      wavUrlRef.current = next
      setWavUrl(next)
      if (next !== null && mp3UrlRef.current) {
        URL.revokeObjectURL(mp3UrlRef.current)
        mp3UrlRef.current = null
        setMp3Url(null)
      }
    },
    [wavUrlRef, setWavUrl, mp3UrlRef, setMp3Url],
  )

  const setNextMp3Url = useCallback(
    (next: string | null) => {
      if (mp3UrlRef.current) {
        URL.revokeObjectURL(mp3UrlRef.current)
      }
      mp3UrlRef.current = next
      setMp3Url(next)
      if (next !== null && wavUrlRef.current) {
        URL.revokeObjectURL(wavUrlRef.current)
        wavUrlRef.current = null
        setWavUrl(null)
      }
    },
    [mp3UrlRef, setMp3Url, wavUrlRef, setWavUrl],
  )

  const generateFullAudio = useCallback(() => {
    const worker = workerRef.current
    if (!worker || audioGenerating) return
    const id = ++audioRequestIdRef.current
    latestAudioIdRef.current = id
    setAudioGenerating(true)
    postAfterPaint(worker, {
      type: 'generateAudio',
      source: sourceRef.current,
      id,
      enabledTracks: enabledTracksRef.current,
    } satisfies WorkerRequest)
  }, [
    audioGenerating,
    enabledTracksRef,
    workerRef,
    sourceRef,
    setAudioGenerating,
    latestAudioIdRef,
    audioRequestIdRef,
  ])

  return { setNextWavUrl, setNextMp3Url, generateFullAudio }
}
