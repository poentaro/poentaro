import * as Dialog from '@radix-ui/react-dialog'
import { usePartSettingOptions } from '../hooks/usePartSettingOptions'
import type { PartDeclaration, PartInfo, PartSettings } from '../types'
import { PartRow } from './PartRow'

export interface EditPartsModalProps {
  open: boolean
  onOpenChange: (open: boolean) => void
  partDeclarations: PartDeclaration[]
  allParts: PartInfo[]
  onPartDeclarationChange: (
    abbreviation: string,
    settings: PartSettings,
  ) => void
  /** Bulk-shifts every note already written for this part by `delta`
   * octaves, rewriting `'`/`,` markers in the source (distinct from
   * `octaveOffset` above, which is MIDI-playback-only). */
  onShiftPartOctave: (abbreviation: string, delta: number) => void
  previewInstrument: (programNumber: number) => void
  previewPercussion: (key: number) => void
  stopPreviewInstrument: () => void
  previewAudioPlaying: boolean
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

export function EditPartsModal({
  open,
  onOpenChange,
  partDeclarations,
  allParts,
  onPartDeclarationChange,
  onShiftPartOctave,
  previewInstrument,
  previewPercussion,
  stopPreviewInstrument,
  previewAudioPlaying,
}: EditPartsModalProps) {
  const options = usePartSettingOptions(open)
  return (
    <Dialog.Root open={open} onOpenChange={onOpenChange}>
      <Dialog.Portal>
        <Dialog.Overlay
          style={{
            position: 'fixed',
            inset: 0,
            background: 'rgba(0,0,0,0.35)',
            zIndex: 1000,
          }}
        />
        <Dialog.Content
          data-testid="edit-parts-modal"
          style={{
            position: 'fixed',
            inset: 0,
            background: '#fff',
            zIndex: 1001,
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
              Edit Parts
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
          <div style={{ overflowY: 'auto', flex: 1 }}>
            <table
              style={{
                width: '100%',
                borderCollapse: 'collapse',
                tableLayout: 'fixed',
              }}
            >
              <colgroup>
                <col style={{ width: '10%' }} />
                <col style={{ width: '5%' }} />
                <col style={{ width: '15%' }} />
                <col style={{ width: '11%' }} />
                <col style={{ width: '15%' }} />
                <col style={{ width: '14%' }} />
                <col style={{ width: '12%' }} />
                <col style={{ width: '9%' }} />
                <col style={{ width: '9%' }} />
              </colgroup>
              <thead>
                <tr>
                  <th style={thStyle}>Name</th>
                  <th style={thStyle}>Abbr</th>
                  <th style={thStyle}>Kind / Follow</th>
                  <th style={thStyle}>Instrument</th>
                  <th style={thStyle}>Range</th>
                  <th style={thStyle}>Soundfont</th>
                  <th style={thStyle}>Volume</th>
                  <th style={thStyle}>MIDI octave</th>
                  <th style={thStyle}>Notation octave</th>
                </tr>
              </thead>
              <tbody>
                {options !== null &&
                  partDeclarations.map((declaration, index) => (
                    <PartRow
                      key={declaration.abbreviation}
                      declaration={declaration}
                      options={options}
                      allParts={allParts}
                      isFirstPart={index === 0}
                      onPartDeclarationChange={onPartDeclarationChange}
                      onShiftPartOctave={onShiftPartOctave}
                      rowIndex={index}
                      previewInstrument={previewInstrument}
                      previewPercussion={previewPercussion}
                      stopPreviewInstrument={stopPreviewInstrument}
                      previewAudioPlaying={previewAudioPlaying}
                    />
                  ))}
              </tbody>
            </table>
          </div>
        </Dialog.Content>
      </Dialog.Portal>
    </Dialog.Root>
  )
}
