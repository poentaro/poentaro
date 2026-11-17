import type { Monaco } from '@monaco-editor/react'
import { anyxLightThemeRules } from '../anyx-monaco'
import { SCORE_TOKEN } from '../monacoPoentaroLanguage'
import type { HighlightKind } from '../poentaroWasm'

export const EDITOR_THEME = 'poentaro'
// Matches preview measure highlight (rgba(255, 200, 0, 0.25)); Monaco only accepts hex.
const MEASURE_HIGHLIGHT_COLOR = '#ffc80040'

interface TokenStyle {
  foreground: string
  fontStyle?: string
}

/** Style of each score-string keyword kind the semantic-tokens provider
 * reports; its keys are also that provider's legend. */
export const highlightKindStyles: Record<HighlightKind, TokenStyle> = {
  'directive-key': { foreground: '0000ff' },
  'lyric-reading': { foreground: '795e26', fontStyle: 'italic' },
}

export function definePoentaroEditorTheme(monacoApi: Monaco) {
  monacoApi.editor.defineTheme(EDITOR_THEME, {
    base: 'vs',
    inherit: true,
    rules: [
      ...anyxLightThemeRules,
      { token: SCORE_TOKEN.comment, foreground: '008000', fontStyle: 'italic' },
      { token: SCORE_TOKEN.string, foreground: 'a31515' },
      { token: SCORE_TOKEN.tag, foreground: '267f99', fontStyle: 'bold' },
      { token: SCORE_TOKEN.operator, foreground: '795e26' },
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
