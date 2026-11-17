import type { SyncedShareViewerStatus } from '../hooks/useSyncedShareViewer'
import { TraceId } from './TraceId'

/** Viewer states that replace the score with a full-page message. */
export function showsSharedViewerNotice(
  status: SyncedShareViewerStatus,
): boolean {
  return noticeFor(status) !== null
}

/** What fills the page in place of the score when a synced link cannot show
 * one. The wording is neutral about file versus folder: the worker redacts a
 * stopped share completely, so the viewer cannot tell which it was. */
export function SharedViewerNotice({
  status,
  traceId,
}: {
  status: SyncedShareViewerStatus
  traceId: string | null
}) {
  const notice = noticeFor(status)
  if (!notice) return null
  return (
    <main className="shared-viewer-notice" data-testid="shared-viewer-notice">
      <h1>{notice.title}</h1>
      <p>{notice.detail}</p>
      <TraceId traceId={traceId} />
    </main>
  )
}

function noticeFor(
  status: SyncedShareViewerStatus,
): { title: string; detail: string } | null {
  switch (status) {
    case 'ended':
      return {
        title: 'This link is no longer shared',
        detail:
          'The owner stopped sharing this file or folder. Ask them for a new link, or reload to check again.',
      }
    case 'unreachable':
      return {
        title: 'Could not load this link',
        detail:
          'The shared score could not be reached. Check your connection and try reloading.',
      }
    case 'too-large':
      return {
        title: 'This shared folder is too large to open',
        detail:
          'It holds more files than a link can show. Ask the owner to share a smaller folder.',
      }
    default:
      return null
  }
}
