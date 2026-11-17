import type { RefObject } from 'react'
import { useCallback } from 'react'
import type { EditorSelection, PitchDescription } from '../types'
import type { WorkerRequest } from '../worker/poentaro.worker'
import type { DescribeSelectionRequestTracker } from './usePoentaroWorkerTypes'

interface UsePoentaroWorkerDescribeSelectionParams {
  workerRef: RefObject<Worker | null>
  sourceRef: RefObject<string>
  describeSelectionTracker: DescribeSelectionRequestTracker
}

/** Asks the worker what the one note or chord `ranges` covers means in
 * letter names (see `pitch_description::describe_selection`). Read-only, so
 * unlike `usePoentaroWorkerEditRange` there's no selection to restore
 * afterward — a superseded request is simply never resolved. */
export function usePoentaroWorkerDescribeSelection({
  workerRef,
  sourceRef,
  describeSelectionTracker,
}: UsePoentaroWorkerDescribeSelectionParams) {
  const describeSelection = useCallback(
    (ranges: EditorSelection[]) =>
      new Promise<PitchDescription | null>((resolve) => {
        const worker = workerRef.current
        if (!worker) {
          resolve(null)
          return
        }
        const id = ++describeSelectionTracker.requestIdRef.current
        describeSelectionTracker.latestIdRef.current = id
        describeSelectionTracker.pendingRequestsRef.current.set(id, resolve)
        worker.postMessage({
          type: 'describeSelection',
          source: sourceRef.current,
          ranges,
          id,
        } satisfies WorkerRequest)
      }),
    [workerRef, sourceRef, describeSelectionTracker],
  )

  return { describeSelection }
}
