import MonacoEditor, { type OnMount } from '@monaco-editor/react'
import type { editor } from 'monaco-editor'
import { type RefObject, useEffect, useMemo, useRef, useState } from 'react'
import {
  type PartToggleState,
  poentaroWasm,
  type StreamablePart,
} from '../../poentaroWasm'
import type { EditorSelection, PartFocusSelectionHandle } from '../../types'
import {
  byteOffsetToStringIndex,
  stringIndexToByteOffset,
} from '../../utils/byteSpan'
import { buildDiagnosticMarkers } from '../editorDiagnosticViewZones'
import { definePoentaroEditorTheme, EDITOR_THEME } from '../editorTheme'
import '../FileDialogs.css'
import { applyPartFocusViewZones } from './partFocusViewZones'
import './PartFocusEditor.css'
import {
  type MountedEditor,
  usePartFocusSelection,
} from './usePartFocusSelection'

const MARKER_OWNER = 'part-focus'
const FORMAT_EDIT_SOURCE = 'part-focus-format'

interface PartFocusEditorProps {
  fileId: string
  /** The main editor's source; the stream is applied onto it. */
  source: string
  part: StreamablePart
  /** The stream as extracted when the part was chosen. */
  initialStream: string
  partToggles: PartToggleState
  /** The last good draft source, so the preview can follow it; `null` once
   * the editor closes. */
  onDraftChange: (draftSource: string | null) => void
  onApply: (newSource: string) => void
  onCancel: () => void
  /** Set while mounted, so the preview's selections land in the stream. */
  handleRef: RefObject<PartFocusSelectionHandle | null>
  /** The stream selection as draft source lines, like `Editor`'s. */
  onSelectionChange: (
    startLine: number,
    endLine: number,
    isEmpty: boolean,
  ) => void
  /** The stream selection as draft source byte ranges, like `Editor`'s. */
  onSelectionOffsetChange: (ranges: EditorSelection[], isEmpty: boolean) => void
}

function changedMeasuresLabel(count: number): string {
  return `${count} ${count === 1 ? 'measure' : 'measures'} changed`
}

/**
 * Edits one part's stream in place of the main editor, one measure per
 * line, with every other part's lines around each measure as read-only
 * ghost lines. Rust re-bars the stream on every keystroke
 * (`formatPartStream`), checks it (`applyPartStream`) and supplies the
 * ghosts (`partFocusMeasures`); this renders what comes back.
 */
