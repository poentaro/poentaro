import { type NoteTiming, poentaroWasm } from '../poentaroWasm'
import {
  binaryBufferFromResult,
  singleErrorDiagnostic,
} from './exportMessageHandlers'
import type { WorkerRequest, WorkerResponse } from './poentaro.worker'
import { sing, vocalsFor } from './singer'

function noteTimingsFromSource(
  source: string,
  visibleTracks: string[] | undefined,
  enabledTracks: string[] | undefined,
): NoteTiming[] {
  const result = poentaroWasm().listNoteTimings(
    source,
    visibleTracks,
    enabledTracks,
  )
  return result.tag === 'ok' ? result.val.timings : []
}

function noteTimingsForRangeFromSource(
  source: string,
  startMeasureIndex: number,
  endMeasureIndex: number,
  extendToLastOccurrence: boolean,
  respectSequence: boolean,
  sequenceEntryStartIndex: number | undefined,
  sequenceEntryEndIndex: number | undefined,
  visibleTracks: string[] | undefined,
  enabledTracks: string[] | undefined,
): NoteTiming[] {
  const result = poentaroWasm().listNoteTimingsForRange(
    source,
    startMeasureIndex,
    endMeasureIndex,
    extendToLastOccurrence,
    respectSequence,
    sequenceEntryStartIndex,
    sequenceEntryEndIndex,
    visibleTracks,
    enabledTracks,
  )
  return result.tag === 'ok' ? result.val.timings : []
}

export async function handleGenerateAudio(
  msg: Extract<WorkerRequest, { type: 'generateAudio' }>,
  loadedSoundfont: Uint8Array | null,
): Promise<void> {
  if (!loadedSoundfont) {
    postMessage({ type: 'audioErr', id: msg.id } satisfies WorkerResponse)
    return
  }
  let vocals: Float32Array
  try {
    vocals = await vocalsFor(msg.source, msg.enabledTracks)
  } catch {
    postMessage({ type: 'audioErr', id: msg.id } satisfies WorkerResponse)
    return
  }
  const wavResult = poentaroWasm().generateWav(
    msg.source,
    msg.enabledTracks,
    false,
    loadedSoundfont,
    vocals,
  )
  if (wavResult.tag === 'ok') {
    const wavBuffer = binaryBufferFromResult(wavResult.val.wav)
    // `msg.enabledTracks` here is always the part-visibility toggle's state
    // (never a playback-only mute override — this handler has no such
    // concept), so it doubles as `visibleTracks`. Passing it as Rust's own
    // `visible_tracks` makes `sourcePartIndex` already agree with the
    // rendered SVG's `data-part-index` (including a `MultiMeasureRest` run
    // only created once a hidden sibling part's notes are removed) — no
    // further client-side remap needed.
    const noteTimings = noteTimingsFromSource(
      msg.source,
      msg.enabledTracks,
      undefined,
    )
    postMessage(
      {
        type: 'audio',
        id: msg.id,
        wav: wavBuffer,
        noteTimings,
      } satisfies WorkerResponse,
      { transfer: [wavBuffer] },
    )
    return
  }
  postMessage({ type: 'audioErr', id: msg.id } satisfies WorkerResponse)
}

/** Shifts every timing's `startS`/`endS` back by `trimStartS`, so they
 * stay relative to the start of a clip that Rust has sample-accurately
 * trimmed down to `[trimStartS, trimEndS]` (see `crate::wav::TrimWindow`)
 * instead of the full, untrimmed measure-range clip they were originally
 * computed against. */
function shiftNoteTimings(
  timings: NoteTiming[],
  trimStartS: number,
): NoteTiming[] {
  return timings.map((t) => ({
    ...t,
    startS: t.startS - trimStartS,
    endS: t.endS - trimStartS,
  }))
}

