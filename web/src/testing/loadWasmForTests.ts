import { readFileSync } from 'node:fs'
import { instantiate } from '../../../crates/poentaro-wasm/pkg-component/poentaro_wasm.js'
import { setWasmRoot } from '../poentaroWasm'

const CORE_WASM_PATH = new URL(
  '../../../crates/poentaro-wasm/pkg-component/poentaro_wasm.core.wasm',
  import.meta.url,
)

/** Instantiates the real wasm component in Node and installs it as the
 * `poentaroWasm()` root, for unit tests of code that calls into Rust. */
export async function loadWasmForTests(): Promise<void> {
  const module = await WebAssembly.compile(readFileSync(CORE_WASM_PATH))
  setWasmRoot(await instantiate(() => module, {}))
}
