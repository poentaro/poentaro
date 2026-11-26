import type { Codec } from '../generated/protocol/test'
import { type Protocol, poentaroWasm } from '../poentaroWasm'
import { NetworkFailure, WorkerRequestError } from '../syncedShare/workerClient'
import { syncedShareWorkerOrigin } from '../syncedShare/workerUrl'
import { markSchemaOutdated } from './schemaOutdated'

/** The header a client sends its schema's hash in; the worker answers 409
 * when it serves another schema. */
export const SCHEMA_HEADER = 'Poentaro-Schema'

/** The session token names no live session. */
export class SessionEnded extends Error {}

/** The worker serves a newer schema than this tab was built from. */
export class SchemaOutdated extends Error {}

/** Encodes and decodes `protocol`'s requests in Rust. */
export function codecFor(protocol: Protocol): Codec {
  return {
    encodeRequest: (calls) =>
      poentaroWasm().protocolEncodeRequest(protocol, calls),
    decodeResponse: (methods, response) =>
      poentaroWasm().protocolDecodeResponse(protocol, methods, response),
  }
}

export interface Endpoint {
  /** The worker's host, as `VITE_SYNCED_SHARE_HOST` names it. */
  host: string
  /** The worker's path: `api`, or a protocol of `codecFor`. */
  protocol: 'api' | Protocol
  schemaHash: string
  /** Told the trace ID of each response that carries one, so a method error
   * can name its request in the worker's `app_log`. */
  onTraceId?: (traceId: string) => void
}

const TRACE_ID_HEADER = 'X-Trace-Id'

/** Sends one request to `endpoint` and returns the response text. Throws
 * `NetworkFailure` when the request never got an answer, `SessionEnded` on
 * 401, `SchemaOutdated` on 409 (and asks the app to reload), and
 * `WorkerRequestError` on any other failure status. */
export function sendTo(
  endpoint: Endpoint,
): (request: string) => Promise<string> {
  const url = `${syncedShareWorkerOrigin(endpoint.host)}/${endpoint.protocol}`
  const headers: Record<string, string> = {
    'Content-Type':
      endpoint.protocol === 'api'
        ? 'application/json'
        : 'text/plain; charset=utf-8',
    [SCHEMA_HEADER]: endpoint.schemaHash,
  }
  return async (request) => {
    let response: Response
    let body: string
    try {
      response = await fetch(url, { method: 'POST', headers, body: request })
      body = await response.text()
    } catch (error) {
      throw new NetworkFailure(
        error instanceof Error ? error.message : String(error),
      )
    }
    const traceId = response.headers.get(TRACE_ID_HEADER)
    if (traceId) endpoint.onTraceId?.(traceId)
    if (response.status === 401) throw new SessionEnded(body)
    if (response.status === 409) {
      markSchemaOutdated()
      throw new SchemaOutdated(body)
    }
    if (!response.ok) throw new WorkerRequestError(response, null)
    return body
  }
}
