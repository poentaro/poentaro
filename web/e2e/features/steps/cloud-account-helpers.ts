import type { BrowserContext, Page } from '@playwright/test'
import { DEFAULT_E2E_EMAIL, signInE2eAccount } from '../../e2eAccounts.ts'

// Shared, non-glue helpers for the cloud (D1) storage backend's e2e
// coverage -- imported by several `.steps.ts` files across
// `account-sign-in.feature`, `files-cloud-backend.feature`,
// `cloud-backend-conflict-resolution.feature`, and the `-cloud`
// transpositions of the deleted `-github` feature files. Kept separate from
// `cloud-account.steps.ts` (which owns the actual `Given`/`When`/`Then`
// registrations) so plain functions can be imported without pulling in
// step-registration side effects.

export { DEFAULT_E2E_EMAIL }

/** The `localStorage` key `web/src/storage/accountAuth.ts` keeps the
 * signed-in session under -- hardcoded rather than imported, so the
 * Playwright process never loads the app module and its React imports. */
export const ACCOUNT_AUTH_STORAGE_KEY = 'poentaro:account-auth:v2'

// The token `seedSignedInSession` seeded most recently, for steps that check
// what the worker now thinks of the page's session (e.g. after signing out).
let lastSeededToken: string | null = null

export function lastSeededSessionToken(): string {
  if (!lastSeededToken) throw new Error('no signed-in session was seeded')
  return lastSeededToken
}

/** Signs `email` in with a session of its own and seeds it into every page
 * `target` opens from now on, the way a browser that signed in earlier
 * would hold it. Returns the seeded session token. */
export async function seedSignedInSession(
  target: Page | BrowserContext,
  email: string,
): Promise<string> {
  const token = await signInE2eAccount(email)
  lastSeededToken = token
  await target.addInitScript(
    ({ key, value }: { key: string; value: string }) => {
      localStorage.setItem(key, value)
    },
    { key: ACCOUNT_AUTH_STORAGE_KEY, value: JSON.stringify({ token, email }) },
  )
  return token
}

/** Seeds the `'cloud'` storage-backend preference (see
 * `useStorageBackend.ts`'s `StorageBackendPreference`) via `addInitScript`
 * before navigating, then loads the app. Signing in alone
 * (`accountAuth.ts`) never switches the active backend -- see
 * `useStorageBackend.ts`'s own doc comment on that being an explicit,
 * independent choice -- so every scenario that wants to load straight onto
 * the cloud-backed file list (rather than exercising the sign-in ->
 * "select Cloud storage" UI flow itself, see `account-sign-in.feature`)
 * needs to seed this preference directly, same as the deleted GitHub
 * backend's tests seeded `{backend: 'github', github: {owner}}`. */
export async function gotoCloudApp(page: Page): Promise<void> {
  await page.addInitScript(() => {
    localStorage.setItem(
      'poentaro:storage-backend:v1',
      JSON.stringify({ backend: 'cloud' }),
    )
  })
  await page.goto('/')
}
