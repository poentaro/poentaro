// The synthetic email + password accounts the e2e scenarios sign in as.
// Each one is a real account in the local `cf dev` worker's D1, created on
// first use through `poentaro public`'s `account/sign up`. Every email a
// scenario uses must be listed here: the list is also the allowlist the
// worker's `poentaro test` `accounts/reset` may delete (see
// `playwright.config.ts` and `crates/live-share-worker/src/e2e_reset.rs`).
//
// A module of its own, free of imports, since `playwright.config.ts`
// imports it.

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