export function PartFocusEditor({
  fileId,
  source,
  part,
  initialStream,
  partToggles,
  onDraftChange,
  onApply,
  onCancel,
  handleRef,
  onSelectionChange,
  onSelectionOffsetChange,
}: PartFocusEditorProps) {
  const abbreviation = part.abbreviation
  const [text, setText] = useState(initialStream)
  const [mounted, setMounted] = useState<MountedEditor | null>(null)
  const viewZoneIdsRef = useRef<string[]>([])
  const prefixesRef = useRef<editor.IEditorDecorationsCollection | null>(null)
  const sourceRef = useRef(source)
  const onCancelRef = useRef(onCancel)
  useEffect(() => {
    sourceRef.current = source
    onCancelRef.current = onCancel
  })
  const { goodDraftRef, reportSelection } = usePartFocusSelection({
    mounted,
    source,
    abbreviation,
    handleRef,
    onSelectionChange,
    onSelectionOffsetChange,
  })

  // Escape cancels, unless Monaco used it (e.g. to close its find widget).
  useEffect(() => {
    const handleKeyDown = (event: KeyboardEvent) => {
      if (event.key === 'Escape' && !event.defaultPrevented) {
        onCancelRef.current()
      }
    }
    window.addEventListener('keydown', handleKeyDown)
    return () => window.removeEventListener('keydown', handleKeyDown)
  }, [])

  const result = useMemo(
    () => poentaroWasm().applyPartStream(source, abbreviation, text),
    [source, abbreviation, text],
  )
  // A draft with errors has no source; the preview keeps the last good one.
  useEffect(() => {
    if (result.source === undefined) return
    goodDraftRef.current = { stream: text, draft: result.source }
    onDraftChange(result.source)
  }, [result.source, text, onDraftChange, goodDraftRef])
  useEffect(() => () => onDraftChange(null), [onDraftChange])

  const focusMeasures = useMemo(() => {
    const response = poentaroWasm().partFocusMeasures(
      source,
      abbreviation,
      partToggles,
    )
    return response.tag === 'ok' ? response.val : []
  }, [source, abbreviation, partToggles])

  useEffect(() => {
    if (!mounted) return
    const { ed, monacoApi } = mounted
    const model = ed.getModel()
    if (!model) return
    monacoApi.editor.setModelMarkers(
      model,
      MARKER_OWNER,
      buildDiagnosticMarkers(model, monacoApi, result.diagnostics),
    )
    viewZoneIdsRef.current = applyPartFocusViewZones(
      ed,
      viewZoneIdsRef.current,
      result.measureStarts,
      focusMeasures,
    )
    prefixesRef.current?.set(
      Array.from({ length: model.getLineCount() }, (_, index) => ({
        range: new monacoApi.Range(index + 1, 1, index + 1, 1),
        options: {
          before: {
            content: `[${abbreviation}] `,
            inlineClassName: 'part-focus-prefix',
          },
        },
      })),
    )
  }, [mounted, result, focusMeasures, abbreviation])

  const handleMount: OnMount = (ed, monacoApi) => {
    const model = ed.getModel()
    if (!model) return
    model.setEOL(monacoApi.editor.EndOfLineSequence.LF)
    prefixesRef.current = ed.createDecorationsCollection()
    let formatting = false
    let composing = false

    // Re-bars the stream in place. The edit joins the keystroke's undo
    // element (no `pushUndoStop`), so one undo reverts both.
    const format = () => {
      const current = model.getValue()
      const selections = ed.getSelections() ?? []
      const formatted = poentaroWasm().formatPartStream(
        sourceRef.current,
        abbreviation,
        current,
        Uint32Array.from(selections, (selection) =>
          stringIndexToByteOffset(
            current,
            model.getOffsetAt(selection.getPosition()),
          ),
        ),
      )
      if (formatted.edits.length === 0) return
      const rangeOf = (start: number, end: number) => {
        const startPosition = model.getPositionAt(
          byteOffsetToStringIndex(current, start),
        )
        const endPosition = model.getPositionAt(
          byteOffsetToStringIndex(current, end),
        )
        return monacoApi.Range.fromPositions(startPosition, endPosition)
      }
      formatting = true
      ed.executeEdits(
        FORMAT_EDIT_SOURCE,
        formatted.edits.map((edit) => ({
          range: rangeOf(edit.span.start, edit.span.end),
          text: edit.replacement,
        })),
      )
      formatting = false
      const next = model.getValue()
      ed.setSelections(
        Array.from(formatted.cursors, (offset) => {
          const position = model.getPositionAt(
            byteOffsetToStringIndex(next, offset),
          )
          return monacoApi.Selection.fromPositions(position, position)
        }),
      )
      setText(next)
    }

    ed.onDidChangeModelContent((event) => {
      setText(model.getValue())
      if (formatting || composing || event.isUndoing || event.isRedoing) {
        return
      }
      queueMicrotask(format)
    })
    ed.onDidChangeCursorSelection(() => reportSelection(ed))
    ed.onDidCompositionStart(() => {
      composing = true
    })
    ed.onDidCompositionEnd(() => {
      composing = false
      queueMicrotask(format)
    })
    // The extracted stream keeps the source's bars; re-bar it up front, and
    // drop that from the undo history (`setValue` clears it).
    format()
    model.setValue(model.getValue())
    setMounted({ ed, monacoApi })
    ed.focus()
  }

  const appliedSource = result.source
  const canApply = appliedSource !== undefined && result.changedMeasureCount > 0
  const errorCount = result.diagnostics.filter(
    (diagnostic) => diagnostic.severity === 'error',
  ).length

  return (
    <div className="editor part-focus-editor" data-testid="part-focus-editor">
      <div className="part-focus-header">
        <span className="part-focus-title" data-testid="part-focus-title">
          Editing {part.displayName} [{abbreviation}]
        </span>
        <span
          className="part-focus-status"
          data-testid="part-focus-status"
          role="status"
        >
          {changedMeasuresLabel(result.changedMeasureCount)}
          {errorCount > 0 &&
            ` · ${errorCount} ${errorCount === 1 ? 'error' : 'errors'}`}
        </span>
        <div className="part-focus-actions">
          <button
            type="button"
            className="file-dialog-button"
            onClick={onCancel}
          >
            Cancel
          </button>
          <button
            type="button"
            className="file-dialog-button file-dialog-button--primary"
            disabled={!canApply}
            onClick={() => {
              if (appliedSource !== undefined) onApply(appliedSource)
            }}
          >
            Apply
          </button>
        </div>
      </div>
      <div className="part-focus-surface">
        <MonacoEditor
          height="100%"
          language="plaintext"
          theme={EDITOR_THEME}
          path={`part-focus://${fileId}/${abbreviation}`}
          defaultValue={initialStream}
          beforeMount={definePoentaroEditorTheme}
          onMount={handleMount}
          options={{
            minimap: { enabled: false },
            fontFamily: 'var(--mono)',
            fontSize: 14,
            lineHeight: 21,
            padding: { top: 16, bottom: 16 },
            scrollBeyondLastLine: false,
            wordWrap: 'off',
            renderLineHighlight: 'line',
            renderValidationDecorations: 'on',
            overviewRulerLanes: 2,
            hideCursorInOverviewRuler: true,
            overviewRulerBorder: false,
            glyphMargin: false,
            folding: false,
            lineNumbers: 'on',
            lineNumbersMinChars: 3,
            quickSuggestions: false,
            wordBasedSuggestions: 'off',
            scrollbar: {
              verticalScrollbarSize: 10,
              horizontalScrollbarSize: 10,
            },
          }}
        />
      </div>
    </div>
  )
}
