// The single import surface for the wit-bindgen/jco wasm component: every
// type is re-exported straight from the jco-generated `.d.ts` (derived from
// `crates/poentaro-wasm/wit/world.wit`), and every call goes through
// `poentaroWasm()`, the component's generated `Root`. There is deliberately
// no reshaping layer in between — a field or case renamed on the Rust/WIT
// side surfaces as a type error at every consumer.
import type { Root } from '../../crates/poentaro-wasm/pkg-component/poentaro_wasm.js'

export type * from '../../crates/poentaro-wasm/pkg-component/poentaro_wasm.js'

let wasmRoot: Root | null = null

/** Set by `wasmInit.ts`/`poentaro.worker.ts` once the component is
 * instantiated on their respective thread. */
export function setWasmRoot(root: Root): void {
  wasmRoot = root
}

export function poentaroWasm(): Root {
  if (!wasmRoot) {
    throw new Error(
      'poentaroWasm: called before the wasm component finished initializing',
    )
  }
  return wasmRoot
}
