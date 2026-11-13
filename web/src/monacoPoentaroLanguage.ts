import type { Monaco } from '@monaco-editor/react'
import type * as monacoEditor from 'monaco-editor'
import { anyxLanguageConfiguration, anyxMonarchLanguage } from './anyx-monaco'

export const POENTARO_LANGUAGE_ID = 'poentaro'

/** Tokens of the score string's own coloring. anyx sees the score as one
 * raw string; these more specific tokens win over its string color. */
export const SCORE_TOKEN = {
  tag: 'tag.score',
  operator: 'operator.score',
  string: 'string.score',
  comment: 'comment.score',
} as const

/** Inside a `"""` raw string: the score's punctuation-level structure. Its
 * keywords come from the Rust lexers via
 * `registerPoentaroSemanticTokensProvider`. `$S2` is the run of quotes that
 * opened the raw string. */
const scoreState: monacoEditor.languages.IMonarchLanguageRule[] = [
  [
    /^[ \t]*("{3,})/,
    {
      cases: {
        '$1==$S2': { token: 'string.quote', next: '@pop' },
        '@default': SCORE_TOKEN.string,
      },
    },
  ],
  [/"([^"\\]|\\.)*"/, SCORE_TOKEN.string],
  [/\/\/.*$/, SCORE_TOKEN.comment],
  [/\[[^\]\n]*\]/, SCORE_TOKEN.tag],
  [/[#_=.\-~',]/, SCORE_TOKEN.operator],
  [/[()]/, '@brackets'],
  // Notes, words and lyrics keep the default text color.
  [/[^"[/#_=.\-~',()]+/, ''],
  [/./, ''],
]

/** anyx's tokenizer, except that a raw string holds the score. The raw
 * string rule goes first, so it opens the score state rather than anyx's
 * plain `raw` state. */
const poentaroMonarchLanguage: monacoEditor.languages.IMonarchLanguage = {
  ...anyxMonarchLanguage,
  tokenizer: {
    ...anyxMonarchLanguage.tokenizer,
    root: [
      [/("{3,})(?=[ \t]*$)/, { token: 'string.quote', next: '@score.$1' }],
      ...(anyxMonarchLanguage.tokenizer.root ?? []),
    ],
    score: scoreState,
  },
}

let registered = false

/** `.poentaro` documents are anyx, colored by anyx's tokenizer
 * (`anyx-monaco.ts`); the score string keeps its own coloring. */
export function registerPoentaroLanguage(monacoApi: Monaco) {
  if (registered) return
  registered = true

  monacoApi.languages.register({ id: POENTARO_LANGUAGE_ID })
  monacoApi.languages.setLanguageConfiguration(
    POENTARO_LANGUAGE_ID,
    anyxLanguageConfiguration,
  )
  monacoApi.languages.setMonarchTokensProvider(
    POENTARO_LANGUAGE_ID,
    poentaroMonarchLanguage,
  )
}
