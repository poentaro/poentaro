/// <reference types="node" />
import { readdirSync, readFileSync } from 'node:fs'
import { join } from 'node:path'
import { fileURLToPath } from 'node:url'
import { describe, expect, it } from 'vitest'
import {
  groupAttributesForTag,
  tagFromElement,
  tagSelector,
} from './dataAttributes'
import type { Tag } from './poentaroWasm'

const srcDir = fileURLToPath(new URL('.', import.meta.url))

function cssFilesUnder(dir: string): string[] {
  return readdirSync(dir, { recursive: true, encoding: 'utf-8' })
    .filter((path) => path.endsWith('.css'))
    .map((path) => join(dir, path))
}

// `data-tag`/`data-variant` values are jco-generated WIT strings. Plain CSS
// can't reference those types, so selectors on them belong in
// `injectPreviewInteractionStyles.ts`, built from the typed helpers in
// `dataAttributes.ts`.
describe('plain CSS files', () => {
  it.each(
    cssFilesUnder(srcDir),
  )('%s has no data-tag/data-variant selector', (cssPath) => {
    expect(readFileSync(cssPath, 'utf-8')).not.toMatch(/data-(tag|variant)\b/)
  })
})

const sampleTags: Tag[] = [
  { tag: 'measure', val: { index: 1, end: 2 } },
  { tag: 'bar-number', val: { index: 3, end: 3 } },
  { tag: 'section-label', val: { label: 'Chorus' } },
  { tag: 'note', val: { sourcePartIndex: 1, noteId: 4 } },
  {
    tag: 'part-label',
    val: { sourcePartIndex: 0, measureIndexStart: 2, measureIndexEnd: 5 },
  },
  { tag: 'lyric', val: { sourcePartIndex: 1, noteId: 7 } },
  {
    tag: 'lyric-label',
    val: {
      sourcePartIndex: 1,
      measureIndexStart: 0,
      measureIndexEnd: 3,
    },
  },
  { tag: 'bar-line', val: { measureIndexNext: 4, measureIndexPrev: 3 } },
]

/** Just enough of `Element` for `tagFromElement`: attributes as written by
 * `groupAttributesForTag`. */
function fakeElement(attributes: Record<string, string | number>): Element {
  return {
    getAttribute: (name: string) =>
      name in attributes ? String(attributes[name]) : null,
  } as Element
}

describe('Tag data attributes', () => {
  it.each(
    sampleTags,
  )('round-trips $tag through the written attributes', (tag) => {
    expect(tagFromElement(fakeElement(groupAttributesForTag(tag)))).toEqual(tag)
  })

  it.each(
    sampleTags,
  )('tagSelector($tag) matches the written attributes', (tag) => {
    const expected = Object.entries(groupAttributesForTag(tag))
      .map(([name, value]) => `[${name}="${value}"]`)
      .join('')
    expect(tagSelector(tag.tag, tag.val)).toBe(expected)
  })
})
