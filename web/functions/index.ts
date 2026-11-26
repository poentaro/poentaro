// Cloudflare Pages Function for the root path. A shared-score link carries its
// share id in the `?s=` query param (`buildSyncedShareUrl`,
// `src/syncedShareUrl.ts`) rather than in a `#` fragment, because browsers
// don't send fragments in requests and a link-preview crawler (WhatsApp,
// Slack, ...) -- which never executes JS -- would otherwise only ever see the
// static shell's generic "Poentaro" title. This function reads the param, fetches
// the share's current filename from `live-share-worker`, and rewrites the
// HTML before it leaves the edge. The client reads the same param.
// (Legacy `/#synced=` links never reach this code path; they keep the
// generic preview.)
//
// Only wired up for Cloudflare Pages -- GitHub Pages is plain static hosting
// with no way to run this, so share links opened there keep the generic
// preview.

import deployConfig from '../src/generated/live-share-worker/deployConfig.json'
import shareIdFormat from '../src/generated/live-share-worker/shareIdFormat.json'
import { Client, SCHEMA_HASH } from '../src/generated/protocol/api'
import { SHARE_QUERY_PARAM } from '../src/shareQueryParam'

// Generated from `crates/live-share-worker/src/share_id.rs`
// (`share_id_format`) by `build:worker-types` -- a standalone JSON file, so
// importing it pulls nothing of the SPA's bundle into this edge function.
const SHARE_ID_LENGTH = shareIdFormat.length
const SHARE_ID_PATTERN = new RegExp(shareIdFormat.pattern)

export const onRequestGet: PagesFunction = async (context) => {
  const response = await context.next()

  const url = new URL(context.request.url)
  const payload = url.searchParams.get(SHARE_QUERY_PARAM)
  if (!payload) return response

  const shareId = payload.slice(0, SHARE_ID_LENGTH)
  if (!SHARE_ID_PATTERN.test(shareId)) return response

  const client = new Client(async (body) => {
    const result = await fetch(`https://${deployConfig.workerHost}/api`, {
      method: 'POST',
      headers: {
        'Content-Type': 'application/json',
        'Poentaro-Schema': SCHEMA_HASH,
      },
      body,
      cache: 'no-store',
    })
    if (!result.ok) throw new Error(`share preview: ${result.status}`)
    return result.text()
  })
  const outcome = await client.openSharedLink(shareId).catch(() => undefined)
  if (outcome?.kind !== 'answered' || outcome.value.kind === 'ended')
    return response
  const shared = outcome.value
  const name = shared.kind === 'file' ? shared.file.name : shared.folder.name
  const title = `${name.replace(/\.poentaro$/, '')} - Poentaro`

  // Rewrites the static shell's existing tags in place rather than
  // appending new ones: per the Open Graph spec, a parser takes the first
  // occurrence of a given property, so an appended `og:title` would be
  // shadowed by (not override) `index.html`'s default one.
  return new HTMLRewriter()
    .on('title', {
      element(element) {
        element.setInnerContent(title)
      },
    })
    .on('meta[property="og:title"]', {
      element(element) {
        element.setAttribute('content', title)
      },
    })
    .transform(response)
}
