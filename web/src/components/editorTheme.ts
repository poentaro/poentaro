import type { Monaco } from '@monaco-editor/react'
import type { HighlightKind } from '../poentaroWasm'

export const EDITOR_THEME = 'poentaro'
// Matches preview measure highlight (rgba(255, 200, 0, 0.25)); Monaco only accepts hex.
const MEASURE_HIGHLIGHT_COLOR = '#ffc80040'

interface TokenStyle {
  foreground: string
  fontStyle?: string
}

/** Style of each keyword kind the semantic-tokens provider reports; its keys
 * are also that provider's legend. */
export const highlightKindStyles: Record<HighlightKind, TokenStyle> = {
  'section-header': { foreground: '0000ff', fontStyle: 'bold' },
  'metadata-key': { foreground: '001080' },
  'part-kind': { foreground: '267f99' },
  'directive-key': { foreground: '0000ff' },
  'lyric-reading': { foreground: '795e26', fontStyle: 'italic' },
}

export function definePoentaroEditorTheme(monacoApi: Monaco) {
  monacoApi.editor.defineTheme(EDITOR_THEME, {
    base: 'vs',
    inherit: true,
    rules: [
      { token: 'comment', foreground: '008000', fontStyle: 'italic' },
      { token: 'string', foreground: 'a31515' },
      { token: 'tag', foreground: '267f99', fontStyle: 'bold' },
      { token: 'operator', foreground: '795e26' },
      ...Object.entries(highlightKindStyles).map(([token, style]) => ({
        token,
        ...style,
      })),
    ],
    colors: {
      'editor.lineHighlightBackground': MEASURE_HIGHLIGHT_COLOR,
      'editor.lineHighlightBorder': '#00000000',
    },
  })
}
