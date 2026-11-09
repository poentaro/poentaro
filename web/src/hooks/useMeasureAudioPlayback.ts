import type { RefObject } from 'react'
import { useCallback, useRef, useState } from 'react'
import type { NoteTiming } from '../poentaroWasm'
import type { NoteCell } from '../utils/noteSpanSelection'
import type { WorkerRequest } from '../worker/poentaro.worker'

interface UseMeasureAudioPlaybackParams {
  workerRef: RefObject<Worker | null>
  sourceRef: RefObject<string>
  enabledTracksRef: RefObject<string[] | undefined>
  renderedTracksRef: RefObject<string[] | undefined>
  selectedMeasureRange: { start: number; end: number } | null
  /**
   * Read at click time rather than depended on directly, since the selection
   * is owned by `useSequenceNavigation` in `App.tsx`, downstream of this hook
   * (which is itself owned by `usePoentaroWorker`) — a ref avoids the circular
   * dependency of threading a fresh value back into this hook's own call.
   */
  selectedSequenceRangeRef: RefObject<{
    start: number
    end: number
    entryStartIndex: number
    entryEndIndex: number
  } | null>
  /** The visible-part abbreviations a no-mounted-editor (Synced/shared
   * viewer) measure/bar-line/part-label selection touched — `undefined`
   * when there's no such restriction to apply (an editor is mounted, no
   * selection has been made, or the selection covers every visible part).
   * Read at click time by `playSelectedMeasures`, same reasoning as
   * `selectedSequenceRangeRef` above — see
   * `useAppSelectionAndNavigation`'s matching parameter doc comment. */
  measureRangeSelectedPartNamesRef: RefObject<string[] | undefined>
  /** Total measures in the score (`measureSpans.length`), used by `playAll`
   * to span from the first measure through the last written one. */
  totalMeasures: number
  /** Number of `sequence` entries (`0` when the score has none), used by
   * `playAll` to play every entry in `sequence` order. */
  sequenceEntryCount: number
}

