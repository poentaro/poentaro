import type {
  SvgKind,
  SvgVariant,
  Tag,
  TransparentRectRole,
} from './poentaroWasm'

/**
 * Every `data-variant` attribute value written onto rendered preview SVG
 * elements: a text element's `SvgVariant`, a transparent rect's
 * `TransparentRectRole`, or the `playback-cursor-rect` element kind — each
 * written verbatim as the jco-generated string, so there is no TS-side
 * mapping to keep in step with the WIT enums.
 */
export type DataVariant =
  | SvgVariant
  | TransparentRectRole
  | Extract<SvgKind['tag'], 'playback-cursor-rect'>

/** Selects the elements whose `data-variant` is `variant`. */
export function variantSelector(variant: DataVariant): string {
  return `[data-variant="${variant}"]`
}

/** Selects the `<rect>` elements whose `data-variant` is `variant`. */
export function rectVariantSelector(variant: DataVariant): string {
  return `rect${variantSelector(variant)}`
}

/**
 * Every `data-tag` attribute value written onto a rendered preview SVG `<g>`
 * group, keyed by `Tag['tag']`
 * (`crates/poentaro-wasm/wit/world.wit`). `satisfies Record<Tag['tag'], string>`
 * means renaming, adding, or removing a `Tag` variant fails the TS build
 * right here until this map (and everything derived from it below) is
 * updated — closing the exact `data-tag`/`Tag` gap the "Cross-boundary
 * invariants" section of `AGENTS.md` names.
 *
 * This file is the single source of truth for the tag ↔ attribute-name ↔
 * field mapping in both directions: `groupAttributesForTag` (write) and
 * `tagFromElement` (read) below, plus the typed selector builders
 * (`tagSelector`, `tagFieldAttribute`, `ancestorTagXpath`) that replace every
 * other file's hand-formatted `[data-tag="..."]` selector string, including
 * the e2e step files'.
 */
export const DATA_TAG = {
  measure: 'measure',
  'bar-number': 'bar-number',
  'section-label': 'section-label',
  note: 'note',
  'part-label': 'part-label',
  lyric: 'lyric',
  'lyric-label': 'lyric-label',
  'bar-line': 'bar-line',
  'guitar-diagram': 'guitar-diagram',
} as const satisfies Record<Tag['tag'], string>

/**
 * The two boolean `data-*-range-active` flags imperatively toggled on a
 * part-label/lyric-label click-target rect by `previewLabelRangeHighlights.ts`
 * (`setAttribute`/`removeAttribute`, no value — a presence-only flag) and
 * read back by `injectPreviewInteractionStyles.ts`.
 */
export const DATA_RANGE_ACTIVE_FLAG = {
  partLabel: 'data-part-label-range-active',
  lyricLabel: 'data-lyric-label-range-active',
} as const

/** The `val` payload of the `Tag` variant `K`. */
type TagValue<K extends Tag['tag']> = Extract<Tag, { tag: K }>['val']

/**
 * The `data-*` attribute each `Tag` variant's `val` field is written to on
 * its rendered `<g>` group. `satisfies` requires exactly the generated
 * fields of every variant, so adding, renaming or removing a WIT `Tag`
 * field fails the build here. The writer (`groupAttributesForTag`), the
 * reader (`tagFromElement`) and the selector builder (`tagSelector`) all
 * derive their attribute names from this one table.
 */
const TAG_FIELD_ATTRIBUTES = {
  measure: { index: 'data-measure-index', end: 'data-measure-index-end' },
  'bar-number': {
    index: 'data-measure-index',
    end: 'data-measure-index-end',
  },
  'section-label': { label: 'data-section-label' },
  note: { sourcePartIndex: 'data-part-index', noteId: 'data-note-id' },
  'part-label': {
    sourcePartIndex: 'data-part-index',
    measureIndexStart: 'data-measure-index-start',
    measureIndexEnd: 'data-measure-index-end',
  },
  lyric: {
    sourcePartIndex: 'data-part-index',
    noteId: 'data-note-id',
  },
  'lyric-label': {
    sourcePartIndex: 'data-part-index',
    measureIndexStart: 'data-measure-index-start',
    measureIndexEnd: 'data-measure-index-end',
  },
  'bar-line': {
    measureIndexNext: 'data-measure-index-next',
    measureIndexPrev: 'data-measure-index-prev',
  },
  'guitar-diagram': {
    frets: 'data-guitar-frets',
    baseFret: 'data-guitar-base-fret',
  },
} as const satisfies {
  [K in Tag['tag']]: Record<keyof TagValue<K>, `data-${string}`>
}

