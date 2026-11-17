/// <reference lib="webworker" />
import {
  instantiateSinger,
  type Singer,
  type SungPiece,
} from './singerComponent'

/** Sings one piece of a song at a time, so `singer.ts` can sing a song's
 * pieces on several of these at once. */
export type SingerWorkerRequest =
  | { type: 'module'; module: WebAssembly.Module }
  | { type: 'singPiece'; id: number; plan: Uint8Array; index: number }

export type SingerWorkerResponse =
  | { id: number; piece: SungPiece }
  | { id: number; error: string }

let singer: Promise<Singer> | undefined

self.onmessage = async (event: MessageEvent<SingerWorkerRequest>) => {
  const msg = event.data
  if (msg.type === 'module') {
    singer = instantiateSinger(msg.module)
    return
  }
  try {
    if (!singer) throw new Error('the singer has no wasm module yet')
    const piece = (await singer).singPiece(msg.plan, msg.index)
    postMessage({ id: msg.id, piece } satisfies SingerWorkerResponse, {
      transfer: [piece.samples.buffer],
    })
  } catch (error) {
    postMessage({
      id: msg.id,
      error: String(error),
    } satisfies SingerWorkerResponse)
  }
}
