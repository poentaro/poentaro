import { type GenerateWavResponse, poentaroWasm } from '../poentaroWasm'
import { binaryBufferFromResult } from './exportMessageHandlers'
import type { WorkerRequest, WorkerResponse } from './poentaro.worker'

function postPreview(
  type: 'instrumentPreview' | 'percussionPreview',
  id: number,
  result: GenerateWavResponse,
): void {
  if (result.tag === 'ok') {
    const wavBuffer = binaryBufferFromResult(result.val.wav)
    postMessage({ type, id, wav: wavBuffer } satisfies WorkerResponse, {
      transfer: [wavBuffer],
    })
    return
  }
  postMessage({
    type:
      type === 'instrumentPreview'
        ? 'instrumentPreviewErr'
        : 'percussionPreviewErr',
    id,
  } satisfies WorkerResponse)
}

export function handlePreviewInstrument(
  msg: Extract<WorkerRequest, { type: 'previewInstrument' }>,
  loadedSoundfont: Uint8Array | null,
): void {
  if (!loadedSoundfont) {
    postMessage({
      type: 'instrumentPreviewErr',
      id: msg.id,
    } satisfies WorkerResponse)
    return
  }
  postPreview(
    'instrumentPreview',
    msg.id,
    poentaroWasm().generateInstrumentPreviewWav(
      msg.programNumber,
      loadedSoundfont,
    ),
  )
}

export function handlePreviewPercussion(
  msg: Extract<WorkerRequest, { type: 'previewPercussion' }>,
  loadedSoundfont: Uint8Array | null,
): void {
  if (!loadedSoundfont) {
    postMessage({
      type: 'percussionPreviewErr',
      id: msg.id,
    } satisfies WorkerResponse)
    return
  }
  postPreview(
    'percussionPreview',
    msg.id,
    poentaroWasm().generatePercussionPreviewWav(msg.key, loadedSoundfont),
  )
}
