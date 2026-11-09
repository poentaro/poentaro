import type {
  MetadataEdit,
  MetadataFields,
  MetadataFlagKey,
  MetadataNumberKey,
  MetadataTextKey,
} from '../poentaroWasm'
import { metadataFieldHelp } from '../utils/metadataFieldHelp'
import {
  CheckboxFieldRow,
  NumberFieldRow,
  TextFieldRow,
} from './MetadataFieldRows'

export interface MetadataFieldsTableBodyProps {
  metadata: MetadataFields
  showHelp: (label: string, help: string) => void
  onFieldChange: (edit: MetadataEdit) => void
  /** Editing a preset: unset fields show the song's `metadata` values,
   * and set ones can be reset to them. */
  editingPreset: boolean
}

const flagDefaults = {
  'merge-duplicate-measures-across-parts': 'mergeDuplicateMeasuresAcrossParts',
  'hide-resting-parts': 'hideRestingParts',
  'hide-system-dividers': 'hideSystemDividers',
} as const satisfies Record<MetadataFlagKey, string>

/** An input's text as an optional value: empty clears the field. */
export function optionalText(value: string): string | undefined {
  return value === '' ? undefined : value
}

/** A number input's text as an optional value: empty (or unparseable)
 * clears the field. */
export function optionalNumber(value: string): number | undefined {
  const parsed = Number.parseInt(value, 10)
  return Number.isNaN(parsed) ? undefined : parsed
}

/** The `<tbody>` of `EditMetadataModal`'s plain (non-text-style) field
 * table — split out to keep that file under the repo's max-file-lines cap.
 * Text-style kinds (Title Style, Subtitle Style, ...) live in the separate
 * `MetadataStylesTable`, not here. */