/** Whether a rendered group of each `Tag` variant gets a pointer cursor. */
const TAG_HAS_POINTER_CURSOR = {
  measure: true,
  'bar-number': true,
  'section-label': true,
  note: false,
  'part-label': true,
  lyric: false,
  'lyric-label': true,
  'bar-line': true,
  'guitar-diagram': false,
} as const satisfies Record<Tag['tag'], boolean>

/** The `data-*` attribute name `field` of `Tag` variant `tagType` is
 * written to, e.g. `tagFieldAttribute('note', 'noteId')` is
 * `'data-note-id'`. */
export function tagFieldAttribute<K extends Tag['tag']>(
  tagType: K,
  field: keyof TagValue<K>,
): string {
  const fieldAttributes: Record<string, string> = TAG_FIELD_ATTRIBUTES[tagType]
  const attribute = fieldAttributes[field as string]
  if (attribute === undefined) {
    throw new Error(`No data attribute for ${tagType}.${String(field)}`)
  }
  return attribute
}

/** The `data-*` attributes a rendered `<g>` group carries for its `Tag`,
 * keyed by real attribute name so `PreviewSvgRenderer.tsx` can spread them
 * straight onto the element — the writer half of this file's tag ↔
 * attribute mapping. */
export function groupAttributesForTag(
  tag: Tag | undefined,
): Record<string, string | number> {
  if (!tag) return {}
  return Object.fromEntries([
    ['data-tag', DATA_TAG[tag.tag]],
    ...Object.entries(tag.val)
      .filter(([, value]) => value !== undefined)
      .map(([field, value]) => [
        tagFieldAttribute(tag.tag, field as keyof TagValue<typeof tag.tag>),
        value,
      ]),
  ])
}

/** Whether a rendered group for `tag` should show a pointer cursor. */
export function tagHasPointerCursor(tag: Tag | undefined): boolean {
  return tag ? TAG_HAS_POINTER_CURSOR[tag.tag] : false
}

/** Selects the rendered groups of `Tag` variant `tagType` whose `data-*`
 * attributes match every given field, e.g.
 * `tagSelector('note', { sourcePartIndex: 0, noteId: 3 })` is
 * `[data-tag="note"][data-part-index="0"][data-note-id="3"]`. */
export function tagSelector<K extends Tag['tag']>(
  tagType: K,
  fields: Partial<TagValue<K>> = {},
): string {
  const fieldSelectors = Object.entries(fields)
    .filter(([, value]) => value !== undefined)
    .map(
      ([field, value]) =>
        `[${tagFieldAttribute(tagType, field as keyof TagValue<K>)}="${value}"]`,
    )
  return `[data-tag="${DATA_TAG[tagType]}"]${fieldSelectors.join('')}`
}

/** Reads `field` of `Tag` variant `tagType` off a rendered group as an
 * integer, or `undefined` if it's missing or not a valid integer. */
function readIntField<K extends Tag['tag']>(
  el: Element,
  tagType: K,
  field: keyof TagValue<K>,
): number | undefined {
  const value = el.getAttribute(tagFieldAttribute(tagType, field))
  if (value === null) return undefined
  const parsed = Number.parseInt(value, 10)
  return Number.isNaN(parsed) ? undefined : parsed
}

/** Reverse lookup of `DATA_TAG`, so `tagFromElement`'s switch below is keyed
 * off a real `Tag['tag']` literal union and its `default` arm's `never` check
 * catches an unhandled variant at compile time. */
function tagTypeFromDataTagValue(value: string | null): Tag['tag'] | undefined {
  return (Object.keys(DATA_TAG) as Tag['tag'][]).find(
    (tagType) => DATA_TAG[tagType] === value,
  )
}

