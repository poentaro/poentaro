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

/** What the score's voiced lyric parts sing, to mix into its audio; empty
 * when nothing sings. */
export async function vocalsFor(
  source: string,
  enabledTracks: string[] | undefined,
): Promise<Float32Array> {
  const plan = poentaroWasm().singingPlan(source, enabledTracks)
  if (!plan) return new Float32Array()
  return (await singer()).sing(plan)
}
