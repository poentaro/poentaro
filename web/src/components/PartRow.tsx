import * as Slider from '@radix-ui/react-slider'
import { ChevronDown, ChevronUp } from 'lucide-react'
import { useState } from 'react'
import { range } from 'remeda'
import type { PartSettingOptions } from '../hooks/usePartSettingOptions'
import type { OctaveOffsetRange } from '../poentaroWasm'
import type {
  PartDeclaration,
  PartInfo,
  PartMode,
  PartSettings,
} from '../types'
import type { EditPartsModalProps } from './EditPartsModal'
import { RadixSelect, RadixSelectItem } from './RadixTableSelect'
import { SoundfontSearchModal } from './SoundfontSearchModal'

/** Every octave offset in `offsets`, highest first, labelled as the
 * `parts` line writes it (`+2`, `0`, `-1`). */
function octaveOptions(offsets: OctaveOffsetRange) {
  return range(offsets.min, offsets.max + 1)
    .reverse()
    .map((offset) => ({
      value: String(offset),
      label: offset > 0 ? `+${offset}` : String(offset),
    }))
}

export function PartRow({
  declaration,
  options,
  allParts,
  isFirstPart,
  onPartDeclarationChange,
  onShiftPartOctave,
  rowIndex,
  previewInstrument,
  previewPercussion,
  stopPreviewInstrument,
  previewAudioPlaying,
}: {
  declaration: PartDeclaration
  options: PartSettingOptions
  allParts: PartInfo[]
  isFirstPart: boolean
  onPartDeclarationChange: EditPartsModalProps['onPartDeclarationChange']
  onShiftPartOctave: EditPartsModalProps['onShiftPartOctave']
  rowIndex: number
  previewInstrument: (programNumber: number) => void
  previewPercussion: (key: number) => void
  stopPreviewInstrument: () => void
  previewAudioPlaying: boolean
}) {
  const [searchOpen, setSearchOpen] = useState(false)
  const { limits, rangeInstruments } = options
  const partInfo = allParts.find(
    (p) => p.abbreviation === declaration.abbreviation,
  )
  const precedingParts = allParts.slice(
    0,
    allParts.findIndex((p) => p.abbreviation === declaration.abbreviation),
  )

  const { settings } = declaration
  const change = (patch: Partial<PartSettings>) =>
    onPartDeclarationChange(declaration.abbreviation, { ...settings, ...patch })
  const followTarget =
    settings.mode.tag === 'follow' ? settings.mode.val : undefined
  const takesRangeInstrument =
    settings.mode.tag === 'notes' || settings.mode.tag === 'follow'

  function handleModeChange(newMode: string) {
    // The select only offers these; lyric parts are not listed here at all.
    const tag = newMode as Exclude<PartMode['tag'], 'lyrics'>
    if (tag === 'follow') {
      const defaultTarget = precedingParts[0]?.abbreviation
      if (defaultTarget !== undefined)
        change({ mode: { tag, val: defaultTarget } })
    } else {
      change({ mode: { tag } })
    }
  }

  const rowBg = rowIndex % 2 === 0 ? '#fafafa' : '#fff'

  return (
    <tr style={{ background: rowBg }}>
      <td style={tdStyle}>
        {partInfo?.displayName ?? declaration.abbreviation}
      </td>
      <td style={tdStyle}>
        <span
          style={{ fontFamily: 'var(--mono)', fontSize: '12px', color: '#666' }}
        >
          {declaration.abbreviation}
        </span>
      </td>
      <td style={tdStyle}>
        <div style={{ display: 'flex', alignItems: 'center', gap: '6px' }}>
          <RadixSelect
            value={settings.mode.tag}
            onValueChange={handleModeChange}
            placeholder="mode"
            testId={`mode-select-${declaration.abbreviation}`}
          >
            <RadixSelectItem value="chords">chords</RadixSelectItem>
            <RadixSelectItem value="notes">notes</RadixSelectItem>
            <RadixSelectItem value="percussion">percussion</RadixSelectItem>
            {!isFirstPart && (
              <RadixSelectItem value="follow">follow</RadixSelectItem>
            )}
          </RadixSelect>
          {followTarget !== undefined && precedingParts.length > 0 && (
            <RadixSelect
              value={followTarget}
              onValueChange={(target) =>
                change({ mode: { tag: 'follow', val: target } })
              }
              placeholder="target"
              testId={`follow-target-select-${declaration.abbreviation}`}
            >
              {precedingParts.map((part) => (
                <RadixSelectItem
                  key={part.abbreviation}
                  value={part.abbreviation}
                >
                  {part.abbreviation}
                </RadixSelectItem>
              ))}
            </RadixSelect>
          )}
        </div>
      </td>
      <td style={tdStyle}>
        {takesRangeInstrument && (
          <RadixSelect
            value={settings.rangeInstrument ?? NO_RANGE_INSTRUMENT}
            onValueChange={(value) =>
              change({
                rangeInstrument:
                  value === NO_RANGE_INSTRUMENT ? undefined : value,
              })
            }
            placeholder="instrument"
            testId={`instrument-select-${declaration.abbreviation}`}
          >
            <RadixSelectItem value={NO_RANGE_INSTRUMENT}>
              {followTarget === undefined ? 'none' : 'as target'}
            </RadixSelectItem>
            {rangeInstruments.map((instrument) => (
              <RadixSelectItem key={instrument.name} value={instrument.name}>
                {instrument.label}
              </RadixSelectItem>
            ))}
          </RadixSelect>
        )}
      </td>
      <td style={tdStyle}>
        <button
          type="button"
          onClick={() => setSearchOpen(true)}
          data-testid={`soundfont-select-${declaration.abbreviation}`}
          style={{
            display: 'inline-flex',
            alignItems: 'center',
            fontSize: '12px',
            fontFamily: 'var(--mono, monospace)',
            border: '1px solid #cbd5e0',
            borderRadius: '3px',
            background: '#fff',
            color: '#2d3748',
            padding: '2px 6px',
            cursor: 'pointer',
            height: '22px',
            whiteSpace: 'nowrap',
            minWidth: '80px',
          }}
        >
          {declaration.soundLabel ?? 'default sound'}
        </button>
        <SoundfontSearchModal
          open={searchOpen}
          onOpenChange={setSearchOpen}
          mode={
            settings.mode.tag === 'percussion' ? 'percussion' : 'instrument'
          }
          currentProgram={settings.program ?? null}
          onSelect={(program) => {
            change({ program: program ?? undefined })
            setSearchOpen(false)
          }}
          previewInstrument={previewInstrument}
          previewPercussion={previewPercussion}
          stopPreviewInstrument={stopPreviewInstrument}
          previewAudioPlaying={previewAudioPlaying}
        />
      </td>
      <td style={tdStyle}>
        <div style={{ display: 'flex', alignItems: 'center', gap: '6px' }}>
          <Slider.Root
            min={limits.volume.min}
            max={limits.volume.max}
            step={1}
            value={[settings.volume]}
            onValueChange={([v]) => {
              if (v !== undefined) change({ volume: v })
            }}
            data-testid={`volume-slider-${declaration.abbreviation}`}
            style={{
              position: 'relative',
              display: 'flex',
              alignItems: 'center',
              userSelect: 'none',
              touchAction: 'none',
              width: '80px',
              height: '20px',
            }}
          >
            <Slider.Track
              style={{
                background: '#e2e8f0',
                position: 'relative',
                flexGrow: 1,
                borderRadius: '9999px',
                height: '4px',
              }}
            >
              <Slider.Range
                style={{
                  position: 'absolute',
                  background: '#4a90d9',
                  borderRadius: '9999px',
                  height: '100%',
                }}
              />
            </Slider.Track>
            <Slider.Thumb
              style={{
                display: 'block',
                width: '14px',
                height: '14px',
                background: '#fff',
                border: '2px solid #4a90d9',
                borderRadius: '9999px',
                cursor: 'pointer',
                outline: 'none',
              }}
            />
          </Slider.Root>
          <span
            data-testid={`volume-value-${declaration.abbreviation}`}
            style={{
              fontSize: '11px',
              color: '#666',
              fontFamily: 'var(--mono, monospace)',
              minWidth: '28px',
            }}
          >
            {settings.volume}%
          </span>
        </div>
      </td>
      <td style={tdStyle}>
        <RadixSelect
          value={String(settings.octaveOffset)}
          onValueChange={(value) =>
            change({ octaveOffset: Number.parseInt(value, 10) })
          }
          placeholder="octave"
          testId={`octave-select-${declaration.abbreviation}`}
        >
          {octaveOptions(limits.octaveOffset).map((option) => (
            <RadixSelectItem key={option.value} value={option.value}>
              {option.label}
            </RadixSelectItem>
          ))}
        </RadixSelect>
      </td>
      <td style={tdStyle}>
        {followTarget === undefined && (
          <div style={{ display: 'flex', alignItems: 'center', gap: '2px' }}>
            <button
              type="button"
              onClick={() => onShiftPartOctave(declaration.abbreviation, -1)}
              data-testid={`notation-octave-down-${declaration.abbreviation}`}
              title="Shift notation down one octave"
              aria-label="Shift notation down one octave"
              style={octaveShiftButtonStyle}
            >
              <ChevronDown size={14} />
            </button>
            <button
              type="button"
              onClick={() => onShiftPartOctave(declaration.abbreviation, 1)}
              data-testid={`notation-octave-up-${declaration.abbreviation}`}
              title="Shift notation up one octave"
              aria-label="Shift notation up one octave"
              style={octaveShiftButtonStyle}
            >
              <ChevronUp size={14} />
            </button>
          </div>
        )}
      </td>
    </tr>
  )
}

/** The instrument picker's value for a line that names no instrument (Radix
 * Select can't hold an empty value). */
const NO_RANGE_INSTRUMENT = 'none'

const octaveShiftButtonStyle: React.CSSProperties = {
  display: 'inline-flex',
  alignItems: 'center',
  justifyContent: 'center',
  width: '22px',
  height: '22px',
  border: '1px solid #cbd5e0',
  borderRadius: '3px',
  background: '#fff',
  color: '#2d3748',
  cursor: 'pointer',
}

const tdStyle: React.CSSProperties = {
  padding: '6px 10px',
  borderBottom: '1px solid #eee',
  verticalAlign: 'middle',
  fontSize: '13px',
}
