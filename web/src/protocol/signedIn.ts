import * as api from '../generated/protocol/api'

// Free of the wasm and the browser, so the e2e suite's Node side shares it.

/** A method of the generated `api` client that takes the session token as
 * its last parameter: the schema left it on bearer access. */
type SignedInMethod<K extends keyof api.Client> = api.Client[K] extends (
  ...args: [...infer Rest, string]
) => infer Result
  ? (...args: Rest) => Result
  : never

type SignedInMethods = {
  [K in keyof api.Client]: K extends 'batch'
    ? never
    : api.Client[K] extends (...args: infer All) => unknown
      ? All extends [...unknown[], infer Last]
        ? Last extends string
          ? K
          : never
        : never
      : never
}[keyof api.Client]

/** The endpoints a signed-in account calls, with the session token
 * already given. `batch` sends several of them as one request. */
export type AppClient = {
  [K in SignedInMethods]: SignedInMethod<K>
} & {
  batch<R>(build: (calls: AppClient) => Promise<R>[]): Promise<R[]>
}

/** `client`'s signed-in endpoints, each given `token`. Which ones take it
 * is the schema's `METHOD_PARAMETERS`, not a list kept here. */
export function signedIn(client: api.Client, token: string): AppClient {
  return new Proxy({} as AppClient, {
    get(_, name: string) {
      if (name === 'batch') {
        return <R>(build: (calls: AppClient) => Promise<R>[]) =>
          client.batch((calls) => build(signedIn(calls, token)))
      }
      const parameters = api.METHOD_PARAMETERS[name]
      if (parameters?.at(-1) !== 'token') return undefined
      const methods = client as unknown as Record<
        string,
        (...args: unknown[]) => unknown
      >
      return (...args: unknown[]) => methods[name]?.call(client, ...args, token)
    },
  })
}
