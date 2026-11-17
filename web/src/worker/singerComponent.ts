import type { Root as Singer } from '../../../crates/poentaro-singer-wasm/pkg-component/poentaro_singer_wasm.js'

export type { Singer }
export type SungPiece = ReturnType<Singer['singPiece']>

/** The singing voice's wasm, compiled once and shared with every worker
 * that sings (a compiled module can be posted to a worker). */
export async function compileSinger(): Promise<WebAssembly.Module> {
  const coreWasm = await import(
    '../../../crates/poentaro-singer-wasm/pkg-component/poentaro_singer_wasm.core.wasm?url'
  )
  const response = await fetch(coreWasm.default)
  return WebAssembly.compile(await response.arrayBuffer())
}

export async function instantiateSinger(
  module: WebAssembly.Module,
): Promise<Singer> {
  const component = await import(
    '../../../crates/poentaro-singer-wasm/pkg-component/poentaro_singer_wasm.js'
  )
  return component.instantiate(() => module, {})
}
