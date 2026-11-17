import createClient from 'openapi-fetch'
import type { paths } from '../src/generated/live-share-worker/schema'

// The synthetic email + password accounts the e2e scenarios sign in as.
// Each one is a real account in the local `cf dev` worker's D1, created on
// first use through the worker's own `POST /auth/sign-up`. Every email a
// scenario uses must be listed here: the list is also the allowlist the
// worker's test-only `POST /e2e/reset` route may delete (see
// `playwright.config.ts` and `crates/live-share-worker/src/e2e_reset.rs`).
//
// Deliberately free of side effects at import time, since both
// `playwright.config.ts` and the step files import it.

/** Same origin as `cloudFileHelpers.ts`'s `CLOUD_WORKER_ORIGIN`, repeated
 * here so `playwright.config.ts` can import this module without pulling in
 * the step helpers. */
const WORKER_ORIGIN = 'http://localhost:8797'

export const DEFAULT_E2E_EMAIL = 'e2e-test-user@poentaro.test'

export const E2E_EMAILS = [
  DEFAULT_E2E_EMAIL,
  'e2e-test-user-two@poentaro.test',
  // Dedicated to `file-op-error-cloud.feature`'s failed-create scenario --
  // that scenario asserts no file named exactly "untitled" exists for the
  // signed-in account after a failed create, but every "New" click names
  // its file that same literal "untitled". Sharing the default account with
  // `files-cloud-backend.feature`'s "Creating a file persists it to the
  // cloud backend" scenario would be a cross-scenario race once
  // `fullyParallel` runs both at once.
  'e2e-test-user-create-error@poentaro.test',
  // Dedicated to the multi-device scenario, whose sign-out on one device
  // must not affect any scenario running in parallel.
  'e2e-test-user-multi-device@poentaro.test',
  // `account-auth.feature`'s accounts, one per scenario so parallel
  // scenarios never share one. Those that create an account, fail a
  // sign-in, lock it out or change its password reset it first (see
  // `resetE2eAccounts`), since that would otherwise carry over to the next
  // run. `e2e-nobody` never has an account, but its failed sign-ins still
  // count toward a lockout.
  'e2e-sign-up@poentaro.test',
  'e2e-weak-password@poentaro.test',
  'e2e-sign-in@poentaro.test',
  'e2e-wrong-password@poentaro.test',
  'e2e-nobody@poentaro.test',
  'e2e-lockout@poentaro.test',
  'e2e-change-password@poentaro.test',
  'e2e-sign-out@poentaro.test',
  // `request-trace-id.feature`'s refused sign-in.
  'e2e-trace-id@poentaro.test',
  // `session-ended.feature`, whose scenarios end their page's session.
  'e2e-session-ended@poentaro.test',
  // `agent-protocol.feature`: the key's owner, and an account whose files
  // the key must never reach.
  'e2e-agent@poentaro.test',
  'e2e-agent-other@poentaro.test',
] as const

/** The password every seeded e2e account is created with. */
export const E2E_PASSWORD = 'e2e-correct-horse-battery'

const worker = createClient<paths>({ baseUrl: WORKER_ORIGIN })

function assertKnownEmail(email: string): void {
  if (!(E2E_EMAILS as readonly string[]).includes(email)) {
    throw new Error(
      `e2eAccounts: ${JSON.stringify(email)} is not in E2E_EMAILS -- add it there so the e2e reset can clean it up`,
    )
  }
}

/** A fresh session token for `email`'s account, creating the account with
 * `E2E_PASSWORD` the first time. Every call signs in again, so each caller
 * holds its own session: one scenario signing out (which deletes only that
 * session) never invalidates another's. */
export async function signInE2eAccount(email: string): Promise<string> {
  assertKnownEmail(email)
  const body = { email, password: E2E_PASSWORD }
  const signUp = await worker.POST('/auth/sign-up', { body })
  if (signUp.data) return signUp.data.sessionToken
  if (signUp.error?.code !== 'email_taken') {
    throw new Error(
      `e2eAccounts: sign-up for ${email} failed with ${signUp.response.status}: ${JSON.stringify(signUp.error)}`,
    )
  }
  const signIn = await worker.POST('/auth/sign-in', { body })
  if (!signIn.data) {
    throw new Error(
      `e2eAccounts: sign-in for ${email} failed with ${signIn.response.status}: ${JSON.stringify(signIn.error)}`,
    )
  }
  return signIn.data.sessionToken
}

/** Deletes `emails`' accounts outright (files, shares, sessions,
 * credentials and lockouts), for scenarios that need an account that
 * doesn't exist yet, or one in a known state. */
export async function resetE2eAccounts(emails: string[]): Promise<void> {
  emails.forEach(assertKnownEmail)
  const { data, error, response } = await worker.POST('/e2e/reset', {
    body: { emails },
  })
  if (!data) {
    throw new Error(
      `e2eAccounts: reset of ${emails.join(', ')} failed with ${response.status}: ${JSON.stringify(error)}`,
    )
  }
}

/** Whether the worker still accepts `token` -- asks for the account's file
 * list, which needs a valid session. */
export async function isSessionTokenAccepted(token: string): Promise<boolean> {
  const { response } = await worker.POST('/files/list', {
    body: { identityToken: token },
  })
  if (response.status === 401) return false
  if (response.ok) return true
  throw new Error(
    `e2eAccounts: checking a session token failed with ${response.status}`,
  )
}
