import { readFileSync } from 'node:fs'
import { instantiate } from '../../../crates/poentaro-wasm/pkg-component/poentaro_wasm.js'
import { setWasmRoot } from '../poentaroWasm'
import { type CloudBackendConfig, createCloudBackend } from './cloudBackend'
import { baseSnapshotHooks } from './cloudBackendOutbox'
import { createFakeAppFiles, type FakeFile } from './fakeAppFiles'
import { createFileTextCache } from './fileTextCache'
import { createBaseSnapshots } from './outbox/baseSnapshots'
import { createMemoryOutboxStore } from './outbox/memoryOutboxStore'
import { createOutboxLooper } from './outbox/outboxLooper'
import { sendMessage } from './outbox/sendMessage'

const config = {
  token: 'test-token',
  account: 'test-account',
  workerHost: 'localhost:8787',
}

export async function installRealWasm(): Promise<void> {
  const bytes = readFileSync(
    new URL(
      '../../../crates/poentaro-wasm/pkg-component/poentaro_wasm.core.wasm',
      import.meta.url,
    ),
  )
  const module = await WebAssembly.compile(bytes)
  setWasmRoot(await instantiate(() => module, {}))
}

/** A cloud backend whose outbox runs on the real wasm decisions, an
 * in-memory store, a single (always-leader) tab and a manual clock: call
 * `wake()` after `advance()` to run what has become due. Its server is an
 * in-memory `poentaro app` holding `seed`. */
export function createTestBackend(
  seed: FakeFile[] = [],
  overrides: Partial<CloudBackendConfig> = {},
) {
  let time = 1_000_000
  let wake: () => void = () => undefined
  const server = createFakeAppFiles(seed)
  const store = createMemoryOutboxStore()
  const bases = createBaseSnapshots(store)
  const texts = createFileTextCache(store)
  const looper = createOutboxLooper({
    ...baseSnapshotHooks(bases, texts),
    store,
    sendMessage: (request) =>
      sendMessage(request, {
        app: server.app,
        onServerText: (fileId, revision, text) => {
          void texts.write(fileId, revision, text)
        },
      }),
    createScheduler: (onWake) => {
      wake = onWake
      return {
        wakeAt: () => undefined,
        wakeNow: onWake,
        dispose: () => undefined,
      }
    },
    runAsLeader: (task) => {
      const controller = new AbortController()
      void task(controller.signal)
      return () => controller.abort()
    },
    now: () => time,
    random: () => 0,
  })
  const backend = createCloudBackend(
    { ...config, ...overrides },
    {
      looper,
      bases,
      texts,
      app: server.app,
      ensureWasm: installRealWasm,
    },
  )
  return {
    backend,
    server,
    looper,
    bases,
    texts,
    store,
    wake: () => wake(),
    advance: (ms: number) => {
      time += ms
    },
  }
}
