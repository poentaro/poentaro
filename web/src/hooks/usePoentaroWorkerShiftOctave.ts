import type { RefObject } from 'react'
import { useCallback } from 'react'
import type { WorkerRequest } from '../worker/poentaro.worker'
import type { TextRequestTracker } from './usePoentaroWorkerTypes'

interface UsePoentaroWorkerShiftOctaveParams {
  workerRef: RefObject<Worker | null>
  sourceRef: RefObject<string>
  shiftPartOctaveTracker: TextRequestTracker
}

/** Sends a "notation octave" shift for one part to the worker and resolves
 * once it replies with the rewritten `.poentaro` source. */
export function usePoentaroWorkerShiftOctave({
  workerRef,
  sourceRef,
  shiftPartOctaveTracker,
}: UsePoentaroWorkerShiftOctaveParams) {
  const shiftPartOctave = useCallback(
    (abbreviation: string, delta: number) =>
      new Promise<string>((resolve) => {
        const worker = workerRef.current
        if (!worker) {
          resolve(sourceRef.current)
          return
        }
        const id = ++shiftPartOctaveTracker.requestIdRef.current
        shiftPartOctaveTracker.latestIdRef.current = id
        shiftPartOctaveTracker.pendingRequestsRef.current.set(id, resolve)
        worker.postMessage({
          type: 'shiftPartOctave',
          source: sourceRef.current,
          abbreviation,
          delta,
          id,
        } satisfies WorkerRequest)
      }),
    [workerRef, sourceRef, shiftPartOctaveTracker],
  )

  return { shiftPartOctave }
}
