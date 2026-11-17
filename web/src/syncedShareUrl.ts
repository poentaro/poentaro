import { FILE_EXTENSION, stripFileExtension } from './fileExtension'
import shareIdFormat from './generated/live-share-worker/shareIdFormat.json'
import { SHARE_QUERY_PARAM } from './shareQueryParam'

// Legacy link format (`/#synced=<payload>`), still parsed so links shared
// before the move to `?s=` keep working. Never generated any more.
const LEGACY_SYNCED_HASH_PREFIX = '#synced='

// Generated from `crates/live-share-worker/src/share_id.rs`
// (`share_id_format`) by `build:worker-types` -- the worker mints every
// share id in exactly this shape, and this is its only definition in TS.
const SHARE_ID_LENGTH = shareIdFormat.length
const SHARE_ID_PATTERN = new RegExp(shareIdFormat.pattern)

// Purely cosmetic — separates the share id from the filename so a copied
// link reads clearly. Parsing doesn't need it to find the boundary (the
// share id's length is fixed, see above), so the filename after it is never
// escaped and may contain any character, including further dashes.
const FILENAME_SEPARATOR = '--'

export interface SyncedSharePayload {
  shareId: string
  /** Filename at share-time, carried in the URL purely so the link reads as
   * something a human can recognize (in chat, browser history, etc). Never
   * authoritative — the share's stored doc is the source of truth, so a
   * rename after sharing doesn't invalidate the link, it just makes this
   * cosmetic copy stale. Whitespace in it was replaced by `-` when the link
   * was built. */
  filename?: string
}

export function buildSyncedShareUrl(
  shareId: string,
  filename?: string,
): string {
  const base = new URL(import.meta.env.BASE_URL, window.location.origin)
  const bareName =
    filename === undefined ? undefined : stripFileExtension(filename)
  // Whitespace becomes `-` so a chat app (e.g. WhatsApp) doesn't end the
  // auto-linked URL at the first space.
  const dashedName = bareName?.trim().replace(/\s+/g, '-')
  const namePart = dashedName ? `${FILENAME_SEPARATOR}${dashedName}` : ''
  // Built by hand rather than via `searchParams.set`, which would
  // percent-encode every non-ASCII character and make the filename unreadable
  // in a chat app whose link preview fails. Only the characters that would
  // break the query string are escaped.
  const payload = `${shareId}${namePart}`.replace(
    /[%&#+]/g,
    (char) => `%${char.charCodeAt(0).toString(16).toUpperCase()}`,
  )
  return `${base.origin}${base.pathname}?${SHARE_QUERY_PARAM}=${payload}`
}

function parsePayload(payload: string): SyncedSharePayload | null {
  const shareId = payload.slice(0, SHARE_ID_LENGTH)
  if (!SHARE_ID_PATTERN.test(shareId)) return null
  const rest = payload.slice(SHARE_ID_LENGTH)
  if (rest === '') return { shareId }
  if (!rest.startsWith(FILENAME_SEPARATOR)) return null
  const filename = `${rest.slice(FILENAME_SEPARATOR.length)}${FILE_EXTENSION}`
  return { shareId, filename }
}

/** Reads a synced-share link: the `?s=` query param, falling back to the
 * legacy `#synced=` hash. */
export function parseSyncedShare(
  search: string = window.location.search,
  hash: string = window.location.hash,
): SyncedSharePayload | null {
  const queryPayload = new URLSearchParams(search).get(SHARE_QUERY_PARAM)
  if (queryPayload !== null) return parsePayload(queryPayload)
  if (!hash.startsWith(LEGACY_SYNCED_HASH_PREFIX)) return null
  return parsePayload(hash.slice(LEGACY_SYNCED_HASH_PREFIX.length))
}

/** Drops the synced-share reference (`?s=` and the legacy `#synced=` hash)
 * from the address bar, keeping any other query params such as `?file=`. */
export function clearSyncedShare(): void {
  const url = new URL(window.location.href)
  url.hash = ''
  url.searchParams.delete(SHARE_QUERY_PARAM)
  const search = url.searchParams.toString()
  history.replaceState(null, '', `${url.pathname}${search ? `?${search}` : ''}`)
}
