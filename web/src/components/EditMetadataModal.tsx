import * as Dialog from '@radix-ui/react-dialog'
import { useState } from 'react'
import type {
  FontSizeDefault,
  MetadataEdit,
  MetadataFields,
  TextStyleComponentValue,
  TextStyleKind,
} from '../poentaroWasm'
import { metadataFieldHelp } from '../utils/metadataFieldHelp'
import { FieldHelpModal } from './FieldHelpModal'
import { MetadataFieldsTableBody } from './MetadataFieldsTableBody'
import type { StyleRowSpec } from './MetadataStylesTable'
import { MetadataStylesTable } from './MetadataStylesTable'

export interface EditMetadataModalProps {
  open: boolean
  onOpenChange: (open: boolean) => void
  /** `null` until the wasm-side parse is available (see
   * `useMetadataFields`); the modal shows no fields until then. */
  metadata: MetadataFields | null
  /** The preset the form edits, or `null` for the song's `metadata`. */
  presetName: string | null
  onFieldChange: (edit: MetadataEdit) => void
  /** Element to confine the modal to (e.g. the editor pane), so it doesn't
   * cover the preview pane. Falls back to viewport-centered when null. */
  container?: HTMLElement | null
}

const styleRowLabels: Record<TextStyleKind, string> = {
  title: 'Title Style',
  subtitle: 'Subtitle Style',
  author: 'Author Style',
  sequence: 'Sequence Style',
  'part-legend': 'Part Legend Style',
  'measure-number': 'Measure Number Style',
  'section-label': 'Section Label Style',
  'part-label': 'Part Label Style',
  'page-number': 'Page Number Style',
  'running-title': 'Running Title Style',
  lyrics: 'Lyrics Style',
  notes: 'Notes Style',
  chords: 'Chords Style',
  'note-dash': 'Note Dash Style',
}

/** A style row's "**Font Size** defaults to …" help line, spelled out from
 * the Rust-side rule its default was resolved by. */
function fontSizeDefaultHelp(rule: FontSizeDefault): string {
  switch (rule.tag) {
    case 'fixed':
      return `**Font Size** defaults to ${rule.val}.`
    case 'row-height-percent':
      return `**Font Size** defaults to ${rule.val}% of **Row Height**.`
    case 'same-as':
      return `**Font Size** defaults to **${styleRowLabels[rule.val]}**'s Font Size.`
  }
}

const thStyle: React.CSSProperties = {
  padding: '6px 10px',
  textAlign: 'left',
  fontWeight: 600,
  fontSize: '12px',
  color: '#444',
  borderBottom: '2px solid #ddd',
  background: '#f5f5f5',
}

export function EditMetadataModal({
  open,
  onOpenChange,
  metadata,
  presetName,
  onFieldChange,
  container,
}: EditMetadataModalProps) {
  const editingPreset = presetName != null
  const [helpContent, setHelpContent] = useState<{
    label: string
    help: string
  } | null>(null)
  const showHelp = (label: string, help: string) =>
    setHelpContent({ label, help })

  const styleRows: StyleRowSpec[] = (metadata?.styles ?? []).map(
    ({ kind, fields, defaults, fontSizeDefault }) => ({
      kind,
      label: styleRowLabels[kind],
      help: `${metadataFieldHelp[kind]}\n\n${fontSizeDefaultHelp(fontSizeDefault)}`,
      value: fields,
      placeholder: defaults,
      editingPreset,
      onReset:
        editingPreset && Object.values(fields).some((field) => field != null)
          ? () => onFieldChange({ tag: 'reset-style', val: kind })
          : undefined,
    }),
  )

  const setStyle = (kind: TextStyleKind) => (value: TextStyleComponentValue) =>
    onFieldChange({ tag: 'style', val: { kind, value } })

  return (
    <Dialog.Root open={open} onOpenChange={onOpenChange} modal={false}>
      <Dialog.Portal container={container ?? undefined}>
        <Dialog.Overlay
          style={{
            position: container ? 'absolute' : 'fixed',
            inset: 0,
            background: 'rgba(0,0,0,0.35)',
            zIndex: 1000,
          }}
        />
        <Dialog.Content
          data-testid="edit-metadata-modal"
          style={{
            position: container ? 'absolute' : 'fixed',
            top: container ? 0 : '50%',
            left: container ? 0 : '50%',
            transform: container ? undefined : 'translate(-50%, -50%)',
            background: '#fff',
            border: container ? 'none' : '1px solid #ddd',
            borderRadius: container ? 0 : '6px',
            boxShadow: '0 8px 32px rgba(0,0,0,0.16)',
            zIndex: 1001,
            minWidth: container ? undefined : '420px',
            width: container ? '100%' : undefined,
            height: container ? '100%' : undefined,
            maxWidth: container ? undefined : '90vw',
            maxHeight: container ? undefined : '80vh',
            display: 'flex',
            flexDirection: 'column',
            fontFamily: 'var(--mono, monospace)',
          }}
        >
          <div
            style={{
              display: 'flex',
              alignItems: 'center',
              justifyContent: 'space-between',
              padding: '12px 16px',
              borderBottom: '1px solid #eee',
            }}
          >
            <Dialog.Title
              style={{ margin: 0, fontSize: '14px', fontWeight: 600 }}
            >
              {editingPreset
                ? `Edit Metadata (${presetName})`
                : 'Edit Metadata'}
            </Dialog.Title>
            <Dialog.Close
              style={{
                background: 'none',
                border: 'none',
                cursor: 'pointer',
                fontSize: '16px',
                color: '#666',
                lineHeight: 1,
                padding: '2px 4px',
              }}
            >
              ×
            </Dialog.Close>
          </div>
          {metadata && (
            <div style={{ overflowY: 'auto', flex: 1 }}>
              <div style={{ overflowX: 'auto' }}>
                <MetadataStylesTable
                  rows={styleRows}
                  showHelp={showHelp}
                  onChange={setStyle}
                />
              </div>
              <table
                style={{
                  width: '100%',
                  borderCollapse: 'collapse',
                  tableLayout: 'fixed',
                }}
              >
                <colgroup>
                  <col style={{ width: '40%' }} />
                  <col style={{ width: '60%' }} />
                </colgroup>
                <thead>
                  <tr>
                    <th style={thStyle}>Field</th>
                    <th style={thStyle}>Value</th>
                  </tr>
                </thead>
                <MetadataFieldsTableBody
                  metadata={metadata}
                  showHelp={showHelp}
                  onFieldChange={onFieldChange}
                  editingPreset={editingPreset}
                />
              </table>
            </div>
          )}
          <FieldHelpModal
            content={helpContent}
            onOpenChange={(open) => !open && setHelpContent(null)}
          />
        </Dialog.Content>
      </Dialog.Portal>
    </Dialog.Root>
  )
}
