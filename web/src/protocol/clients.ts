import * as api from '../generated/protocol/api'
import * as test from '../generated/protocol/test'
import { ensureWasmInit } from '../wasmInit'
import { type AppClient, signedIn } from './signedIn'
import { codecFor, sendTo } from './transport'

export type { AppClient } from './signedIn'

/** The calls the cloud file store makes, so tests can stand in a fake. */
export type AppFileCalls = Pick<
  AppClient,
  | 'readFileAtRevision'
  | 'listVersionsOfFile'
  | 'listFiles'
  | 'readFile'
  | 'writeFileAtRevisionContent'
  | 'createFileAtContent'
  | 'moveFileTo'
  | 'trashFile'
  | 'restoreFile'
  | 'moveFolderTo'
> & {
  batch<R>(build: (calls: AppFileCalls) => Promise<R>[]): Promise<R[]>
}

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

function apiClient({
  host = workerHost(),
  onTraceId,
}: ClientOptions): api.Client {
  return new api.Client(
    sendTo({ host, protocol: 'api', schemaHash: api.SCHEMA_HASH, onTraceId }),
  )
}

/** `api`'s public endpoints: sign up, sign in, open a link, the origin
 * handoff. Calling a signed-in endpoint without a token is refused. */
export async function publicClient(
  options: ClientOptions = {},
): Promise<api.Client> {
  return apiClient(options)
}

/** `api`'s signed-in endpoints, acting as the session `token` names. */
export async function appClient(
  token: string,
  options: ClientOptions = {},
): Promise<AppClient> {
  return signedIn(apiClient(options), token)
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
