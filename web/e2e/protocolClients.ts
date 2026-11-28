import { readFile } from 'node:fs/promises'
import type { Request } from '@playwright/test'
import * as api from '../src/generated/protocol/api'
import * as test from '../src/generated/protocol/test'
import { setWasmRoot } from '../src/poentaroWasm'
import { type CallOutcome, settled } from '../src/protocol/outcome'
import { type AppClient, signedIn } from '../src/protocol/signedIn'
import { codecFor, sendTo } from '../src/protocol/transport'

// The web app's protocol clients, for the Node side of the e2e suite: the
// same generated clients and transport, with the wasm codec read from disk
// instead of fetched. Free of side effects at import time.

/** The local e2e worker, as `cloudFileHelpers.ts`'s `CLOUD_WORKER_ORIGIN`
 * names it. */
export const WORKER_HOST = 'localhost:8797'

const WASM_CORE = new URL(
  '../../crates/poentaro-wasm/pkg-component/poentaro_wasm.core.wasm',
  import.meta.url,
)

let wasmReady: Promise<void> | null = null

export function ensureWasm(): Promise<void> {
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

export async function publicClient(): Promise<api.Client> {
  return new api.Client(
    sendTo({ host: WORKER_HOST, protocol: 'api', schemaHash: api.SCHEMA_HASH }),
  )
}

export async function appClient(token: string): Promise<AppClient> {
  return signedIn(await publicClient(), token)
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

interface SentRequest {
  postData(): string | null
}

/** Whether `message` (a request, or the response to one) is a protocol
 * request that calls `method`, e.g. `sign in`, for steps that
 * watch or wait on what the page sends. */
export function answersCall(
  message: SentRequest | { request(): SentRequest },
  method: string,
): boolean {
  const request = 'postData' in message ? message : message.request()
  return request.postData()?.includes(method) ?? false
}

/** The endpoint each call of an `api` request names, e.g. `open shared
 * link`: its kind on the wire. None when it isn't an `api` request. */
export function apiEndpointsOf(request: Request): string[] {
  if (
    request.method() !== 'POST' ||
    request.url() !== `http://${WORKER_HOST}/api`
  )
    return []
  try {
    const calls = JSON.parse(request.postData() ?? '') as {
      arguments?: { endpoint?: { kind?: string } }
    }[]
    return calls.flatMap((call) => call.arguments?.endpoint?.kind ?? [])
  } catch {
    return []
  }
}

/** An endpoint's name with its slots dropped: `open shared link _` is the
 * wire's `open shared link`. */
export function endpointKind(method: string): string {
  return method.replaceAll(' _', '').replace(/^_ /, '')
}

/** Whether an `api` request calls `method` (`open shared link _`). */
export async function publicCallTo(
  request: Request,
  method: string,
): Promise<boolean> {
  return apiEndpointsOf(request).includes(endpointKind(method))
}
