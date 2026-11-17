import { DiffEditor, type DiffOnMount } from '@monaco-editor/react'
import type { DiffLayout } from '../hooks/useDiffLayout'
import {
  POENTARO_LANGUAGE_ID,
  registerPoentaroLanguage,
} from '../monacoPoentaroLanguage'
import { EDITOR_THEME } from './editorTheme'
import './FileDialogs.css'

interface PoentaroDiffEditorProps {
  original: string
  modified: string
  layout: DiffLayout
  /** Whether the modified side can be edited; the original never can. */
  editable?: boolean
  onMount?: DiffOnMount
}

/** A Monaco diff of two `.poentaro` texts, highlighted like the main editor,
 * with changed words marked inside each changed line. Fills its parent. */
export function PoentaroDiffEditor({
  original,
  modified,
  layout,
  editable = false,
  onMount,
}: PoentaroDiffEditorProps) {
  return (
    <DiffEditor
      height="100%"
      language={POENTARO_LANGUAGE_ID}
      theme={EDITOR_THEME}
      original={original}
      modified={modified}
      beforeMount={registerPoentaroLanguage}
      options={{
        readOnly: !editable,
        originalEditable: false,
        renderSideBySide: layout === 'side-by-side',
        // Otherwise Monaco forces inline below ~900px, overriding the toggle.
        useInlineViewWhenSpaceIsLimited: false,
        wordWrap: 'on',
        diffWordWrap: 'on',
        minimap: { enabled: false },
        fontFamily: 'var(--mono)',
        fontSize: 14,
        lineHeight: 24,
        renderValidationDecorations: 'on',
        scrollBeyondLastLine: false,
      }}
      onMount={onMount}
    />
  )
}

/** Switches a `PoentaroDiffEditor` between inline and side by side. */
export function DiffLayoutToggle({
  layout,
  onToggle,
  testId,
}: {
  layout: DiffLayout
  onToggle: () => void
  testId: string
}) {
  return (
    <button
      type="button"
      className="file-dialog-button"
      data-testid={testId}
      onClick={onToggle}
    >
      {layout === 'inline' ? 'Side by side' : 'Inline'}
    </button>
  )
}
