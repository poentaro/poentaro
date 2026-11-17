import type { Monaco } from '@monaco-editor/react'
import type * as monacoEditor from 'monaco-editor'
import { highlightKindStyles } from './components/editorTheme'
import { POENTARO_LANGUAGE_ID } from './monacoPoentaroLanguage'
import { type HighlightKind, poentaroWasm } from './poentaroWasm'
import { encodeSemanticTokens } from './semanticTokens'
import { ensureWasmInit } from './wasmInit'

let registered = false

/** Highlights keywords from the Rust lexers' `highlight-tokens`, so the
 * editor never re-spells the `.poentaro` vocabulary. */
export function registerPoentaroSemanticTokensProvider(monacoApi: Monaco) {
  if (registered) return
  registered = true

  const tokenTypes = Object.keys(highlightKindStyles) as HighlightKind[]
  monacoApi.languages.registerDocumentSemanticTokensProvider(
    POENTARO_LANGUAGE_ID,
    {
      getLegend: () => ({ tokenTypes, tokenModifiers: [] }),
      async provideDocumentSemanticTokens(
        model: monacoEditor.editor.ITextModel,
      ) {
        await ensureWasmInit()
        const source = model.getValue(monacoApi.editor.EndOfLinePreference.LF)
        const tokens = poentaroWasm().highlightTokens(source)
        return { data: encodeSemanticTokens(source, tokens, tokenTypes) }
      },
      releaseDocumentSemanticTokens() {},
    },
  )
}
