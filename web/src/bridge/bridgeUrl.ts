import { MIGRATE_QUERY_PARAM } from '../originMigration/originMigrationPayload'

/**
 * Where the bridge sends a visitor of `current`: the same path (minus the
 * old deployment's `basePath`, e.g. GitHub Pages' `/jianpu-generator/`),
 * query and fragment on `targetOrigin`, plus the handoff id when there is
 * one. Share links carry their payload in the query and the fragment, so
 * both must survive untouched.
 */
export function bridgeRedirectUrl(
  current: URL,
  basePath: string,
  targetOrigin: string,
  migrationId: string | null,
): string {
  const target = new URL(targetOrigin)
  target.pathname = current.pathname.startsWith(basePath)
    ? `/${current.pathname.slice(basePath.length)}`
    : current.pathname
  target.search = current.search
  if (migrationId !== null) {
    target.searchParams.set(MIGRATE_QUERY_PARAM, migrationId)
  }
  target.hash = current.hash
  return target.toString()
}
