/** A request failed before receiving an HTTP response. */
export class NetworkFailure extends Error {}

/** A protocol endpoint returned an unexpected HTTP failure. */
export class WorkerRequestError extends Error {
  readonly traceId: string | null
  readonly response: Response
  readonly rawBody: unknown
  constructor(response: Response, rawBody: unknown) {
    const traceId = response.headers.get('X-Trace-Id')
    super(
      `worker request failed with status ${response.status}${traceId ? `, trace ID ${traceId}` : ''}`,
    )
    this.traceId = traceId
    this.response = response
    this.rawBody = rawBody
  }
}
export function traceIdOf(error: unknown): string | null {
  return error instanceof WorkerRequestError ? error.traceId : null
}
