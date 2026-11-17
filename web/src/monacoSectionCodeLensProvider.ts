import type { Monaco } from '@monaco-editor/react'
import type { editor, IDisposable, languages } from 'monaco-editor'
import { POENTARO_LANGUAGE_ID } from './monacoPoentaroLanguage'
import { poentaroWasm, type SectionKind } from './poentaroWasm'
import { spanPositions } from './semanticTokens'
import { ensureWasmInit } from './wasmInit'

/** The CodeLens command shown above each header of a section kind; a kind
 * without one gets no lens. */
export type SectionCodeLensCommands = Partial<
  Record<SectionKind, languages.Command>
>

/** Adds a CodeLens above every section header the Rust parser recognizes
 * (`section-headers`), so a lens can never sit on a line the parser doesn't
 * treat as a header. */
export function registerSectionCodeLensProvider(
  monacoApi: Monaco,
  commands: SectionCodeLensCommands,
): IDisposable {
  return monacoApi.languages.registerCodeLensProvider(POENTARO_LANGUAGE_ID, {
    async provideCodeLenses(model: editor.ITextModel) {
      await ensureWasmInit()
      const source = model.getValue(monacoApi.editor.EndOfLinePreference.LF)
      const headers = poentaroWasm().sectionHeaders(source)
      const positions = spanPositions(
        source,
        headers.map(({ span }) => span),
      )
      const lenses = headers.flatMap(({ kind, span }) => {
        const command = commands[kind]
        const position = positions.get(span.start)
        if (!command || !position) return []
        const line = position.line + 1
        return [{ range: new monacoApi.Range(line, 1, line, 1), command }]
      })
      return { lenses, dispose: () => {} }
    },
  })
}
