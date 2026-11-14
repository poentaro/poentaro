import { testClient, valueOrFail } from './protocolClients.ts'

// Wipes the synthetic e2e accounts (`E2E_EMAILS` in `e2eAccounts.ts`) and
// everything they own from the local D1 before every run.
//
// Why this exists: CI never runs this suite, but a local run reuses the
// same `crates/live-share-worker/.wrangler/e2e-state` D1 dir across every
// invocation (`playwright.config.ts`'s `reuseExistingServer`). Scenarios
// that create files under app-assigned default names
// (`files-cloud-backend.feature`'s "untitled"/"source 2"-style duplicates)
// leave real rows behind with no UI path to hard-delete them --
// `fileStore.ts`'s `reservedNames()` treats binned names as taken too -- so
// every run would reserve one more numbered name ("source 2", "source 3",
// ...) and the next run's "the active tab becomes 'source 2'" expectation
// would fall further behind until it fails outright.
//
// The delete itself is `poentaro test`'s `accounts/reset`
// (`crates/live-share-worker/src/e2e_reset.rs`), which owns the SQL and
// only touches the accounts `playwright.config.ts` allowlists for it -- so
// nothing here knows the D1 schema, and any failure (including `/test`
// being missing because a worker started without that config is being
// reused) fails the run instead of silently resetting nothing.
export default async function globalSetup() {
  const deleted = await valueOrFail(
    'e2e reset',
    (await testClient()).accounts.reset([]),
  )
  console.log(
    `e2e reset: deleted ${deleted.accounts} accounts, ${deleted.files} files and ${deleted.shares} shares`,
  )
}