export async function handleGenerateMeasureRangeAudio(
  msg: Extract<WorkerRequest, { type: 'generateMeasureRangeAudio' }>,
  loadedSoundfont: Uint8Array | null,
): Promise<void> {
  if (!loadedSoundfont) {
    postMessage({
      type: 'measureRangeAudioErr',
      id: msg.id,
    } satisfies WorkerResponse)
    return
  }
  // `msg.visibleTracks` (the part-visibility toggle's own state) is passed
  // as Rust's own `visible_tracks`, so `sourcePartIndex`/block structure
  // (including a `MultiMeasureRest` run only created once a hidden sibling
  // part's notes are removed) already agree with the currently rendered
  // SVG's `data-part-index`/`data-note-id` — no client-side remap needed.
  // `msg.enabledTracks` is this clip's own, possibly narrower, playback mute
  // (e.g. "play selection"), applied on top without affecting either.
  const fullRangeNoteTimings = noteTimingsForRangeFromSource(
    msg.source,
    msg.startMeasureIndex,
    msg.endMeasureIndex,
    msg.extendToLastOccurrence,
    msg.respectSequence,
    msg.sequenceEntryStartIndex,
    msg.sequenceEntryEndIndex,
    msg.visibleTracks,
    msg.enabledTracks,
  )
  // "Play selection": narrow the clip Rust synthesizes down to exactly the
  // range-selected notes' elapsed-seconds span (sample-accurate trim/fade —
  // see `crate::wav::TrimWindow`), derived from the full range's note
  // timings fetched above. `undefined` for a plain measure-range play
  // (every other caller), which always plays the range in full.
  const trim = msg.trimToSelectedNoteCells
    ? poentaroWasm().noteSelectionTrimWindow(
        msg.trimToSelectedNoteCells,
        fullRangeNoteTimings,
      )
    : undefined
  let vocals: Float32Array
  try {
    vocals = await sing(
      poentaroWasm().singingPlanForMeasureRange(
        msg.source,
        msg.startMeasureIndex,
        msg.endMeasureIndex,
        msg.extendToLastOccurrence,
        msg.respectSequence,
        msg.sequenceEntryStartIndex,
        msg.sequenceEntryEndIndex,
        msg.enabledTracks,
      ),
    )
  } catch {
    postMessage({
      type: 'measureRangeAudioErr',
      id: msg.id,
    } satisfies WorkerResponse)
    return
  }
  const wavResult = poentaroWasm().generateWavForMeasureRange(
    msg.source,
    msg.startMeasureIndex,
    msg.endMeasureIndex,
    msg.extendToLastOccurrence,
    msg.respectSequence,
    msg.sequenceEntryStartIndex,
    msg.sequenceEntryEndIndex,
    msg.enabledTracks,
    msg.metronome,
    trim?.startS,
    trim?.endS,
    trim?.nextNoteStartS,
    loadedSoundfont,
    vocals,
  )
  if (wavResult.tag === 'ok') {
    const wavBuffer = binaryBufferFromResult(wavResult.val.wav)
    postMessage(
      {
        type: 'measureRangeAudio',
        id: msg.id,
        wav: wavBuffer,
        noteTimings: trim
          ? shiftNoteTimings(fullRangeNoteTimings, trim.startS)
          : fullRangeNoteTimings,
      } satisfies WorkerResponse,
      { transfer: [wavBuffer] },
    )
    return
  }
  postMessage({
    type: 'measureRangeAudioErr',
    id: msg.id,
  } satisfies WorkerResponse)
}

/**
 * One-shot MP3 export — like [`handleGenerateAudio`], this also produces the
 * WAV preview's interactive playback cursor: MP3 gets the same inline
 * player, so it needs the same note timings. `listNoteTimings` runs off the
 * source alone (see `noteTimingsFromSource`), independent of the audio
 * codec, so this reuses it exactly as the WAV path does.
 */
export async function handleGenerateMp3(
  msg: Extract<WorkerRequest, { type: 'generateMp3' }>,
  loadedSoundfont: Uint8Array | null,
): Promise<void> {
  if (!loadedSoundfont) {
    postMessage({
      type: 'mp3Err',
      id: msg.id,
      diagnostics: singleErrorDiagnostic('Soundfont is not yet loaded.'),
    } satisfies WorkerResponse)
    return
  }
  let vocals: Float32Array
  try {
    vocals = await vocalsFor(msg.source, msg.enabledTracks)
  } catch (error) {
    postMessage({
      type: 'mp3Err',
      id: msg.id,
      diagnostics: singleErrorDiagnostic(`Could not sing the lyrics: ${error}`),
    } satisfies WorkerResponse)
    return
  }
  const result = poentaroWasm().generateMp3(
    msg.source,
    msg.enabledTracks,
    loadedSoundfont,
    vocals,
  )
  if (result.tag === 'ok') {
    const mp3Buffer = binaryBufferFromResult(result.val.mp3)
    const noteTimings = noteTimingsFromSource(
      msg.source,
      msg.enabledTracks,
      undefined,
    )
    postMessage(
      {
        type: 'mp3',
        id: msg.id,
        mp3: mp3Buffer,
        noteTimings,
      } satisfies WorkerResponse,
      { transfer: [mp3Buffer] },
    )
    return
  }
  postMessage({
    type: 'mp3Err',
    id: msg.id,
    diagnostics: result.val.diagnostics,
  } satisfies WorkerResponse)
}
