import { poentaroWasm } from '../poentaroWasm'
import type { SingerWorkerRequest, SingerWorkerResponse } from './singer.worker'
import {
  compileSinger,
  instantiateSinger,
  type Singer,
  type SungPiece,
} from './singerComponent'

type Choir = { singer: Singer; workers: Worker[] }

/** Workers that sing, at most one per spare core. */
const CHOIR_SIZE = Math.max(
  1,
  Math.min(8, (navigator.hardwareConcurrency || 2) - 1),
)

let choirPromise: Promise<Choir> | null = null

/** The singing voice is its own wasm module, fetched the first time a score
 * sings, so scores without a singing part never download it. This worker
 * keeps one instance to split and mix a song; the choir's workers sing its
 * pieces. */
function choir(): Promise<Choir> {
  if (!choirPromise) {
    choirPromise = compileSinger()
      .then(async (module) => {
        const workers = Array.from({ length: CHOIR_SIZE }, () => {
          const worker = new Worker(
            new URL('./singer.worker.ts', import.meta.url),
            { type: 'module' },
          )
          worker.postMessage({
            type: 'module',
            module,
          } satisfies SingerWorkerRequest)
          return worker
        })
        return { singer: await instantiateSinger(module), workers }
      })
      .catch((error: unknown) => {
        choirPromise = null
        throw error
      })
  }
  return choirPromise
}

let nextPieceRequestId = 0

function singPieceOn(
  worker: Worker,
  plan: Uint8Array,
  index: number,
): Promise<SungPiece> {
  const id = nextPieceRequestId++
  return new Promise((resolve, reject) => {
    const settle = () => {
      worker.removeEventListener('message', onMessage)
      worker.removeEventListener('error', onError)
    }
    const onMessage = (event: MessageEvent<SingerWorkerResponse>) => {
      if (event.data.id !== id) return
      settle()
      if ('error' in event.data) reject(new Error(event.data.error))
      else resolve(event.data.piece)
    }
    const onError = (event: ErrorEvent) => {
      settle()
      reject(new Error(event.message))
    }
    worker.addEventListener('message', onMessage)
    worker.addEventListener('error', onError)
    worker.postMessage({
      type: 'singPiece',
      id,
      plan,
      index,
    } satisfies SingerWorkerRequest)
  })
}

/** Every piece of `plan` sung, each worker taking the next piece as soon as
 * it finishes one. */
async function singPieces(
  { singer, workers }: Choir,
  plan: Uint8Array,
): Promise<SungPiece[]> {
  const count = singer.pieceCount(plan)
  const pieces: SungPiece[] = []
  let next = 0
  await Promise.all(
    workers.map(async (worker) => {
      while (next < count) {
        const index = next++
        pieces.push(await singPieceOn(worker, plan, index))
      }
    }),
  )
  return pieces
}

async function singPlan(plan: Uint8Array): Promise<Float32Array> {
  const sung = await choir()
  return sung.singer.mix(plan, await singPieces(sung, plan))
}

/** Recently sung plans (or ones being sung), so playing the same score
 * again doesn't sing it again. Keyed by the plan's bytes. */
const sungPlans = new Map<string, Promise<Float32Array>>()
const SUNG_PLANS_KEPT = 8

/** `plan` sung; empty when there is no plan (nothing sings). */
export function sing(plan: Uint8Array | undefined): Promise<Float32Array> {
  if (!plan) return Promise.resolve(new Float32Array())
  const key = new TextDecoder().decode(plan)
  const cached = sungPlans.get(key)
  if (cached) return cached
  const samples = singPlan(plan)
  sungPlans.set(key, samples)
  samples.catch(() => sungPlans.delete(key))
  if (sungPlans.size > SUNG_PLANS_KEPT) {
    const oldest = sungPlans.keys().next().value
    if (oldest !== undefined) sungPlans.delete(oldest)
  }
  return samples
}

/** What the score's singing parts sing, to mix into its audio; empty when
 * nothing sings. */
export function vocalsFor(
  source: string,
  enabledTracks: string[] | undefined,
): Promise<Float32Array> {
  return sing(poentaroWasm().singingPlan(source, enabledTracks))
}

/** Each singing track of a split audio export, sung. */
export async function splitVocalsFor(
  source: string,
  presetName: string | undefined,
): Promise<TrackVocals[]> {
  const plans = poentaroWasm().splitSingingPlans(source, presetName)
  return Promise.all(
    plans.map(async ({ trackName, plan }) => ({
      trackName,
      samples: await sing(plan),
    })),
  )
}

export type TrackVocals = { trackName: string; samples: Float32Array }
