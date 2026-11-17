import { hasPreviewStyleFaces } from '../fontRoles'
import type {
  TextStyleComponentValue,
  TextStyleDefaults,
  TextStyleFields,
  TextStyleKind,
} from '../poentaroWasm'
import { FieldLabel } from './FieldHelpModal'
import {
  inheritedStyle,
  NumberStepper,
  ResetOverrideButton,
} from './MetadataFieldRows'
import { optionalNumber } from './MetadataFieldsTableBody'
import {
  type BooleanColumn,
  booleanColumns,
  fontFamilyChoices,
  fontFamilyOptionLabels,
  numericColumns,
} from './metadataStyleColumns'

export interface StyleRowSpec {
  kind: TextStyleKind
  label: string
  help: string
  value: TextStyleFields
  /** What each component resolves to when unset. */
  placeholder: TextStyleDefaults
  /** Editing a preset: unset components show the song's `metadata`
   * values, muted. */
  editingPreset: boolean
  /** Set while the preset overrides any component: clears them all. */
  onReset?: () => void
  /** No row currently sets this `false` — every kind, including
   * `notes`/`chords`/`note_dash` (whose glyph widths are re-measured
   * against whichever font `font_family` resolves to, see `syntax.md`),
   * accepts a `font_family` override. Kept as an escape hatch in case a
   * future kind needs to opt out again. */
  showFontFamily?: boolean
}

const thStyle: React.CSSProperties = {
  padding: '6px 4px',
  textAlign: 'center',
  fontWeight: 600,
  fontSize: '11px',
  color: '#444',
  borderBottom: '2px solid #ddd',
  background: '#f5f5f5',
  whiteSpace: 'nowrap',
}

const thLabelStyle: React.CSSProperties = {
  ...thStyle,
  textAlign: 'left',
  padding: '6px 10px',
}

const tdStyle: React.CSSProperties = {
  padding: '4px',
  borderBottom: '1px solid #eee',
  verticalAlign: 'middle',
}

const tdLabelStyle: React.CSSProperties = {
  ...tdStyle,
  padding: '6px 10px',
  fontSize: '13px',
  whiteSpace: 'nowrap',
}

const styleToggleButtonStyle: React.CSSProperties = {
  width: '22px',
  height: '22px',
  lineHeight: 1,
  fontSize: '13px',
  padding: 0,
  border: '1px solid #cbd5e0',
  borderRadius: '3px',
  background: '#fff',
  color: '#444',
  cursor: 'pointer',
}

const styleToggleButtonPressedStyle: React.CSSProperties = {
  ...styleToggleButtonStyle,
  background: '#e2e8f0',
  borderColor: '#94a3b8',
  color: '#1a202c',
}

const styleToggleButtonDisabledStyle: React.CSSProperties = {
  ...styleToggleButtonStyle,
  background: '#f5f5f5',
  borderColor: '#e2e8f0',
  color: '#bbb',
  cursor: 'not-allowed',
}

const fontFamilySelectStyle: React.CSSProperties = {
  width: '100%',
  height: '22px',
  fontSize: '11px',
  border: '1px solid #cbd5e0',
  borderRadius: '3px',
  background: '#fff',
  color: '#444',
}

/** One `B`/`I`/`U` toggle button for a single boolean style component:
 * `checked` reflects the effective value (the field's own override, falling
 * back to its default when unset — same fallback `NumberStepper` shows via
 * its greyed-out `placeholder`), and clicking always writes the opposite as
 * an explicit `yes`/`no` (mirrors `CheckboxFieldRow`, which never exposes an
 * "unset" state in its UI either). */
function StyleToggleButton({
  column,
  checked,
  disabledReason,
  ariaLabel,
  onClick,
}: {
  column: BooleanColumn
  checked: boolean
  /** When set, the toggle renders disabled and this becomes its `title`
   * (e.g. explaining that the row's current font has no real bold/italic
   * face — see `hasPreviewStyleFaces`). */
  disabledReason?: string
  ariaLabel: string
  onClick: () => void
}) {
  const disabled = disabledReason != null
  return (
    <button
      type="button"
      aria-label={ariaLabel}
      aria-pressed={checked}
      disabled={disabled}
      title={disabledReason}
      style={
        disabled
          ? styleToggleButtonDisabledStyle
          : checked
            ? styleToggleButtonPressedStyle
            : styleToggleButtonStyle
      }
      onClick={onClick}
    >
      <span style={column.glyphStyle}>{column.glyph}</span>
    </button>
  )
}