/**
 * The `Tag` a rendered `<g>` group's own `data-*` attributes encode, or
 * `undefined` if `el` isn't such a group (no recognized `data-tag`) or is
 * missing a field that `type` requires — the reader half of this file's tag
 * ↔ attribute mapping, mirroring `groupAttributesForTag` field-for-field. Replaces
 * every hand-rolled `dataset.partIndex`/`Number(...)` parsing that used to be
 * duplicated per consumer (`clickableElementIdFromElement`,
 * `usePlaybackCursor.ts`, `previewRangeHighlights.ts`,
 * `previewLabelRangeHighlights.ts`).
 */
export function tagFromElement(el: Element): Tag | undefined {
  const tagType = tagTypeFromDataTagValue(el.getAttribute('data-tag'))
  if (tagType === undefined) return undefined
  switch (tagType) {
    case 'measure':
    case 'bar-number': {
      const index = readIntField(el, tagType, 'index')
      const end = readIntField(el, tagType, 'end')
      if (index === undefined || end === undefined) return undefined
      return { tag: tagType, val: { index, end } }
    }
    case 'section-label': {
      const label = el.getAttribute(tagFieldAttribute(tagType, 'label'))
      if (label === null) return undefined
      return { tag: 'section-label', val: { label } }
    }
    case 'note': {
      const sourcePartIndex = readIntField(el, tagType, 'sourcePartIndex')
      const noteId = readIntField(el, tagType, 'noteId')
      if (sourcePartIndex === undefined || noteId === undefined)
        return undefined
      return { tag: 'note', val: { sourcePartIndex, noteId } }
    }
    case 'part-label': {
      const sourcePartIndex = readIntField(el, tagType, 'sourcePartIndex')
      const measureIndexStart = readIntField(el, tagType, 'measureIndexStart')
      const measureIndexEnd = readIntField(el, tagType, 'measureIndexEnd')
      if (
        sourcePartIndex === undefined ||
        measureIndexStart === undefined ||
        measureIndexEnd === undefined
      )
        return undefined
      return {
        tag: 'part-label',
        val: { sourcePartIndex, measureIndexStart, measureIndexEnd },
      }
    }
    case 'lyric': {
      const sourcePartIndex = readIntField(el, tagType, 'sourcePartIndex')
      const noteId = readIntField(el, tagType, 'noteId')
      if (sourcePartIndex === undefined || noteId === undefined)
        return undefined
      return { tag: 'lyric', val: { sourcePartIndex, noteId } }
    }
    case 'lyric-label': {
      const sourcePartIndex = readIntField(el, tagType, 'sourcePartIndex')
      const measureIndexStart = readIntField(el, tagType, 'measureIndexStart')
      const measureIndexEnd = readIntField(el, tagType, 'measureIndexEnd')
      if (
        sourcePartIndex === undefined ||
        measureIndexStart === undefined ||
        measureIndexEnd === undefined
      )
        return undefined
      return {
        tag: 'lyric-label',
        val: { sourcePartIndex, measureIndexStart, measureIndexEnd },
      }
    }
    case 'bar-line': {
      return {
        tag: 'bar-line',
        val: {
          measureIndexNext: readIntField(el, tagType, 'measureIndexNext'),
          measureIndexPrev: readIntField(el, tagType, 'measureIndexPrev'),
        },
      }
    }
    case 'guitar-diagram': {
      const frets = el.getAttribute(tagFieldAttribute(tagType, 'frets'))
      const baseFret = readIntField(el, tagType, 'baseFret')
      if (frets === null || baseFret === undefined) return undefined
      return { tag: 'guitar-diagram', val: { frets, baseFret } }
    }
    default: {
      const exhaustiveCheck: never = tagType
      throw new Error(
        `Unhandled data-tag value: ${JSON.stringify(exhaustiveCheck)}`,
      )
    }
  }
}

/** A Playwright `xpath=` selector for the nearest ancestor group of `Tag`
 * variant `tagType`. */
export function ancestorTagXpath(tagType: Tag['tag']): string {
  return `xpath=ancestor::*[@data-tag="${DATA_TAG[tagType]}"][1]`
}
