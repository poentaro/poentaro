import { readFile } from 'node:fs/promises'
import * as app from '../src/generated/protocol/app'
import * as publicProtocol from '../src/generated/protocol/public'
import * as test from '../src/generated/protocol/test'
import { setWasmRoot } from '../src/poentaroWasm'
import { type CallOutcome, settled } from '../src/protocol/outcome'
import { codecFor, sendTo } from '../src/protocol/transport'

// The web app's protocol clients, for the Node side of the e2e suite: the
// same generated clients and transport, with the wasm codec read from disk
// instead of fetched. Free of side effects at import time.

/** The local e2e worker, as `cloudFileHelpers.ts`'s `CLOUD_WORKER_ORIGIN`
 * names it. */
const WORKER_HOST = 'localhost:8797'

const WASM_CORE = new URL(
  '../../crates/poentaro-wasm/pkg-component/poentaro_wasm.core.wasm',
  import.meta.url,
)

let wasmReady: Promise<void> | null = null

function ensureWasm(): Promise<void> {
  if (!wasmReady) {
    // Imported here, not at the top, so `playwright.config.ts` (which
    // imports `e2eAccounts.ts`) never loads the component.
    wasmReady = Promise.all([
      import('../../crates/poentaro-wasm/pkg-component/poentaro_wasm.js'),
      readFile(WASM_CORE).then((bytes) => WebAssembly.compile(bytes)),
    ])
      .then(([component, module]) => component.instantiate(() => module, {}))
      .then(setWasmRoot)
  }
  return wasmReady
}

export async function publicClient(): Promise<publicProtocol.Client> {
  await ensureWasm()
  return new publicProtocol.Client(
    codecFor('public'),
    sendTo({
      host: WORKER_HOST,
      protocol: 'public',
      schemaHash: publicProtocol.SCHEMA_HASH,
    }),
  )
}

export async function appClient(token: string): Promise<app.Client> {
  await ensureWasm()
  return new app.Client(
    codecFor('app'),
    sendTo({
      host: WORKER_HOST,
      protocol: 'app',
      schemaHash: app.SCHEMA_HASH,
      token,
    }),
  )
}

export async function testClient(): Promise<test.Client> {
  await ensureWasm()
  return new test.Client(
    codecFor('test'),
    sendTo({
      host: WORKER_HOST,
      protocol: 'test',
      schemaHash: test.SCHEMA_HASH,
    }),
  )
}

/** `pending`'s value; a method error fails the step, naming `what`. */
export async function valueOrFail<T, E>(
  what: string,
  pending: Promise<CallOutcome<T, E>>,
): Promise<T> {
  const outcome = settled(await pending)
  if (outcome.kind === 'error') {
    throw new Error(`${what} failed: ${JSON.stringify(outcome.error)}`)
  }
  return outcome.value
}

/** Whether `response` answers a protocol request that calls `method`, e.g.
 * `account/sign in`, for steps that wait on a request the page sends. */
export function answersCall(
  response: { request(): { postData(): string | null } },
  method: string,
): boolean {
  return response.request().postData()?.includes(method) ?? false
}
