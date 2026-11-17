import { memo } from 'react'
import type { SvgDocument } from '../poentaroWasm'
import type { StaffScore } from '../worker/messages'
import { renderSvgDocument } from './PreviewSvgRenderer'

interface PreviewPagesProps {
  documents: SvgDocument[]
  /** When set, its Verovio-engraved staff-notation pages are shown instead
   * of `documents`. */
  staffScore: StaffScore | null
}

/** One Verovio page. Memoized on the page's SVG text: React re-sets
 * `innerHTML` whenever the `dangerouslySetInnerHTML` object is new, which
 * would wipe what's painted onto the page (the staff measure highlight) on
 * every unrelated re-render. */
const StaffPage = memo(function StaffPage({ svg }: { svg: string }) {
  return (
    <div
      className="preview-page preview-page--staff"
      data-testid="staff-page"
      // biome-ignore lint/security/noDangerouslySetInnerHtml: Verovio's own SVG output, engraved from MEI that Rust escapes (`mei_export::escape`)
      dangerouslySetInnerHTML={{ __html: svg }}
    />
  )
})

export function PreviewPages({ documents, staffScore }: PreviewPagesProps) {
  if (staffScore) {
    return staffScore.pages.map((page, i) => (
      // biome-ignore lint/suspicious/noArrayIndexKey: pages have no stable identifier
      <StaffPage key={i} svg={page} />
    ))
  }
  return documents.map((doc, i) => (
    // biome-ignore lint/suspicious/noArrayIndexKey: pages have no stable identifier
    <div key={i} className="preview-page">
      {renderSvgDocument(doc, i)}
    </div>
  ))
}