/** Manages generating and playing back audio for a range of measures (e.g. the currently selected measures). */
export function useMeasureAudioPlayback({
  workerRef,
  sourceRef,
  enabledTracksRef,
  renderedTracksRef,
  selectedMeasureRange,
  selectedSequenceRangeRef,
  measureRangeSelectedPartNamesRef,
  totalMeasures,
  sequenceEntryCount,
}: UseMeasureAudioPlaybackParams) {
  const [measureAudioGenerating, setMeasureAudioGenerating] = useState(false)
  const [measureAudioPlaying, setMeasureAudioPlaying] = useState(false)
  const [measureAudioNoteTimings, setMeasureAudioNoteTimings] = useState<
    NoteTiming[]
  >([])
  const [measureAudioElement, setMeasureAudioElement] =
    useState<HTMLAudioElement | null>(null)
  const currentMeasureAudioRef = useRef<HTMLAudioElement | null>(null)
  const measureAudioRequestIdRef = useRef(0)
  const latestMeasureAudioIdRef = useRef(0)
  const measureWavUrlRef = useRef<string | null>(null)
  const [metronomeEnabled, setMetronomeEnabled] = useState(false)
  const metronomeEnabledRef = useRef(false)
  metronomeEnabledRef.current = metronomeEnabled

  const setNextMeasureWavUrl = useCallback(
    (next: string | null, nextNoteTimings: NoteTiming[] = []) => {
      if (currentMeasureAudioRef.current) {
        currentMeasureAudioRef.current.pause()
        currentMeasureAudioRef.current = null
      }
      if (measureWavUrlRef.current) {
        URL.revokeObjectURL(measureWavUrlRef.current)
      }
      measureWavUrlRef.current = next
      setMeasureAudioNoteTimings(nextNoteTimings)
      if (next) {
        const audio = new Audio(next)
        currentMeasureAudioRef.current = audio
        setMeasureAudioElement(audio)
        audio.addEventListener('play', () => setMeasureAudioPlaying(true))
        audio.addEventListener('ended', () => {
          setMeasureAudioPlaying(false)
          currentMeasureAudioRef.current = null
          setMeasureAudioElement(null)
        })
        audio.addEventListener('pause', () => setMeasureAudioPlaying(false))
        audio.play().catch(() => {})
      } else {
        setMeasureAudioElement(null)
      }
    },
    [],
  )

  const stopMeasurePlayback = useCallback(() => {
    if (currentMeasureAudioRef.current) {
      currentMeasureAudioRef.current.pause()
      currentMeasureAudioRef.current = null
    }
    setMeasureAudioPlaying(false)
  }, [])

  // biome-ignore lint/correctness/useExhaustiveDependencies: workerRef/sourceRef/enabledTracksRef are stable refs passed in as params
  const playMeasureRange = useCallback(
    (
      startMeasureIndex: number,
      endMeasureIndex: number,
      extendToLastOccurrence: boolean,
      respectSequence: boolean,
      sequenceEntryStartIndex?: number,
      sequenceEntryEndIndex?: number,
      /** When given, overrides the normally-enabled tracks (mute/solo state)
       * for just this playback — e.g. "play selection" muting every part
       * outside the range-selected notes. */
      enabledTracksOverride?: string[],
      /** When given, narrows the generated clip down to exactly these
       * range-selected notes' elapsed-seconds span — sample-accurately
       * trimmed and fade-cut in Rust (see `crate::wav::TrimWindow`) rather
       * than playing the whole `[startMeasureIndex, endMeasureIndex]`
       * range. Only "play selection" (`playNoteSelection`) passes this. */
      trimToSelectedNoteCells?: NoteCell[],
    ) => {
      const worker = workerRef.current
      if (!worker) return
      const id = ++measureAudioRequestIdRef.current
      latestMeasureAudioIdRef.current = id
      setMeasureAudioGenerating(true)
      worker.postMessage({
        type: 'generateMeasureRangeAudio',
        source: sourceRef.current,
        id,
        startMeasureIndex,
        endMeasureIndex,
        extendToLastOccurrence,
        respectSequence,
        sequenceEntryStartIndex,
        sequenceEntryEndIndex,
        enabledTracks: enabledTracksOverride ?? enabledTracksRef.current,
        metronome: metronomeEnabledRef.current,
        // Always the part-visibility toggle's own state (never the
        // selection override above) — see `visibleTracks`'s doc comment in
        // `worker/messages.ts`.
        visibleTracks: renderedTracksRef.current,
        trimToSelectedNoteCells,
      } satisfies WorkerRequest)
    },
    [],
  )

  // biome-ignore lint/correctness/useExhaustiveDependencies: measureRangeSelectedPartNamesRef is a stable ref read at call time, not a reactive dependency
  const playSelectedMeasures = useCallback(() => {
    if (selectedMeasureRange === null) return
    // Exact range: stop at the end measure's first occurrence, so a
    // single-measure selection (e.g. "play current measure") doesn't
    // overrun into a later D.C./D.S. al Coda repeat pass. Ignore
    // sequence/D.C./D.S. entirely, so "play current measure" always plays
    // exactly what is written, regardless of any part omission a sequence
    // entry might apply to this measure's occurrence(s).
    //
    // `enabledTracksOverride` mutes every part outside a no-mounted-editor
    // (Synced/shared viewer) part-label/measure/bar-line selection, the same
    // way the editor's "play selection" mutes every part outside a note
    // range-select (see `playNoteSelection` below) — without switching this
    // button over to that "Selection" flow, since a viewer's selection never
    // flips `notePlaybackSelectionActive` (see
    // `measureRangeSelectedPartNames`'s doc comment in
    // `useMeasureRangeSelection.ts`).
    playMeasureRange(
      selectedMeasureRange.start,
      selectedMeasureRange.end,
      false,
      false,
      undefined,
      undefined,
      measureRangeSelectedPartNamesRef.current,
    )
  }, [selectedMeasureRange, playMeasureRange])

  // biome-ignore lint/correctness/useExhaustiveDependencies: selectedSequenceRangeRef is a stable ref read at call time, not a reactive dependency
  const playFromCurrentMeasure = useCallback(() => {
    const range = selectedSequenceRangeRef.current
    if (range === null) return
    // Exact range: play only the selected `sequence` entries, stopping at
    // the end of the last one. No D.C./D.S./sequence continuation past it.
    // `respectSequence: true` so any `(-abbrev ...)` part omission on the
    // selected entry/entries is honored, not just the written score.
    // The entry index range (rather than just the written measure range)
    // disambiguates a repeated label (e.g. `A, B(-x), B`): every occurrence
    // shares the same written measure range, so without it the backend
    // always resolves to the first occurrence regardless of which one was
    // actually selected.
    playMeasureRange(
      range.start,
      range.end,
      false,
      true,
      range.entryStartIndex,
      range.entryEndIndex,
    )
  }, [playMeasureRange])

  // Plays the whole score — the same performance `generateFullAudio`
  // (Export WAV) renders, but through this hook's autoplay + cursor-sync
  // channel instead of a static downloadable player. With a `sequence`,
  // every entry is passed as an explicit entry range: resolving the written
  // first/last measures by occurrence instead would clip (or fall back to
  // written order) whenever the sequence doesn't start at the first written
  // measure or end at the last one (e.g. `B, A`).
  const playAll = useCallback(() => {
    if (totalMeasures === 0) return
    if (sequenceEntryCount > 0) {
      playMeasureRange(
        0,
        totalMeasures - 1,
        true,
        true,
        0,
        sequenceEntryCount - 1,
      )
      return
    }
    playMeasureRange(0, totalMeasures - 1, true, true)
  }, [playMeasureRange, totalMeasures, sequenceEntryCount])

  // Plays only the range-selected parts (see `useNoteSelection`), muting
  // every other part, then trims the generated clip down to exactly the
  // selected notes' elapsed-seconds span (sample-accurate trim/fade done in
  // Rust — see `crate::wav::TrimWindow`) instead of playing the selection's
  // full boundary measures.
  const playNoteSelection = useCallback(
    (
      minMeasureIndex: number,
      maxMeasureIndex: number,
      selectedPartNames: string[],
      selectedCells: NoteCell[],
    ) => {
      playMeasureRange(
        minMeasureIndex,
        maxMeasureIndex,
        false,
        false,
        undefined,
        undefined,
        selectedPartNames,
        selectedCells,
      )
    },
    [playMeasureRange],
  )

  return {
    measureAudioGenerating,
    setMeasureAudioGenerating,
    measureAudioPlaying,
    measureAudioNoteTimings,
    measureAudioElement,
    setNextMeasureWavUrl,
    stopMeasurePlayback,
    playMeasureRange,
    playSelectedMeasures,
    playFromCurrentMeasure,
    playNoteSelection,
    playAll,
    metronomeEnabled,
    setMetronomeEnabled,
    latestMeasureAudioIdRef,
    measureWavUrlRef,
  }
}
