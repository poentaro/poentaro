import {
  clearSeedTokens,
  DEFAULT_CLOUD_FILE_CONTENT,
  seedCloudFile,
} from '../../cloudFileHelpers'
import { DEFAULT_E2E_EMAIL } from '../../e2eAccounts.ts'
import { seedSignedInSession } from './cloud-account-helpers'
import { BeforeScenario, Given } from './fixtures'

// Shared account sign-in + cloud-file-seeding `Given` steps, used as the
// `Background` for `account-sign-in.feature`, `files-cloud-backend.feature`,
// `cloud-backend-conflict-resolution.feature`, and every `-cloud`
// transposition of a deleted `-github` feature file. Kept in its own file
// (rather than folded into e.g. `files-cloud-backend.steps.ts`) since it's
// shared across so many feature files -- mirrors this project's existing
// convention of a dedicated steps file per reusable concern (compare
// `synced-share-button.steps.ts`'s "the owner is signed in as
// {string}").
//
// Module-level, not a fixture -- matches every other `.steps.ts` file's
// established pattern (e.g. `lastSignInPopup` in
// `synced-share-sign-in.steps.ts`). Tracks whichever email the
// current scenario's "an account is signed in as ..." step used, so a
// later "... is seeded for the signed-in account" step in the same
// scenario seeds under the same account without every feature file having
// to repeat the email in both steps.
let signedInEmail: string = DEFAULT_E2E_EMAIL

// The most recently seeded file's *display* name (extension stripped),
// tracked the same way as `signedInEmail` above. Needed because the shared
// `e2e-test-user` account is real and persistent across this whole suite
// run, not a fresh in-memory mock scoped to one scenario like the deleted
// GitHub backend's tests had -- by the time a scenario's own "the active
// file" step runs, the account can already hold many *other* scenarios'
// files too (concurrent workers, or leftover rows from earlier runs), so
// grabbing "the first tab in the list" can no longer be assumed to mean
// "the file this scenario just seeded". Steps that need to act on "the
// file I just seeded" (rename/delete/duplicate in
// `files-cloud-backend.steps.ts`) select it by this exact tracked name
// instead.
let lastSeededFileDisplayName = ''

BeforeScenario(async () => {
  signedInEmail = DEFAULT_E2E_EMAIL
  clearSeedTokens()
  lastSeededFileDisplayName = ''
})

/**
 * Pre-seeds a signed-in session (`accountAuth.ts`), bypassing the account
 * dialog, which has its own coverage in `account-auth.feature`.
 */
Given(
  'an account is signed in as {string}',
  async ({ page }, email: string) => {
    signedInEmail = email
    await seedSignedInSession(page, email)
  },
)

function trackSeededFileDisplayName(name: string): void {
  lastSeededFileDisplayName = name.replace(/\.poentaro$/, '')
}

Given(
  'a cloud file named {string} is seeded for the signed-in account',
  async ({}, name: string) => {
    await seedCloudFile(signedInEmail, name)
    trackSeededFileDisplayName(name)
  },
)

Given(
  'a cloud file named {string} with content {string} is seeded for the signed-in account',
  async ({}, name: string, content: string) => {
    await seedCloudFile(signedInEmail, name, content)
    trackSeededFileDisplayName(name)
  },
)

/** The email the current scenario's "an account is signed in as ..." step
 * captured -- exported for other `.steps.ts` files that need to seed
 * additional cloud files inline within their own `Given` (e.g. the restore-
 * collision scenario's two-file setup) rather than via the two generic
 * steps above. */
export function currentSignedInEmail(): string {
  return signedInEmail
}

/** The display name (extension stripped) of the most recently seeded file
 * -- see `lastSeededFileDisplayName`'s doc comment above for why steps that
 * act on "the active file" need to select it by this exact name rather
 * than assuming it's whichever tab happens to sort first. */
export function currentSeededFileDisplayName(): string {
  return lastSeededFileDisplayName
}

export { DEFAULT_CLOUD_FILE_CONTENT }
