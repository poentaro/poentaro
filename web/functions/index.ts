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

import createClient from 'openapi-fetch'
import deployConfig from '../src/generated/live-share-worker/deployConfig.json'
import type { paths } from '../src/generated/live-share-worker/schema'
import shareIdFormat from '../src/generated/live-share-worker/shareIdFormat.json'
import { SHARE_QUERY_PARAM } from '../src/shareQueryParam'

// Generated from `crates/live-share-worker/src/share_id.rs`
// (`share_id_format`) by `build:worker-types` -- a standalone JSON file, so
// importing it pulls nothing of the SPA's bundle into this edge function.
const SHARE_ID_LENGTH = shareIdFormat.length
const SHARE_ID_PATTERN = new RegExp(shareIdFormat.pattern)

// Typed from the worker's own OpenAPI spec (`build:worker-types`), so the
// route, its path param and the `SyncedDoc` response body all come from the
// Rust handler's signature. Only the types are imported from the generated
// schema; the client itself is `openapi-fetch`, not the SPA's
// `workerClient.ts`, to keep this edge bundle free of SPA code.
const workerClient = createClient<paths>({
  baseUrl: `https://${deployConfig.workerHost}`,
})

export const onRequestGet: PagesFunction = async (context) => {
  const response = await context.next()

  const url = new URL(context.request.url)
  const payload = url.searchParams.get(SHARE_QUERY_PARAM)
  if (!payload) return response

  const shareId = payload.slice(0, SHARE_ID_LENGTH)
  if (!SHARE_ID_PATTERN.test(shareId)) return response

  const doc = await workerClient
    .GET('/shares/{share_id}', {
      params: { path: { share_id: shareId } },
      cache: 'no-store',
    })
    .then((result) => result.data)
    .catch(() => undefined)
  if (!doc || doc.ended || !doc.filename) return response

  const title = `${doc.filename.replace(/\.poentaro$/, '')} - Poentaro`

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
