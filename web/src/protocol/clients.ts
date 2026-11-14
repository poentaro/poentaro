import * as app from '../generated/protocol/app'
import * as publicProtocol from '../generated/protocol/public'
import * as test from '../generated/protocol/test'
import { ensureWasmInit } from '../wasmInit'
import { codecFor, sendTo } from './transport'

/** The calls the cloud file store makes, so tests can stand in a fake. */
export type AppFileCalls = Pick<app.Client, 'files' | 'folders'>

export interface ClientOptions {
  /** The worker's host; `workerHost()` when left out. */
  host?: string
  /** Told each response's trace ID. */
  onTraceId?: (traceId: string) => void
}

/** The worker's host, as `VITE_SYNCED_SHARE_HOST` names it. */
export function workerHost(): string {
  return import.meta.env.VITE_SYNCED_SHARE_HOST ?? ''
}

/** `poentaro public`: sign up, sign in, open a link, the origin handoff.
 * Requests are encoded in Rust, so this waits for the wasm. */
export async function publicClient({
  host = workerHost(),
  onTraceId,
}: ClientOptions = {}): Promise<publicProtocol.Client> {
  await ensureWasmInit()
  return new publicProtocol.Client(
    codecFor('public'),
    sendTo({
      host,
      protocol: 'public',
      schemaHash: publicProtocol.SCHEMA_HASH,
      onTraceId,
    }),
  )
}

/** `poentaro app`, acting as the session `token` names. */
export async function appClient(
  token: string,
  { host = workerHost(), onTraceId }: ClientOptions = {},
): Promise<app.Client> {
  await ensureWasmInit()
  return new app.Client(
    codecFor('app'),
    sendTo({
      host,
      protocol: 'app',
      schemaHash: app.SCHEMA_HASH,
      token,
      onTraceId,
    }),
  )
}

/** `poentaro test`, served only to end-to-end tests. */
export async function testClient({
  host = workerHost(),
}: ClientOptions = {}): Promise<test.Client> {
  await ensureWasmInit()
  return new test.Client(
    codecFor('test'),
    sendTo({ host, protocol: 'test', schemaHash: test.SCHEMA_HASH }),
  )
}
