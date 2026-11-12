import MonacoEditor, { type Monaco, type OnMount } from '@monaco-editor/react'
import type { editor, IDisposable } from 'monaco-editor'
import {
  forwardRef,
  type ReactNode,
  useCallback,
  useEffect,
  useImperativeHandle,
  useRef,
} from 'react'
import {
  POENTARO_LANGUAGE_ID,
  registerPoentaroLanguage,
} from '../monacoPoentaroLanguage'
import { registerPoentaroRenameProvider } from '../monacoRenameProvider'
import { registerSectionCodeLensProvider } from '../monacoSectionCodeLensProvider'
import { registerPoentaroSemanticTokensProvider } from '../monacoSemanticTokensProvider'
import type {
  Diagnostic,
  DiagnosticViewZone,
  EditorHandle,
  EditorSelection,
  MeasureSpan,
} from '../types'
import { stringIndexToByteOffset } from '../utils/byteSpan'
import {
  buildDiagnosticMarkers,
  createDiagnosticViewZoneDomNode,
  errorViewZoneHeightInPx,
} from './editorDiagnosticViewZones'
import { createEditorImperativeHandle } from './editorImperativeHandle'
import {
  createMeasureViewZoneDomNode,
  measureViewZoneLineNumber,
} from './editorMeasureViewZones'
import { definePoentaroEditorTheme, EDITOR_THEME } from './editorTheme'
import { useEditorValueSync } from './useEditorValueSync'

export interface EditorProps {
  /** Unique per-file ID; gives each file its own Monaco model and undo stack. */
  path?: string
  value: string
  onChange: (value: string) => void
  readOnly?: boolean
  diagnostics?: Diagnostic[]
  diagnosticViewZones?: DiagnosticViewZone[]
  measureSpans?: MeasureSpan[]
  toolbar?: ReactNode
  /** The line span of every selection (multicursor included). `isEmpty` is
   * true when every selection is a plain caret (0-length) rather
   * than a highlighted range — callers use this to gate the preview's
   * measure-background highlight, which should only show for a bare caret. */
  onSelectionChange?: (
    startLine: number,
    endLine: number,
    isEmpty: boolean,
  ) => void
  /** Same selection as `onSelectionChange`, but as UTF-8 byte offsets — one
   * entry per selection in Monaco's current (possibly multicursor)
   * selection set, via `ed.getSelections()`/`stringIndexToByteOffset` — used
   * by callers whose measure mapping is byte-offset-based rather than
   * line-based (see `useNoteSelection`'s `handleEditorSelectionChange`) and
   * by the "shift selection octave" toolbar action, which needs every
   * disjoint piece of a multicursor selection (e.g. one produced by
   * clicking a part label), not just the primary one. `isEmpty` is true
   * only when every selection in the set is empty. */
  onSelectionOffsetChange?: (
    ranges: EditorSelection[],
    isEmpty: boolean,
  ) => void
  onCursorLineChange?: (line: number) => void
  onPlayMeasure?: () => void
  onForceSave?: () => void
  onEditPartsClick?: () => void
  onEditMetadataClick?: () => void
}

const MARKER_OWNER = 'poentaro'

