import createClient from 'openapi-fetch'
import type { components, paths } from '../generated/live-share-worker/schema'
import { syncedShareWorkerOrigin } from './workerUrl'

/**
 * Typed client for `crates/live-share-worker`, generated from the worker's
 * own OpenAPI spec (`build:worker-types`): each route's path, method, path
 * params, request body and response come from the Rust handler's signature
 * (see `crates/live-share-worker/src/handlers/routes.rs`), so nothing here
 * or at a call site restates them by hand.
 */
export function createWorkerClient(host: string) {
  return createClient<paths>({ baseUrl: syncedShareWorkerOrigin(host) })
}

export type WorkerSchemas = components['schemas']

/** Every failure body the worker answers with, tagged by `code`, with the
 * request's `traceId` (`ApiErrorBody`). */
export type ApiError = WorkerSchemas['ApiErrorBody']

/** The response header the worker answers every request's trace ID in. */
const TRACE_ID_HEADER = 'X-Trace-Id'

/** Thrown by `callWorker` when `fetch` itself rejects (offline, DNS
 * failure, aborted request) -- distinct from a response that arrived with a
 * failure status (`WorkerRequestError` below). */
export class NetworkFailure extends Error {}

/** Thrown by `callWorker` when a success response's body isn't valid JSON
 * (e.g. truncated by a worker restarting mid-request). */
export class UnreadableResponse extends Error {}

/** Thrown by `callWorker` for any non-2xx response. `apiError` is the
 * worker's `ApiError` body, or `null` when the body isn't one at all (e.g.
 * a proxy's HTML error page) -- callers switch on `apiError?.code`.
 * `traceId` names the request in the worker's `app_log`; `null` when the
 * response never came from the worker. */
export class WorkerRequestError extends Error {
  readonly apiError: ApiError | null
  readonly response: Response
  readonly rawBody: unknown
  readonly traceId: string | null

  constructor(response: Response, body: unknown) {
    const apiError = isApiError(body) ? body : null
    const traceId =
      apiError?.traceId || response.headers.get(TRACE_ID_HEADER) || null
    super(
      `worker request failed with status ${response.status}${apiError ? ` (${apiError.code})` : ''}${traceId ? `, trace ID ${traceId}` : ''}`,
    )
    this.apiError = apiError
    this.response = response
    this.rawBody = body
    this.traceId = traceId
  }
}

/** The worker's trace ID for the request behind `error`, when it has one: a
 * request that never reached the worker (`NetworkFailure`) has none. */
export function traceIdOf(error: unknown): string | null {
  return error instanceof WorkerRequestError ? error.traceId : null
}

/** Only checks the body is `ApiError`-shaped at all (a JSON object with a
 * string `code`) -- which `code` it is, is then the generated type's job. */
function isApiError(body: unknown): body is ApiError {
  return (
    typeof body === 'object' &&
    body !== null &&
    typeof (body as { code?: unknown }).code === 'string'
  )
}

/** Awaits an openapi-fetch call, returning its success body and turning a
 * failure into `NetworkFailure`/`UnreadableResponse`/`WorkerRequestError`,
 * so every caller classifies one of those instead of an openapi-fetch
 * result. */
export async function callWorker<
  Result extends { data?: unknown; error?: unknown; response: Response },
>(pending: Promise<Result>): Promise<SuccessData<Result>> {
  let result: Result
  try {
    result = await pending
  } catch (error) {
    // openapi-fetch rejects with the `response.json()` `SyntaxError` for an
    // unparseable body, and with `fetch`'s own rejection otherwise.
    if (error instanceof SyntaxError) {
      throw new UnreadableResponse(error.message)
    }
    throw new NetworkFailure(
      error instanceof Error ? error.message : String(error),
    )
  }
  if (!result.response.ok) {
    throw new WorkerRequestError(result.response, result.error)
  }
  return result.data as SuccessData<Result>
}

/** The success arm's `data` of an openapi-fetch result union. */
type SuccessData<Result> = Result extends { error?: never; data: infer Data }
  ? Data
  : never
