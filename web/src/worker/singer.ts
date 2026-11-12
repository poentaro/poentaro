import type { Root as Singer } from '../../../crates/poentaro-singer-wasm/pkg-component/poentaro_singer_wasm.js'
import { poentaroWasm } from '../poentaroWasm'

let singerPromise: Promise<Singer> | null = null

/** The singing voice is its own wasm module, fetched the first time a score
 * sings, so scores without a voiced lyric part never download it. */
function singer(): Promise<Singer> {
  if (!singerPromise) {
    singerPromise = Promise.all([
      import(
        '../../../crates/poentaro-singer-wasm/pkg-component/poentaro_singer_wasm.js'
      ),
      import(
        '../../../crates/poentaro-singer-wasm/pkg-component/poentaro_singer_wasm.core.wasm?url'
      ),
    ])
      .then(async ([component, coreWasm]) => {
        const response = await fetch(coreWasm.default)
        const module = await WebAssembly.compile(await response.arrayBuffer())
        return component.instantiate(() => module, {})
      })
      .catch((error: unknown) => {
        singerPromise = null
        throw error
      })
  }
  return singerPromise
}

/** Recently sung plans, so playing the same score again doesn't sing it
 * again. Keyed by the plan's bytes. */
const sungPlans = new Map<string, Float32Array>()
const SUNG_PLANS_KEPT = 8

/** `plan` sung; empty when there is no plan (nothing sings). */
export async function sing(
  plan: Uint8Array | undefined,
): Promise<Float32Array> {
  if (!plan) return new Float32Array()
  const key = new TextDecoder().decode(plan)
  const cached = sungPlans.get(key)
  if (cached) return cached
  const samples = (await singer()).sing(plan)
  sungPlans.set(key, samples)
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
