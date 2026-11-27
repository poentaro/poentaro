import type { Page, Request, Response, Route } from '@playwright/test'
import { endpointKind, WORKER_HOST } from './protocolClients'

// Watching and faking the page's signed-in `api` calls: every one is a
// `POST /api` whose JSON names the endpoint, so steps match on the
// endpoint instead of on a URL.

/** Where the page sends `api` requests. */
export const APP_ENDPOINT = `http://${WORKER_HOST}/api`

/** An endpoint as the old protocol named it, slots included: `read file _`.
 * Matched against the wire's endpoint kind (`read file`). */
export type AppMethod = string

/** One call of a request. */
export interface AppCall {
  /** The endpoint's kind on the wire, e.g. `write file at revision content`. */
  method: string
  /** The endpoint's fields, in the schema's order. */
  arguments: unknown[]
}

interface WireCall {
  arguments?: { endpoint?: { kind?: string } & Record<string, unknown> }
}

/** The calls `request` makes, or none when it is not an `api` request. */
export async function appCallsOf(request: Request): Promise<AppCall[]> {
  if (request.method() !== 'POST' || request.url() !== APP_ENDPOINT) return []
  try {
    const calls = JSON.parse(request.postData() ?? '') as WireCall[]
    return calls.map((call) => {
      const { kind = '', ...fields } = call.arguments?.endpoint ?? {}
      return { method: kind, arguments: Object.values(fields) }
    })
  } catch {
    return []
  }
}

/** The request's first call if it is a call of `method`. */
export async function appCallTo(
  request: Request,
  method: AppMethod,
): Promise<AppCall | undefined> {
  const [call] = await appCallsOf(request)
  return call?.method === endpointKind(method) ? call : undefined
}

/** Sends every `api` request whose call is `method` to `handler`; the rest
 * go on to the worker (or an earlier route). Resolves to a function that
 * removes the route. */
export async function routeAppCall(
  page: Page,
  method: AppMethod,
  handler: (route: Route, call: AppCall) => Promise<void>,
): Promise<() => Promise<void>> {
  const routed = async (route: Route) => {
    const call = await appCallTo(route.request(), method)
    if (call) await handler(route, call)
    else await route.fallback()
  }
  await page.route(APP_ENDPOINT, routed)
  return () => page.unroute(APP_ENDPOINT, routed)
}

/** One call's outcome, in the shape steps write: `{ kind: 'value', value }`,
 * `{ kind: 'error', error: { kind, ... } }`, ... Instants are milliseconds,
 * as on the wire. */
export type AppOutcome =
  | { kind: 'value'; value: unknown }
  | { kind: 'error'; error: { kind: string } & Record<string, unknown> }
  | { kind: 'unauthenticated' }
  | { kind: 'internalError' }

type WireReply =
  | { kind: 'answered'; value: unknown }
  | { kind: 'failed'; problem: { kind: string } & Record<string, unknown> }
  | { kind: 'unauthenticated' }
  | { kind: 'internal error' }

function fromWire(reply: WireReply): AppOutcome {
  switch (reply.kind) {
    case 'answered':
      return { kind: 'value', value: reply.value }
    case 'failed':
      return { kind: 'error', error: reply.problem }
    case 'unauthenticated':
      return reply
    case 'internal error':
      return { kind: 'internalError' }
  }
}

function toWire(outcome: AppOutcome): WireReply {
  switch (outcome.kind) {
    case 'value':
      return { kind: 'answered', value: outcome.value }
    case 'error':
      return { kind: 'failed', problem: outcome.error }
    case 'unauthenticated':
      return outcome
    case 'internalError':
      return { kind: 'internal error' }
  }
}

/** The outcome of each call, read from a response's text. */
export async function outcomesOf(
  _methods: AppMethod[],
  text: string,
): Promise<AppOutcome[]> {
  return (JSON.parse(text) as WireReply[]).map(fromWire)
}

/** What the worker answered a request of one call of `method`: `value`,
 * the endpoint error's kind (`taken by other files`), or how the call
 * failed. */
export async function answerOf(
  method: AppMethod,
  response: Response,
): Promise<string> {
  const [outcome] = await outcomesOf([method], await response.text())
  if (outcome?.kind === 'error') return outcome.error.kind
  return outcome?.kind ?? 'missing'
}

/** An endpoint error's kind, e.g. `trashed`. */
export type AppRefusal = string

/** Answers the routed request with `outcomes`, one per call, as the worker
 * would. */
export async function answerCalls(
  route: Route,
  outcomes: AppOutcome[],
): Promise<void> {
  await route.fulfill({
    status: 200,
    contentType: 'application/json; charset=utf-8',
    body: JSON.stringify(outcomes.map(toWire), function (key, value) {
      const raw = (this as Record<string, unknown>)[key]
      return raw instanceof Date ? raw.getTime() : value
    }),
  })
}

/** Answers every call of the routed request with `outcome`. */
export async function answerCall(
  route: Route,
  outcome: AppOutcome,
): Promise<void> {
  const calls = await appCallsOf(route.request())
  await answerCalls(
    route,
    calls.map(() => outcome),
  )
}

/** Lets the worker answer every request whose calls are of `method`, then
 * hands the page `rewrite`'s version of each call's answer instead. */
export function rewriteAnswers(
  page: Page,
  method: AppMethod,
  rewrite: (outcome: AppOutcome, call: AppCall) => AppOutcome,
): Promise<() => Promise<void>> {
  return routeAppCall(page, method, async (route) => {
    const calls = await appCallsOf(route.request())
    const response = await route.fetch()
    const outcomes = await outcomesOf(
      calls.map((call) => call.method),
      await response.text(),
    )
    await answerCalls(
      route,
      outcomes.map((outcome, index) => {
        const call = calls[index]
        if (!call) throw new Error(`no call for outcome ${index} of ${method}`)
        return rewrite(outcome, call)
      }),
    )
  })
}