export function MetadataFieldsTableBody({
  metadata,
  showHelp,
  onFieldChange,
  editingPreset,
}: MetadataFieldsTableBodyProps) {
  const d = metadata.defaults
  const setText = (key: MetadataTextKey) => (value: string) =>
    onFieldChange({ tag: 'text', val: { key, value: optionalText(value) } })
  const setNumber = (key: MetadataNumberKey) => (value: string) =>
    onFieldChange({ tag: 'number', val: { key, value: optionalNumber(value) } })
  const setFlag =
    (key: MetadataFlagKey) => (e: React.ChangeEvent<HTMLInputElement>) =>
      onFieldChange({ tag: 'flag', val: { key, value: e.target.checked } })
  const resetIfSet = (value: unknown, edit: MetadataEdit) =>
    editingPreset && value != null ? () => onFieldChange(edit) : undefined
  const resetText = (key: MetadataTextKey, value: string | undefined) =>
    resetIfSet(value, { tag: 'text', val: { key, value: undefined } })
  const resetNumber = (key: MetadataNumberKey, value: number | undefined) =>
    resetIfSet(value, { tag: 'number', val: { key, value: undefined } })
  const flagRow = (key: MetadataFlagKey, value: boolean | undefined) => ({
    checked: value ?? d[flagDefaults[key]],
    inherited: editingPreset && value == null,
    onReset: resetIfSet(value, { tag: 'flag', val: { key, value: undefined } }),
    onChange: setFlag(key),
  })

  return (
    <tbody>
      <TextFieldRow
        label="Title *"
        help={metadataFieldHelp.title}
        onShowHelp={showHelp}
        value={metadata.title ?? ''}
        placeholder={d.title}
        onReset={resetText('title', metadata.title)}
        onChange={setText('title')}
      />
      <TextFieldRow
        label="Subtitle"
        help={metadataFieldHelp.subtitle}
        onShowHelp={showHelp}
        value={metadata.subtitle ?? ''}
        placeholder={d.subtitle}
        onReset={resetText('subtitle', metadata.subtitle)}
        onChange={setText('subtitle')}
      />
      <TextFieldRow
        label="Author"
        help={metadataFieldHelp.author}
        onShowHelp={showHelp}
        value={metadata.author ?? ''}
        placeholder={d.author}
        onReset={resetText('author', metadata.author)}
        onChange={setText('author')}
      />
      <NumberFieldRow
        label="Row Height"
        help={metadataFieldHelp['row-height']}
        onShowHelp={showHelp}
        value={metadata.rowHeight ?? ''}
        placeholder={String(d.rowHeight)}
        onReset={resetNumber('row-height', metadata.rowHeight)}
        onChange={setNumber('row-height')}
      />
      <NumberFieldRow
        label="Max Measures Per System"
        help={metadataFieldHelp['max-measures-per-system']}
        onShowHelp={showHelp}
        value={metadata.maxMeasuresPerSystem ?? ''}
        placeholder={String(d.maxMeasuresPerSystem)}
        onReset={resetNumber(
          'max-measures-per-system',
          metadata.maxMeasuresPerSystem,
        )}
        onChange={setNumber('max-measures-per-system')}
      />
      <NumberFieldRow
        label="Note Number Width"
        help={metadataFieldHelp['note-number-width']}
        onShowHelp={showHelp}
        value={metadata.noteNumberWidth ?? ''}
        placeholder={String(d.noteNumberWidth)}
        onReset={resetNumber('note-number-width', metadata.noteNumberWidth)}
        onChange={setNumber('note-number-width')}
      />
      <NumberFieldRow
        label="Parts List Columns"
        help={metadataFieldHelp['parts-list-columns']}
        onShowHelp={showHelp}
        value={metadata.partsListColumns ?? ''}
        placeholder={String(d.partsListColumns)}
        onReset={resetNumber('parts-list-columns', metadata.partsListColumns)}
        onChange={setNumber('parts-list-columns')}
      />
      <NumberFieldRow
        label="Part Label Width"
        help={metadataFieldHelp['part-label-width-pt']}
        onShowHelp={showHelp}
        value={metadata.partLabelWidthPt ?? ''}
        placeholder={String(d.partLabelWidthPt)}
        onReset={resetNumber('part-label-width-pt', metadata.partLabelWidthPt)}
        onChange={setNumber('part-label-width-pt')}
      />
      <NumberFieldRow
        label="Page Margin"
        help={metadataFieldHelp['page-margin-pt']}
        onShowHelp={showHelp}
        value={metadata.pageMarginPt ?? ''}
        placeholder={String(d.pageMarginPt)}
        onReset={resetNumber('page-margin-pt', metadata.pageMarginPt)}
        onChange={setNumber('page-margin-pt')}
      />
      <CheckboxFieldRow
        label="Merge Duplicate Measures Across Parts"
        help={metadataFieldHelp['merge-duplicate-measures-across-parts']}
        onShowHelp={showHelp}
        {...flagRow(
          'merge-duplicate-measures-across-parts',
          metadata.mergeDuplicateMeasuresAcrossParts,
        )}
      />
      <CheckboxFieldRow
        label="Hide Resting Parts"
        help={metadataFieldHelp['hide-resting-parts']}
        onShowHelp={showHelp}
        {...flagRow('hide-resting-parts', metadata.hideRestingParts)}
      />
      <CheckboxFieldRow
        label="Hide System Dividers"
        help={metadataFieldHelp['hide-system-dividers']}
        onShowHelp={showHelp}
        {...flagRow('hide-system-dividers', metadata.hideSystemDividers)}
      />
      <TextFieldRow
        label="Directive Row Offset (x y)"
        help={metadataFieldHelp['directive-row-offset']}
        onShowHelp={showHelp}
        value={metadata.directiveRowOffset ?? ''}
        placeholder={d.directiveRowOffset}
        onReset={resetIfSet(metadata.directiveRowOffset, {
          tag: 'directive-row-offset',
          val: undefined,
        })}
        onChange={(value) =>
          onFieldChange({
            tag: 'directive-row-offset',
            val: optionalText(value),
          })
        }
      />
    </tbody>
  )
}
