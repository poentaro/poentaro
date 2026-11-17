/** The Text Styles table's column definitions, split out of
 * `MetadataStylesTable.tsx` to keep it under the max-file-lines cap. Each
 * column names its `TextStyleFields` key and builds its own
 * `TextStyleComponentValue` edit, both checked against the generated wasm
 * types. */
import type {
  FontFamilyChoice,
  TextStyleComponentValue,
  TextStyleFields,
} from '../poentaroWasm'

/** `TextStyleFields`' keys whose value is a number. */
type NumericField = {
  [Field in keyof TextStyleFields]-?: TextStyleFields[Field] extends
    | number
    | undefined
    ? Field
    : never
}[keyof TextStyleFields]

/** `TextStyleFields`' keys whose value is a `yes`/`no` flag. */
type BooleanField = {
  [Field in keyof TextStyleFields]-?: TextStyleFields[Field] extends
    | boolean
    | undefined
    ? Field
    : never
}[keyof TextStyleFields]

/** One numeric column: which field it shows, its header (also used to
 * build each input's `aria-label` as `"${rowLabel} ${subLabel}"`, so e2e
 * tests can target one component by accessible name rather than positional
 * `nth()` indexing), and the edit that sets it. */
interface NumericColumn {
  field: NumericField
  subLabel: string
  edit: (val: number | undefined) => TextStyleComponentValue
}

export const numericColumns: NumericColumn[] = [
  {
    field: 'fontSize',
    subLabel: 'Font Size',
    edit: (val) => ({ tag: 'font-size', val }),
  },
  {
    field: 'horizontalPaddingPt',
    subLabel: 'H. Padding',
    edit: (val) => ({ tag: 'horizontal-padding-pt', val }),
  },
  {
    field: 'verticalPaddingPt',
    subLabel: 'V. Padding',
    edit: (val) => ({ tag: 'vertical-padding-pt', val }),
  },
]

/** One boolean toggle column: its full label (the button's `aria-label`,
 * as `"${rowLabel} ${label}"`), the single-letter glyph shown on the
 * button (styled to preview the effect it toggles), whether it selects a
 * font face (so it's a no-op for a font role without one — see
 * `hasPreviewStyleFaces`), and the edit that sets it. */
export interface BooleanColumn {
  field: BooleanField
  label: string
  glyph: string
  glyphStyle: React.CSSProperties
  selectsFontFace: boolean
  edit: (val: boolean) => TextStyleComponentValue
}

export const booleanColumns: BooleanColumn[] = [
  {
    field: 'bold',
    label: 'Bold',
    glyph: 'B',
    glyphStyle: { fontWeight: 'bold' },
    selectsFontFace: true,
    edit: (val) => ({ tag: 'bold', val }),
  },
  {
    field: 'italic',
    label: 'Italic',
    glyph: 'I',
    glyphStyle: { fontStyle: 'italic' },
    selectsFontFace: true,
    edit: (val) => ({ tag: 'italic', val }),
  },
  {
    field: 'underline',
    label: 'Underline',
    glyph: 'U',
    glyphStyle: { textDecoration: 'underline' },
    selectsFontFace: false,
    edit: (val) => ({ tag: 'underline', val }),
  },
]

/** Display label for each `font_family` option, shown in the `<select>`
 * (see `StyleRowSpec.showFontFamily`). */
export const fontFamilyOptionLabels: Record<FontFamilyChoice, string> = {
  serif: 'Serif',
  'sans-serif': 'Sans Serif',
  monospace: 'Monospace',
}

export const fontFamilyChoices = Object.keys(
  fontFamilyOptionLabels,
) as FontFamilyChoice[]
