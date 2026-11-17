import type { NameMatch } from '../poentaroWasm'

/** `text` with the range Rust matched against the search query marked. */
export function HighlightedName({
  text,
  match,
}: {
  text: string
  match?: NameMatch
}) {
  if (!match) return <>{text}</>
  return (
    <>
      {text.slice(0, match.start)}
      <mark className="file-tree-match">
        {text.slice(match.start, match.end)}
      </mark>
      {text.slice(match.end)}
    </>
  )
}
