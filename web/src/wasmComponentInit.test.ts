import { describe, expect, it, vi } from 'vitest'

/**
 * `ensureWasmModule`/`instantiateWasmComponentFromModule` (`wasmInit.ts`)
 * are the wit-bindgen/jco component's fetch-and-compile-once and
 * instantiate-from-an-already-compiled-module primitives (PLAN-wit-bindgen-migration.md
 * Phase 4/6) — this exercises them directly against the mocked jco-generated
 * glue module, complementing `mainThreadWasmInit.race.test.ts`'s cross-file
 * racing coverage.
 */

const hoisted = vi.hoisted(() => ({
  instantiateCallCount: 0,
  lastGetCoreModule: undefined as ((path: string) => unknown) | undefined,
  lastImports: undefined as unknown,
}))

vi.mock('../../crates/poentaro-wasm/pkg-component/poentaro_wasm.js', () => ({
  instantiate: vi.fn(
    (getCoreModule: (path: string) => unknown, imports: unknown) => {
      hoisted.instantiateCallCount++
      hoisted.lastGetCoreModule = getCoreModule
      hoisted.lastImports = imports
      return { greet: vi.fn(() => 'hello') }
    },
  ),
}))

describe('wasm component init', () => {
  it('fetches and compiles the component core module exactly once across concurrent callers', async () => {
    const fetchMock = vi.fn(() => Promise.resolve(new Response()))
    vi.stubGlobal('fetch', fetchMock)
    const fakeModule = {}
    const compileStreamingMock = vi.fn(() => Promise.resolve(fakeModule))
    vi.stubGlobal('WebAssembly', {
      ...WebAssembly,
      compileStreaming: compileStreamingMock,
    })

    const { ensureWasmModule, instantiateWasmComponentFromModule } =
      await import('./wasmInit')

    const [moduleA, moduleB] = await Promise.all([
      ensureWasmModule(),
      ensureWasmModule(),
    ])

    expect(fetchMock).toHaveBeenCalledTimes(1)
    expect(compileStreamingMock).toHaveBeenCalledTimes(1)
    expect(moduleA).toBe(fakeModule)
    expect(moduleB).toBe(fakeModule)

    const exports = await instantiateWasmComponentFromModule(
      moduleA as unknown as WebAssembly.Module,
    )

    expect(hoisted.instantiateCallCount).toBe(1)
    // No host imports needed on `wasm32-unknown-unknown` — confirmed by spike,
    // see PLAN-wit-bindgen-migration.md's "Resolved risks" section.
    expect(hoisted.lastImports).toEqual({})
    // `getCoreModule` must resolve to the one already-compiled module, not
    // trigger a second fetch/compile of its own.
    expect(hoisted.lastGetCoreModule?.('poentaro_wasm.core.wasm')).toBe(
      fakeModule,
    )
    expect(exports.greet('world')).toBe('hello')

    vi.unstubAllGlobals()
  })

  it('instantiates directly from an already-compiled module with no fetch of its own', async () => {
    hoisted.instantiateCallCount = 0
    const fetchMock = vi.fn(() => Promise.resolve(new Response()))
    vi.stubGlobal('fetch', fetchMock)

    const { instantiateWasmComponentFromModule } = await import('./wasmInit')
    const receivedModule = {}

    const exports = await instantiateWasmComponentFromModule(
      receivedModule as unknown as WebAssembly.Module,
    )

    expect(fetchMock).not.toHaveBeenCalled()
    expect(hoisted.instantiateCallCount).toBe(1)
    expect(hoisted.lastImports).toEqual({})
    expect(hoisted.lastGetCoreModule?.('poentaro_wasm.core.wasm')).toBe(
      receivedModule,
    )
    expect(exports.greet('world')).toBe('hello')

    vi.unstubAllGlobals()
  })
})
