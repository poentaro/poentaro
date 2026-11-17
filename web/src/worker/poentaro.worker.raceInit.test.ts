import { describe, expect, it, vi } from 'vitest'
import { ALL_PARTS_VISIBLE } from '../hooks/workerHelpers'
import type { WorkerRequest } from './messages'

/**
 * Reproduces the production bug where the poentaro-wasm binary is fetched
 * multiple times. `ensureInit()` in poentaro.worker.ts guards on a boolean
 * that is only flipped to `true` *after* `await instantiateWasmComponentFromModule()`
 * resolves. The jco-generated `instantiate()` (crates/poentaro-wasm/pkg-component/poentaro_wasm.js)
 * has no in-flight-promise cache of its own either — it only resolves once
 * fully finished — so every worker message that reaches `ensureInit()`
 * before the first `instantiate()` call settles would trigger its own
 * independent instantiation of the component if `ensureInit()` didn't guard
 * on a shared promise.
 *
 * This covers every message type that funnels through `ensureInit()`
 * (everything except `loadSoundfont`/`loadPdfFonts`, which return before
 * reaching it), dispatched in a single burst, so the test guards the
 * invariant itself ("instantiate() runs exactly once no matter which/how
 * many messages race in first") rather than one specific message pairing.
 */

const hoisted = vi.hoisted(() => ({
  instantiateCallCount: 0,
  resolveInstantiate: undefined as (() => void) | undefined,
}))

vi.mock('../../../crates/poentaro-wasm/pkg-component/poentaro_wasm.js', () => ({
  instantiate: vi.fn(() => {
    hoisted.instantiateCallCount++
    return new Promise((resolve) => {
      hoisted.resolveInstantiate = () =>
        resolve({
          listParts: vi.fn(() => ({
            tag: 'ok',
            val: { parts: [], declarations: [] },
          })),
          renderSvg: vi.fn(() => ({
            tag: 'ok',
            val: { documents: [], diagnostics: [], diagnosticViewZones: [] },
          })),
          renderSvgWithHighlightRange: vi.fn(() => ({
            tag: 'ok',
            val: { documents: [], diagnostics: [], diagnosticViewZones: [] },
          })),
          updatePartDeclaration: vi.fn(),
          listMeasureSpans: vi.fn(() => ({
            tag: 'ok',
            val: { spans: [], sectionRanges: [], sequenceEntries: [] },
          })),
          listPartDeclarations: vi.fn(() => ({
            tag: 'ok',
            val: { declarations: [] },
          })),
          generatePdf: vi.fn(() => ({
            tag: 'ok',
            val: { pdf: new Uint8Array() },
          })),
          generateSplitPdfs: vi.fn(() => ({
            tag: 'ok',
            val: { zip: new Uint8Array() },
          })),
          generateMidi: vi.fn(() => ({
            tag: 'ok',
            val: { midi: new Uint8Array() },
          })),
          generateSplitMidis: vi.fn(() => ({
            tag: 'ok',
            val: { zip: new Uint8Array() },
          })),
          generateSplitWavs: vi.fn(() => ({
            tag: 'ok',
            val: { zip: new Uint8Array() },
          })),
          generateWav: vi.fn(() => ({
            tag: 'ok',
            val: { wav: new Uint8Array() },
          })),
          generateWavForMeasureRange: vi.fn(() => ({
            tag: 'ok',
            val: { wav: new Uint8Array() },
          })),
          listNoteTimings: vi.fn(() => ({
            tag: 'ok',
            val: { timings: [] },
          })),
          listNoteTimingsForRange: vi.fn(() => ({
            tag: 'ok',
            val: { timings: [] },
          })),
          generateInstrumentPreviewWav: vi.fn(() => ({
            tag: 'ok',
            val: { wav: new Uint8Array() },
          })),
          generatePercussionPreviewWav: vi.fn(() => ({
            tag: 'ok',
            val: { wav: new Uint8Array() },
          })),
          extractSourceFromPdf: vi.fn(),
          extractSourceFromSvg: vi.fn(),
        })
    })
  }),
}))

const raceMessages: WorkerRequest[] = [
  {
    type: 'render',
    source: '',
    id: 1,
    visibility: ALL_PARTS_VISIBLE,
    notation: 'numbered',
    pageLayout: { tag: 'a4' },
  },
  { type: 'listParts', source: '', id: 2 },
  {
    type: 'updatePartDeclaration',
    source: '',
    abbreviation: 'S',
    settings: {
      mode: { tag: 'notes' },
      program: undefined,
      volume: 100,
      octaveOffset: 0,
    },
    id: 3,
  },
  {
    type: 'generatePdf',
    source: '',
    id: 4,
    visibility: ALL_PARTS_VISIBLE,
    notation: 'numbered',
  },
  {
    type: 'generateSplitPdf',
    source: '',
    id: 5,
    baseName: 'x',
    notation: 'numbered',
  },
  { type: 'generateMidi', source: '', id: 6 },
  { type: 'generateSplitMidi', source: '', id: 7, baseName: 'x' },
  { type: 'generateSplitWav', source: '', id: 8, baseName: 'x' },
  { type: 'generateAudio', source: '', id: 9 },
  {
    type: 'generateMeasureRangeAudio',
    source: '',
    id: 10,
    startMeasureIndex: 0,
    endMeasureIndex: 0,
    extendToLastOccurrence: false,
    respectSequence: false,
    metronome: false,
  },
  {
    type: 'renderWithHighlightRange',
    source: '',
    id: 11,
    ranges: [{ start: 0, end: 0 }],
    visibility: ALL_PARTS_VISIBLE,
    notation: 'numbered',
    pageLayout: { tag: 'a4' },
  },
  { type: 'listMeasureSpans', source: '', id: 12 },
  { type: 'previewInstrument', id: 13, programNumber: 0 },
  { type: 'previewPercussion', id: 14, key: 0 },
  { type: 'importFromFile', id: 15, bytes: new ArrayBuffer(0), kind: 'svg' },
]

describe('poentaro.worker ensureInit', () => {
  it('only instantiates the wasm component once, no matter how many messages race in before it resolves', async () => {
    vi.stubGlobal('self', globalThis)
    const postMessageMock = vi.fn()
    vi.stubGlobal('postMessage', postMessageMock)

    await import('./poentaro.worker')

    const onmessage = (self as unknown as { onmessage: (e: unknown) => void })
      .onmessage

    onmessage({ data: { type: 'wasmModule', module: {} } })

    for (const data of raceMessages) {
      onmessage({ data })
    }

    // The shared ensureInit() chain (wasmModulePromise -> instantiate() ->
    // the "ready" postMessage) resolves over several microtask hops; wait
    // for instantiate() to actually fire rather than hardcoding a tick count.
    await vi.waitFor(() => expect(hoisted.instantiateCallCount).toBe(1))

    hoisted.resolveInstantiate?.()
    // Likewise, wait for the "ready" postMessage that instantiate()
    // resolving triggers before unstubbing postMessage out from under it.
    await vi.waitFor(() => expect(postMessageMock).toHaveBeenCalled())
    vi.unstubAllGlobals()
  })
})
