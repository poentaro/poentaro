import type { Monaco } from '@monaco-editor/react'

export const POENTARO_LANGUAGE_ID = 'poentaro'

let registered = false

/** Basic Monarch tokenizer for `.poentaro` files — see syntax.md for
 * the document and syntax.md for the score string's grammar. It only colors
 * punctuation-level structure; keywords come from the Rust lexers via
 * `registerPoentaroSemanticTokensProvider`. */
export function registerPoentaroLanguage(monacoApi: Monaco) {
  if (registered) return
  registered = true

  monacoApi.languages.register({ id: POENTARO_LANGUAGE_ID })

  monacoApi.languages.setLanguageConfiguration(POENTARO_LANGUAGE_ID, {
    comments: { lineComment: '#' },
    brackets: [
      ['(', ')'],
      ['[', ']'],
      ['{', '}'],
    ],
    autoClosingPairs: [
      { open: '(', close: ')' },
      { open: '[', close: ']' },
      { open: '{', close: '}' },
      { open: '"', close: '"' },
    ],
  })

  monacoApi.languages.setMonarchTokensProvider(POENTARO_LANGUAGE_ID, {
    defaultToken: '',
    tokenizer: {
      root: [
        // The score string, closed by the same run of quotes.
        [/("{3,})\s*$/, { token: 'delimiter', next: '@score.$1' }],
        // Quoted strings first: a `#` inside a string is not a comment.
        [/"([^"\\]|\\.)*"/, 'string'],
        [/#.*$/, 'comment'],
        [/\/\/.*$/, 'comment'],
        // [Abbrev] part reference.
        [/\[[^\]\n]*\]/, 'tag'],
        [/[()[\]{}]/, '@brackets'],
        [/[_=.\-~',]/, 'operator'],
      ],
      score: [
        [
          /^\s*("{3,})/,
          {
            cases: {
              '$1==$S2': { token: 'delimiter', next: '@pop' },
              '@default': 'string',
            },
          },
        ],
        [/"([^"\\]|\\.)*"/, 'string'],
        [/\/\/.*$/, 'comment'],
        [/\[[^\]\n]*\]/, 'tag'],
        [/#/, 'operator'],
        [/[()]/, '@brackets'],
        [/[_=.\-~',]/, 'operator'],
      ],
    },
  })
}
