import type { Monaco } from '@monaco-editor/react'
import type { editor } from 'monaco-editor'
import { type RefObject, useCallback, useEffect, useRef } from 'react'
import { poentaroWasm, type Span } from '../../poentaroWasm'
import type { EditorSelection, PartFocusSelectionHandle } from '../../types'
import {
  byteOffsetToStringIndex,
  stringIndexToByteOffset,
} from '../../utils/byteSpan'

export interface MountedEditor {
  ed: editor.IStandaloneCodeEditor
  monacoApi: Monaco
}

/** The last stream that applied cleanly, and its draft. Selections map
 * through it, so they keep working while the stream has errors. */
export interface GoodDraft {
  stream: string
  draft: string
}

type LinesChange = (
  startLine: number,
  endLine: number,
  isEmpty: boolean,
) => void
type OffsetChange = (ranges: EditorSelection[], isEmpty: boolean) => void

interface PartFocusSelectionOptions {
  mounted: MountedEditor | null
  source: string
  abbreviation: string
  /** Set while mounted, so the preview's selections land in the stream. */
  handleRef: RefObject<PartFocusSelectionHandle | null>
  onSelectionChange: LinesChange
  onSelectionOffsetChange: OffsetChange
}

/** The 1-indexed line holding byte `offset` of `text`. */
function lineAtByte(text: string, offset: number): number {
  return text.slice(0, byteOffsetToStringIndex(text, offset)).split('\n').length
}

/** Lines `startLine`..`endLine` (1-indexed, inclusive) of `text` as bytes. */
function linesToByteRange(
  text: string,
  startLine: number,
  endLine: number,
): Span {
  const lines = text.split('\n')
  const startIndex =
    lines.slice(0, startLine - 1).join('\n').length + (startLine > 1 ? 1 : 0)
  const endIndex = lines.slice(0, endLine).join('\n').length
  return {
    start: stringIndexToByteOffset(text, startIndex),
    end: stringIndexToByteOffset(text, endIndex),
  }
}

function reportStreamSelection(
  ed: editor.IStandaloneCodeEditor,
  source: string,
  abbreviation: string,
  goodDraft: GoodDraft | null,
  onSelectionChange: LinesChange,
  onSelectionOffsetChange: OffsetChange,
) {
  const model = ed.getModel()
  if (!model || !goodDraft) return
  const text = model.getValue()
  const selections = ed.getSelections() ?? []
  const isEmpty = selections.every((selection) => selection.isEmpty())
  const byteAt = (position: { lineNumber: number; column: number }) =>
    stringIndexToByteOffset(text, model.getOffsetAt(position))
  const ranges = poentaroWasm().partStreamRangesToSource(
    source,
    abbreviation,
    goodDraft.stream,
    selections.map((selection) => ({
      start: byteAt(selection.getStartPosition()),
      end: byteAt(selection.getEndPosition()),
    })),
  )
  if (ranges.length > 0) {
    onSelectionChange(
      Math.min(
        ...ranges.map((range) => lineAtByte(goodDraft.draft, range.start)),
      ),
      Math.max(
        ...ranges.map((range) => lineAtByte(goodDraft.draft, range.end)),
      ),
      isEmpty,
    )
  }
  onSelectionOffsetChange(ranges, isEmpty)
}

/**
 * Syncs the part focus editor's selection with the preview's, both mapped
 * between the stream and the draft source by Rust. Returns the ref the
 * editor records its last good draft in, and the reporter to call when its
 * cursor moves.
 */
export function usePartFocusSelection({
  mounted,
  source,
  abbreviation,
  handleRef,
  onSelectionChange,
  onSelectionOffsetChange,
}: PartFocusSelectionOptions) {
  const sourceRef = useRef(source)
  const onSelectionChangeRef = useRef(onSelectionChange)
  const onSelectionOffsetChangeRef = useRef(onSelectionOffsetChange)
  const goodDraftRef = useRef<GoodDraft | null>(null)
  useEffect(() => {
    sourceRef.current = source
    onSelectionChangeRef.current = onSelectionChange
    onSelectionOffsetChangeRef.current = onSelectionOffsetChange
  })

  // Reports the stream selection in draft terms, as `Editor` reports its
  // own.
  const reportSelection = useCallback(
    (ed: editor.IStandaloneCodeEditor) =>
      reportStreamSelection(
        ed,
        sourceRef.current,
        abbreviation,
        goodDraftRef.current,
        onSelectionChangeRef.current,
        onSelectionOffsetChangeRef.current,
      ),
    [abbreviation],
  )

  // Selects in the stream what the preview selects in the draft. Always
  // ends in exactly one selection report, which the selection hooks expect
  // to swallow as the echo of their own call.
  useEffect(() => {
    if (!mounted) return
    const { ed, monacoApi } = mounted
    const select = (ranges: Span[]) => {
      const model = ed.getModel()
      const goodDraft = goodDraftRef.current
      if (!model || !goodDraft) return
      const mapped = poentaroWasm().sourceRangesToPartStream(
        sourceRef.current,
        abbreviation,
        goodDraft.stream,
        ranges,
      )
      if (mapped.length === 0) {
        reportSelection(ed)
        return
      }
      const text = model.getValue()
      const positionAt = (offset: number) =>
        model.getPositionAt(byteOffsetToStringIndex(text, offset))
      const selections = mapped.map((span) =>
        monacoApi.Selection.fromPositions(
          positionAt(span.start),
          positionAt(span.end),
        ),
      )
      ed.setSelections(selections)
      ed.revealRangeInCenterIfOutsideViewport(selections[0])
    }
    const selectLines = (
      lineRanges: Array<{ startLine: number; endLine: number }>,
    ) => {
      const draft = goodDraftRef.current?.draft
      if (draft === undefined) return
      select(
        lineRanges.map((range) =>
          linesToByteRange(draft, range.startLine, range.endLine),
        ),
      )
    }
    handleRef.current = {
      setSelections: select,
      setSelectionByLines: (startLine, endLine) =>
        selectLines([{ startLine, endLine }]),
      setSelectionsByLines: selectLines,
      focus: () => ed.focus(),
    }
    return () => {
      handleRef.current = null
    }
  }, [mounted, abbreviation, handleRef, reportSelection])

  return { goodDraftRef, reportSelection }
}
