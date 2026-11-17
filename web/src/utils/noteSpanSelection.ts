import type { Monaco } from '@monaco-editor/react'
import type * as monacoEditor from 'monaco-editor'
import type { NoteCellIn, NoteSelectionRun } from '../poentaroWasm'
import { byteOffsetToStringIndex } from './byteSpan'

/** One rendered note/rest hit-tested off the SVG during a range-select, keyed
 * the same way as the `note` `Tag`'s `data-part-index`/`data-note-id`
 * attributes. */
export type NoteCell = NoteCellIn

/** One contiguous range-selected byte range within a single part's single
 * measure, as grouped by the wasm export `groupNoteSelection`
 * (`note_spans::group_selected_notes_into_contiguous_runs` in Rust). */
export type { NoteSelectionRun }

/**
 * Converts note-selection runs into Monaco multicursor selections, reusing
 * the byte→position conversion `monacoRenameProvider.ts`'s `toRange()`
 * already proves out (`byteOffsetToStringIndex`), generalized to several
 * disjoint selections instead of one. Only `startByte`/`endByte` are read, so
 * this also serves as the shared implementation behind
 * `EditorHandle.setSelections`'s generic byte-range input.
 */
export function buildMonacoSelections(
  runs: Array<Pick<NoteSelectionRun, 'startByte' | 'endByte'>>,
  source: string,
  monacoApi: Monaco,
  model: monacoEditor.editor.ITextModel,
): monacoEditor.Selection[] {
  return runs.map((run) => {
    const startPos = model.getPositionAt(
      byteOffsetToStringIndex(source, run.startByte),
    )
    const endPos = model.getPositionAt(
      byteOffsetToStringIndex(source, run.endByte),
    )
    return new monacoApi.Selection(
      startPos.lineNumber,
      startPos.column,
      endPos.lineNumber,
      endPos.column,
    )
  })
}
