import { readFileSync } from 'node:fs'
import { beforeEach, vi } from 'vitest'
import { instantiate } from '../../../crates/poentaro-wasm/pkg-component/poentaro_wasm.js'
import { setWasmRoot } from '../poentaroWasm'
import {
  callWorker,
  createWorkerClient,
  type WorkerSchemas,
} from '../syncedShare/workerClient'
import { type CloudBackendConfig, createCloudBackend } from './cloudBackend'
import { baseSnapshotHooks } from './cloudBackendOutbox'
import { createBaseSnapshots } from './outbox/baseSnapshots'
import { createMemoryOutboxStore } from './outbox/memoryOutboxStore'
import { createOutboxLooper } from './outbox/outboxLooper'
import { sendMessage } from './outbox/sendMessage'

/** Shared `fetch` mock + response/request helpers for `cloudBackend.test.ts`
 * and `cloudBackendRetry.test.ts` -- split out so each test file stays
 * under this repo's 400-line cap while keeping identical setup. Vitest
 * isolates each test file's module graph by default, so the `fetchMock`
 * instance below is fresh per test file despite the shared import. The
 * worker client (openapi-fetch) calls `fetch` with a single `Request`. */
export const fetchMock = vi.fn<(request: Request) => Promise<Response>>()

export function jsonResponse(status: number, body: unknown): Response {
  return new Response(JSON.stringify(body), {
    status,
    headers: { 'Content-Type': 'application/json' },
  })
}

const TEST_TRACE_ID = '0123456789abcdef'

/** A worker failure response, its body checked against the generated
 * `ApiError` union. */
export function apiErrorResponse(
  status: number,
  error: WorkerSchemas['ApiError'],
): Response {
  return jsonResponse(status, { ...error, traceId: TEST_TRACE_ID })
}

export function emptyResponse(status: number): Response {
  return new Response(null, { status })
}

beforeEach(() => {
  fetchMock.mockReset()
  vi.stubGlobal('fetch', fetchMock)
})

const config = {
  token: 'test-token',
  account: 'test-account',
  workerHost: 'localhost:8787',
}

export interface RecordedCall {
  url: string
  method: string
  body: Record<string, unknown>
}

/** The `index`-th (negative counts from the end) request `fetch` got. */
export async function callAt(index: number): Promise<RecordedCall> {
  const request = fetchMock.mock.calls.at(index)?.[0]
  if (!request) throw new Error(`fetch call ${index} was not made`)
  return {
    url: request.url,
    method: request.method,
    body: (await request.clone().json()) as Record<string, unknown>,
  }
}

export function lastCall(): Promise<RecordedCall> {
  return callAt(-1)
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
 * `wake()` after `advance()` to run what has become due. */
export function createTestBackend(overrides: Partial<CloudBackendConfig> = {}) {
  let time = 1_000_000
  let wake: () => void = () => undefined
  const client = createWorkerClient(config.workerHost)
  const store = createMemoryOutboxStore()
  const bases = createBaseSnapshots(store)
  const looper = createOutboxLooper({
    ...baseSnapshotHooks(bases),
    store,
    sendMessage: (request) =>
      sendMessage(request, {
        client,
        identityToken: config.token,
        fetchServerContent: async (fileId) => {
          const response = await callWorker(
            client.POST('/files/list', {
              body: { identityToken: config.token },
            }),
          )
          return (
            response.files.find((file) => file.id === fileId)?.content ?? ''
          )
        },
        restoreName: () => 'restored.poentaro',
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
      ensureWasm: installRealWasm,
    },
  )
  return {
    backend,
    looper,
    bases,
    store,
    wake: () => wake(),
    advance: (ms: number) => {
      time += ms
    },
  }
}
