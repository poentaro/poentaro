import wasmComponentCoreUrl from '../../crates/poentaro-wasm/pkg-component/poentaro_wasm.core.wasm?url'
import type { Root as WasmComponentExports } from '../../crates/poentaro-wasm/pkg-component/poentaro_wasm.js'
import { instantiate as instantiateWasmComponent } from '../../crates/poentaro-wasm/pkg-component/poentaro_wasm.js'
import { setWasmRoot } from './poentaroWasm'

type WasmProgressListener = (loadedBytes: number, totalBytes: number) => void

const progressListeners = new Set<WasmProgressListener>()

/** Lets `useWasmLoader` observe download progress of the shared wasm fetch
 * below without owning the fetch itself, since `ensureWasmModule` may already
 * be in flight (or memoized) by the time the hook mounts. */
export function subscribeWasmProgress(
  listener: WasmProgressListener,
): () => void {
  progressListeners.add(listener)
  return () => progressListeners.delete(listener)
}

function reportProgress(loadedBytes: number, totalBytes: number): void {
  for (const listener of progressListeners) listener(loadedBytes, totalBytes)
}

async function fetchWithProgress(url: string): Promise<Response> {
  const response = await fetch(url)
  if (!response.body) return response

  const totalBytes = Number(response.headers.get('content-length') ?? 0)
  let loadedBytes = 0
  const progressStream = new TransformStream<Uint8Array, Uint8Array>({
    transform(chunk, controller) {
      loadedBytes += chunk.byteLength
      reportProgress(loadedBytes, totalBytes)
      controller.enqueue(chunk)
    },
  })

  return new Response(response.body.pipeThrough(progressStream), {
    headers: response.headers,
  })
}

let modulePromise: Promise<WebAssembly.Module> | null = null

/** Fetches and compiles the wit-bindgen/jco component's core wasm binary
 * exactly once, sharing a single in-flight promise across every caller (both
 * main-thread consumers and the render worker, which is handed the resolved
 * module via `postMessage`) so it's only downloaded once per page load
 * instead of once per JS realm. */
export function ensureWasmModule(): Promise<WebAssembly.Module> {
  if (!modulePromise) {
    modulePromise = fetchWithProgress(wasmComponentCoreUrl)
      .then((response) => WebAssembly.compileStreaming(response))
      .catch(async () =>
        WebAssembly.compile(
          await (await fetch(wasmComponentCoreUrl)).arrayBuffer(),
        ),
      )
  }
  return modulePromise
}

/** Instantiates the wit-bindgen/jco component from an already-compiled core
 * module: `instantiate((_) => module, {})`, no fetch of its own and an empty
 * import object (confirmed sufficient — `wasm32-unknown-unknown` needs no
 * host imports here). This is what the render worker calls once it has
 * received the module via `postMessage`; `ensureWasmInit` below calls it
 * directly on the main thread after compiling. */
export function instantiateWasmComponentFromModule(
  module: WebAssembly.Module,
): Promise<WasmComponentExports> {
  return Promise.resolve(instantiateWasmComponent(() => module, {}))
}

let wasmReady: Promise<void> | null = null

/** Initializes the `poentaro-wasm` component on the main thread, sharing a
 * single in-flight promise across every caller so concurrent first uses
 * (e.g. share-link parsing racing a rename-symbol lookup on startup) don't
 * each trigger their own fetch of the wasm binary. Every `poentaroWasm.ts`
 * function becomes callable once this resolves. */
export function ensureWasmInit(): Promise<void> {
  if (!wasmReady) {
    wasmReady = ensureWasmModule()
      .then((module) => instantiateWasmComponentFromModule(module))
      .then((root) => {
        setWasmRoot(root)
      })
  }
  return wasmReady
}
