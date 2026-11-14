import type { Page, Request, Response, Route } from '@playwright/test'
import { poentaroWasm } from '../src/poentaroWasm'
import { ensureWasm, WORKER_HOST } from './protocolClients'

// Watching and faking the page's `poentaro app` calls: every one is a
// `POST /app` whose text names the method, so steps match on the method
// (read back in Rust) instead of on a URL.

/** Where the page sends `poentaro app` requests. */
export const APP_ENDPOINT = `http://${WORKER_HOST}/app`

/** One call of a request, as the generated client names it. */
export interface AppCall {
  /** The method's full name, e.g. `files/write _ at revision _ content _`. */
  method: string
  arguments: unknown[]
}

/** The calls `request` makes, or none when it is not a `poentaro app`
 * request. */
export async function appCallsOf(request: Request): Promise<AppCall[]> {
  if (request.method() !== 'POST' || request.url() !== APP_ENDPOINT) return []
  await ensureWasm()
  try {
    return JSON.parse(
      poentaroWasm().protocolDecodeRequest('app', request.postData() ?? ''),
    ) as AppCall[]
  } catch {
    return []
  }
}

/** The request's first call if it is a call of `method`. */
export async function appCallTo(
  request: Request,
  method: string,
): Promise<AppCall | undefined> {
  const [call] = await appCallsOf(request)
  return call?.method === method ? call : undefined
}

/** Sends every `poentaro app` request whose call is `method` to `handler`;
 * the rest go on to the worker (or an earlier route). Resolves to a
 * function that removes the route. */
export async function routeAppCall(
  page: Page,
  method: string,
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

/** The outcome of each call of `methods`, read from a response's text. */
export async function outcomesOf(
  methods: string[],
  text: string,
): Promise<AppOutcome[]> {
  await ensureWasm()
  return JSON.parse(
    poentaroWasm().protocolDecodeResponse('app', JSON.stringify(methods), text),
  ) as AppOutcome[]
}

/** What the worker answered a request of one call of `method`: `value`,
 * the method error's kind in words (`destinations taken`), or how the call
 * failed. */
export async function answerOf(
  method: string,
  response: Response,
): Promise<string> {
  const [outcome] = await outcomesOf([method], await response.text())
  const words = (camel: string) =>
    camel.replace(/[A-Z]/g, (letter) => ` ${letter.toLowerCase()}`)
  return outcome?.kind === 'error' && outcome.error
    ? words(outcome.error.kind)
    : words(outcome?.kind ?? 'missing')
}

/** One call's outcome, as the generated client reads it: `{ kind:
 * 'value', value }`, `{ kind: 'error', error: { kind, ... } }`, ... */
export interface AppOutcome {
  kind: string
  value?: unknown
  error?: { kind: string; [field: string]: unknown }
}

/** Answers the routed request with `outcomes`, one per call, written in
 * Rust as the worker would. */
export async function answerCalls(
  route: Route,
  outcomes: AppOutcome[],
): Promise<void> {
  await ensureWasm()
  await route.fulfill({
    status: 200,
    contentType: 'text/plain; charset=utf-8',
    body: poentaroWasm().protocolEncodeResponse(
      'app',
      route.request().postData() ?? '',
      JSON.stringify(outcomes),
    ),
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

/** Answers the routed call with the method error `kind`, e.g. `trashed`. */
export function refuseCall(route: Route, kind: string): Promise<void> {
  return answerCall(route, { kind: 'error', error: { kind } })
}

/** Lets the worker answer every request whose calls are of `method`, then
 * hands the page `rewrite`'s version of each call's answer instead. */
export function rewriteAnswers(
  page: Page,
  method: string,
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
