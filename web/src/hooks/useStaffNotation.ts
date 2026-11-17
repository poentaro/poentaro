import type { RefObject } from 'react'
import { useCallback, useEffect, useMemo, useState } from 'react'
import { type PageLayout, poentaroWasm } from '../poentaroWasm'
import type { ResolvedPartVisibility } from '../types'
import type {
  Notation,
  StaffRenderResult,
  StaffScore,
  WorkerRequest,
} from '../worker/messages'
import { staffClefFor } from '../worker/staffClef'
import type { StaffRenderRequestTracker } from './usePoentaroWorkerTypes'

export type { Notation }

/** The numbered/staff preview toggle plus what the staff view shows. */
export interface StaffNotation {
  notation: Notation
  setNotation: (notation: Notation) => void
  /** The engraved current source; no pages until the first staff render
   * lands. */
  score: StaffScore
  /** True while a staff render (including the first, which also downloads
   * Verovio) is in flight. */
  loading: boolean
  error: string | null
}

const EMPTY_SCORE: StaffScore = { pages: [], measures: [] }

interface UseStaffNotationParams {
  workerRef: RefObject<Worker | null>
  source: string
  visibility: ResolvedPartVisibility
  /** A4 pages, or max view's continuous page (`usePreviewLayout`). */
  pageLayout: PageLayout
  staffRenderTracker: StaffRenderRequestTracker
  debounceMs: number
}

/** The `notation` line of the preset in force, if it has one. */
function presetNotation(
  source: string,
  presetName: string | undefined,
): Notation | undefined {
  if (presetName == null) return undefined
  return poentaroWasm()
    .listPresets(source)
    .find((preset) => preset.name === presetName)?.notation
}

/** The preview's notation: the one picked in this session under the preset
 * in force, else that preset's `notation` line, else numbered. Under a preset
 * in a file you can edit, the picker rewrites the `notation` line instead
 * (`usePresetMenu`), so only a read-only view picks one. While it's a staff
 * notation, asks the worker to engrave the current source with Verovio.
 * Nothing is requested — and Verovio is never downloaded — until the
 * preview first shows staff. */
export function useStaffNotation({
  workerRef,
  source,
  visibility,
  pageLayout,
  staffRenderTracker,
  debounceMs,
}: UseStaffNotationParams): StaffNotation {
  const { requestIdRef, latestIdRef, pendingRequestsRef } = staffRenderTracker
  // Preset name → the notation picked under it; Default is `''`, a name no
  // preset can have.
  const [pickedNotations, setPickedNotations] = useState<
    Record<string, Notation>
  >({})
  const pickKey = visibility.presetName ?? ''
  const setNotation = useCallback(
    (picked: Notation) =>
      setPickedNotations((previous) => ({ ...previous, [pickKey]: picked })),
    [pickKey],
  )
  const fromPreset = useMemo(
    () => presetNotation(source, visibility.presetName),
    [source, visibility.presetName],
  )
  const notation = pickedNotations[pickKey] ?? fromPreset ?? 'numbered'
  const [score, setScore] = useState<StaffScore>(EMPTY_SCORE)
  const [loading, setLoading] = useState(false)
  const [error, setError] = useState<string | null>(null)

  useEffect(() => {
    const worker = workerRef.current
    const clef = staffClefFor(notation)
    if (!clef || !worker) return
    setLoading(true)
    const timer = window.setTimeout(() => {
      const id = ++requestIdRef.current
      latestIdRef.current = id
      pendingRequestsRef.current.set(id, (result: StaffRenderResult) => {
        setLoading(false)
        if (result.status === 'ok') {
          setScore(result.score)
          setError(null)
          return
        }
        setError(result.message)
      })
      worker.postMessage({
        type: 'renderStaff',
        source,
        id,
        visibility,
        clef,
        pageLayout,
      } satisfies WorkerRequest)
    }, debounceMs)
    return () => window.clearTimeout(timer)
  }, [
    notation,
    source,
    visibility,
    pageLayout,
    debounceMs,
    workerRef,
    requestIdRef,
    latestIdRef,
    pendingRequestsRef,
  ])

  return { notation, setNotation, score, loading, error }
}
