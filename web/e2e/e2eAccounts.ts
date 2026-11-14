import { SessionEnded } from '../src/protocol/transport'
import { DEFAULT_E2E_EMAIL, E2E_EMAILS } from './e2eEmails'
import {
  appClient,
  publicClient,
  testClient,
  valueOrFail,
} from './protocolClients'

export { DEFAULT_E2E_EMAIL, E2E_EMAILS }

/** The password every seeded e2e account is created with. */
export const E2E_PASSWORD = 'e2e-correct-horse-battery'

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
  const client = await publicClient()
  const signUp = await client.account.signUpPassword(email, E2E_PASSWORD)
  if (signUp.kind === 'value') return signUp.value.token
  if (signUp.kind !== 'error' || signUp.error.kind !== 'emailTaken') {
    throw new Error(
      `e2eAccounts: sign-up for ${email} failed: ${JSON.stringify(signUp)}`,
    )
  }
  const session = await valueOrFail(
    `e2eAccounts: sign-in for ${email}`,
    client.account.signInPassword(email, E2E_PASSWORD),
  )
  return session.token
}

/** Deletes `emails`' accounts outright (files, shares, sessions,
 * credentials and lockouts), for scenarios that need an account that
 * doesn't exist yet, or one in a known state. */
export async function resetE2eAccounts(emails: string[]): Promise<void> {
  emails.forEach(assertKnownEmail)
  await valueOrFail(
    `e2eAccounts: reset of ${emails.join(', ')}`,
    (await testClient()).accounts.reset(emails),
  )
}

/** Whether the worker still accepts `token` -- asks for the account's API
 * keys, which needs a live session. */
export async function isSessionTokenAccepted(token: string): Promise<boolean> {
  try {
    await valueOrFail(
      'e2eAccounts: checking a session token',
      (await appClient(token)).keys.list(),
    )
    return true
  } catch (error) {
    if (error instanceof SessionEnded) return false
    throw error
  }
}
