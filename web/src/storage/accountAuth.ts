import { useLocalStorage } from 'usehooks-ts'

/**
 * The browser's account session: the opaque session token the worker
 * handed back from `POST /auth/sign-up` or `POST /auth/sign-in`, sent as
 * `identityToken` on every account request (cloud files, Synced Share), and
 * the account's email for the header chip. The worker owns every auth rule
 * (password length, lockout, session expiry); this module only stores what
 * it answered. `AccountAuthModal` writes it, signing out
 * (`useSyncedShareOwner`'s `signOut`) clears it, and a request the worker
 * refuses as `unauthorized` clears it too.
 */
const ACCOUNT_AUTH_STORAGE_KEY = 'poentaro:account-auth:v2'

/** Where GitHub sign-in kept its token. Removed on load: that token means
 * nothing to the worker any more. */
const LEGACY_GITHUB_AUTH_STORAGE_KEY = 'poentaro:synced-share-github-auth:v1'

/** A signed-in session. `email` is only for display; the worker identifies
 * the account from `token` on every request. */
export interface StoredAccountAuth {
  token: string
  email: string
}

function removeLegacyGithubAuth(): void {
  try {
    localStorage.removeItem(LEGACY_GITHUB_AUTH_STORAGE_KEY)
  } catch {
    // Storage unavailable: there's nothing to remove either.
  }
}

removeLegacyGithubAuth()

/** Reactive accessor for the stored session, so the header chip, the share
 * dialog and the storage dialog all re-render the moment it changes. */
export function useAccountAuth() {
  return useLocalStorage<StoredAccountAuth | null>(
    ACCOUNT_AUTH_STORAGE_KEY,
    null,
  )
}