export const Editor = forwardRef<EditorHandle, EditorProps>(function Editor(
  {
    path,
    value,
    onChange,
    readOnly = false,
    diagnostics = [],
    diagnosticViewZones = [],
    measureSpans = [],
    toolbar,
    onSelectionChange,
    onSelectionOffsetChange,
    onCursorLineChange,
    onPlayMeasure,
    onForceSave,
    onEditPartsClick,
    onEditMetadataClick,
  },
  ref,
) {
  const editorRef = useRef<editor.IStandaloneCodeEditor | null>(null)
  const monacoRef = useRef<Monaco | null>(null)
  const measureViewZoneIdsRef = useRef<string[]>([])
  const diagnosticViewZoneIdsRef = useRef<string[]>([])
  const onSelectionChangeRef = useRef(onSelectionChange)
  const onSelectionOffsetChangeRef = useRef(onSelectionOffsetChange)
  const onCursorLineChangeRef = useRef(onCursorLineChange)
  const onPlayMeasureRef = useRef(onPlayMeasure)
  const onForceSaveRef = useRef(onForceSave)
  const onEditPartsClickRef = useRef(onEditPartsClick)
  const onEditMetadataClickRef = useRef(onEditMetadataClick)
  const codeLensProviderRef = useRef<IDisposable | null>(null)
  const syncModel = useEditorValueSync(editorRef, value)
  useEffect(() => {
    onSelectionChangeRef.current = onSelectionChange
    onSelectionOffsetChangeRef.current = onSelectionOffsetChange
    onCursorLineChangeRef.current = onCursorLineChange
    onPlayMeasureRef.current = onPlayMeasure
    onForceSaveRef.current = onForceSave
    onEditPartsClickRef.current = onEditPartsClick
    onEditMetadataClickRef.current = onEditMetadataClick
  })

  const applyDiagnostics = useCallback(() => {
    const ed = editorRef.current
    const monacoApi = monacoRef.current
    const model = ed?.getModel()
    if (!ed || !monacoApi || !model) return

    const markers = buildDiagnosticMarkers(model, monacoApi, diagnostics)
    monacoApi.editor.setModelMarkers(model, MARKER_OWNER, markers)
  }, [diagnostics])

  const applyMeasureViewZones = useCallback(() => {
    const ed = editorRef.current
    const model = ed?.getModel()
    if (!ed || !model) return

    ed.changeViewZones((accessor) => {
      for (const id of measureViewZoneIdsRef.current) {
        accessor.removeZone(id)
      }
      measureViewZoneIdsRef.current = []

      const source = model.getValue()

      measureSpans.forEach((span, index) => {
        const lineNumber = measureViewZoneLineNumber(model, source, span)
        const domNode = createMeasureViewZoneDomNode(
          span.sectionLabel ?? null,
          index,
        )

        const id = accessor.addZone({
          afterLineNumber: lineNumber - 1,
          heightInLines: 1,
          domNode,
        })
        measureViewZoneIdsRef.current.push(id)
      })
    })
  }, [measureSpans])

  const applyDiagnosticViewZones = useCallback(() => {
    const ed = editorRef.current
    const model = ed?.getModel()
    if (!ed || !model) return

    ed.changeViewZones((accessor) => {
      for (const id of diagnosticViewZoneIdsRef.current) {
        accessor.removeZone(id)
      }
      diagnosticViewZoneIdsRef.current = []

      for (const zone of diagnosticViewZones) {
        const domNode = createDiagnosticViewZoneDomNode(
          zone.severity,
          zone.messages,
        )
        const heightInPx = errorViewZoneHeightInPx(
          domNode,
          ed.getLayoutInfo().contentWidth,
        )
        const id = accessor.addZone({
          afterLineNumber: zone.afterLineNumber,
          heightInPx,
          domNode,
        })
        diagnosticViewZoneIdsRef.current.push(id)
      }
    })
  }, [diagnosticViewZones])

  useImperativeHandle(
    ref,
    () => createEditorImperativeHandle(editorRef, monacoRef),
    [],
  )

  useEffect(() => {
    return () => {
      codeLensProviderRef.current?.dispose()
    }
  }, [])

  const handleMount: OnMount = (ed, monacoApi) => {
    editorRef.current = ed
    monacoRef.current = monacoApi
    applyDiagnostics()
    applyMeasureViewZones()
    applyDiagnosticViewZones()
    ed.onDidChangeModel(syncModel)

    ed.addCommand(monacoApi.KeyMod.CtrlCmd | monacoApi.KeyCode.Enter, () =>
      onPlayMeasureRef.current?.(),
    )

    ed.addCommand(monacoApi.KeyMod.CtrlCmd | monacoApi.KeyCode.KeyS, () =>
      onForceSaveRef.current?.(),
    )

    const editPartsCommandId = ed.addCommand(0, () => {
      onEditPartsClickRef.current?.()
    })

    const editMetadataCommandId = ed.addCommand(0, () => {
      onEditMetadataClickRef.current?.()
    })

    codeLensProviderRef.current?.dispose()
    codeLensProviderRef.current = registerSectionCodeLensProvider(monacoApi, {
      parts: { id: editPartsCommandId ?? '', title: 'Edit Parts' },
      metadata: { id: editMetadataCommandId ?? '', title: 'Edit Metadata' },
    })

    const notifyCursor = () => {
      const model = ed.getModel()
      if (!model) return
      const selection = ed.getSelection()
      if (!selection) return
      // A multicursor selection (e.g. one produced by clicking a part
      // label, which selects that part's notes across every measure in
      // its system) surfaces as several disjoint selections here, not
      // one — getSelections()[0] is the primary/anchor selection,
      // matching `selection` above.
      const selections = ed.getSelections() ?? [selection]
      const allEmpty = selections.every((sel) => sel.isEmpty())
      // The line span covers every selection, not just the primary one, so
      // a multi-measure range select reports (and highlights) all of its
      // measures.
      onSelectionChangeRef.current?.(
        Math.min(...selections.map((sel) => sel.startLineNumber)),
        Math.max(...selections.map((sel) => sel.endLineNumber)),
        allEmpty,
      )
      if (onSelectionOffsetChangeRef.current) {
        const text = model.getValue()
        const ranges: EditorSelection[] = selections.map((sel) => ({
          start: stringIndexToByteOffset(
            text,
            model.getOffsetAt(sel.getStartPosition()),
          ),
          end: stringIndexToByteOffset(
            text,
            model.getOffsetAt(sel.getEndPosition()),
          ),
        }))
        onSelectionOffsetChangeRef.current(ranges, allEmpty)
      }
      onCursorLineChangeRef.current?.(selection.startLineNumber)
    }
    ed.onDidChangeCursorPosition(notifyCursor)
    notifyCursor()
  }

  useEffect(() => {
    applyDiagnostics()
  }, [applyDiagnostics])

  useEffect(() => {
    applyMeasureViewZones()
  }, [applyMeasureViewZones])

  useEffect(() => {
    applyDiagnosticViewZones()
  }, [applyDiagnosticViewZones])

  return (
    <div className="editor">
      {toolbar ? <div className="editor-toolbar">{toolbar}</div> : null}
      <div className="editor-surface">
        <MonacoEditor
          height="100%"
          language={POENTARO_LANGUAGE_ID}
          theme={EDITOR_THEME}
          path={path}
          defaultValue={value}
          onChange={(next) => onChange(next ?? '')}
          beforeMount={(monacoApi) => {
            registerPoentaroLanguage(monacoApi)
            registerPoentaroRenameProvider(monacoApi)
            registerPoentaroSemanticTokensProvider(monacoApi)
            definePoentaroEditorTheme(monacoApi)
          }}
          onMount={handleMount}
          options={{
            readOnly,
            codeLens: true,
            'semanticHighlighting.enabled': true,
            minimap: { enabled: false },
            fontFamily: 'var(--mono)',
            fontSize: 14,
            lineHeight: 21,
            padding: { top: 16, bottom: 16 },
            scrollBeyondLastLine: false,
            wordWrap: 'off',
            tabSize: 2,
            renderLineHighlight: 'line',
            renderValidationDecorations: 'on',
            overviewRulerLanes: 2,
            hideCursorInOverviewRuler: true,
            overviewRulerBorder: false,
            glyphMargin: false,
            folding: false,
            lineNumbers: 'on',
            lineNumbersMinChars: 3,
            scrollbar: {
              verticalScrollbarSize: 10,
              horizontalScrollbarSize: 10,
            },
          }}
        />
      </div>
    </div>
  )
})