function StyleTableRow({
  row,
  showHelp,
  onChange,
}: {
  row: StyleRowSpec
  showHelp: (label: string, help: string) => void
  onChange: (value: TextStyleComponentValue) => void
}) {
  const {
    label,
    help,
    value,
    placeholder,
    editingPreset,
    onReset,
    showFontFamily = true,
  } = row
  const mutedIfInherited = (component: unknown) =>
    editingPreset && component == null ? inheritedStyle : undefined
  // A row that opts out of the font-family selector (see
  // `StyleRowSpec.showFontFamily`'s doc comment) has no way to change away
  // from its fixed default face, so there's no capability restriction to
  // look up for it either.
  const effectiveFontFamily = showFontFamily
    ? (value.fontFamily ?? placeholder.fontFamily)
    : null
  return (
    <tr>
      <td style={tdLabelStyle}>
        <span style={{ display: 'inline-flex', alignItems: 'center' }}>
          <FieldLabel label={label} help={help} onShowHelp={showHelp} />
          <ResetOverrideButton label={label} onReset={onReset} />
        </span>
      </td>
      {numericColumns.map((column) => (
        <td key={column.field} style={tdStyle}>
          <NumberStepper
            value={value[column.field] ?? ''}
            defaultValue={placeholder[column.field]}
            min={0}
            aria-label={`${label} ${column.subLabel}`}
            placeholder={String(placeholder[column.field])}
            onChange={(text) => onChange(column.edit(optionalNumber(text)))}
          />
        </td>
      ))}
      {booleanColumns.map((column) => {
        const checked = value[column.field] ?? placeholder[column.field]
        const unsupported =
          effectiveFontFamily != null &&
          column.selectsFontFace &&
          !hasPreviewStyleFaces(effectiveFontFamily)
        const disabledReason =
          unsupported && effectiveFontFamily != null
            ? `${fontFamilyOptionLabels[effectiveFontFamily]} has no ${column.field} face, so this has no visible effect`
            : undefined
        return (
          <td
            key={column.field}
            style={{
              ...tdStyle,
              textAlign: 'center',
              ...mutedIfInherited(value[column.field]),
            }}
          >
            <StyleToggleButton
              column={column}
              checked={checked}
              disabledReason={disabledReason}
              ariaLabel={`${label} ${column.label}`}
              onClick={() => onChange(column.edit(!checked))}
            />
          </td>
        )
      })}
      <td style={tdStyle}>
        {showFontFamily && (
          <select
            aria-label={`${label} Font Family`}
            style={{
              ...fontFamilySelectStyle,
              ...mutedIfInherited(value.fontFamily),
            }}
            value={value.fontFamily ?? placeholder.fontFamily}
            onChange={(e) =>
              onChange({
                tag: 'font-family',
                val: fontFamilyChoices.find(
                  (choice) => choice === e.target.value,
                ),
              })
            }
          >
            {fontFamilyChoices.map((option) => (
              <option key={option} value={option}>
                {fontFamilyOptionLabels[option]}
              </option>
            ))}
          </select>
        )}
      </td>
    </tr>
  )
}

/** The "Text Styles" half of `EditMetadataModal`'s field table: one row per
 * `<kind> = { font_size: N, horizontal_padding_pt: N, vertical_padding_pt:
 * N, bold: yes/no, italic: yes/no, underline: yes/no, font_family: ... }`
 * text-style kind (see the generated `TextStyleFields`), with the component sub-labels
 * (Font Size/H. Padding/V. Padding/B/I/U/Font) hoisted into a single header
 * row instead of repeating per kind. Rendered as its own `<table>`,
 * separate from the plain label/value table for non-style fields (see
 * `MetadataFieldsTableBody`). */
export function MetadataStylesTable({
  rows,
  showHelp,
  onChange,
}: {
  rows: StyleRowSpec[]
  showHelp: (label: string, help: string) => void
  onChange: (kind: TextStyleKind) => (value: TextStyleComponentValue) => void
}) {
  return (
    <table
      style={{
        width: '100%',
        borderCollapse: 'collapse',
      }}
    >
      <colgroup>
        <col style={{ width: '22%' }} />
        <col style={{ width: '13%' }} />
        <col style={{ width: '13%' }} />
        <col style={{ width: '13%' }} />
        <col style={{ width: '8%' }} />
        <col style={{ width: '8%' }} />
        <col style={{ width: '8%' }} />
        <col style={{ width: '15%' }} />
      </colgroup>
      <thead>
        <tr>
          <th style={thLabelStyle}>Text Style</th>
          <th style={thStyle}>Font Size</th>
          <th style={thStyle}>H. Padding</th>
          <th style={thStyle}>V. Padding</th>
          <th style={thStyle}>B</th>
          <th style={thStyle}>I</th>
          <th style={thStyle}>U</th>
          <th style={thStyle}>Font</th>
        </tr>
      </thead>
      <tbody>
        {rows.map((row) => (
          <StyleTableRow
            key={row.kind}
            row={row}
            showHelp={showHelp}
            onChange={onChange(row.kind)}
          />
        ))}
      </tbody>
    </table>
  )
}
