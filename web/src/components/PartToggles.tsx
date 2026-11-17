import * as Tooltip from '@radix-ui/react-tooltip'
import { ChevronDown, ChevronRight, Headphones } from 'lucide-react'
import { useState } from 'react'
import type { PartInfo } from '../types'
import './PartToggles.css'

interface PartTogglesProps {
  parts: PartInfo[]
  hiddenParts: ReadonlySet<string>
  /** Rendered between the "Parts" label and the pills. */
  presetMenu?: React.ReactNode
  soloedParts: ReadonlySet<string>
  onPartToggle: (abbreviation: string, enabled: boolean) => void
  onSoloToggle: (abbreviation: string, soloed: boolean) => void
}

function TooltipLabel({
  tooltip,
  children,
}: {
  tooltip: React.ReactNode
  children: React.ReactNode
}) {
  return (
    <Tooltip.Root>
      <Tooltip.Trigger asChild>{children}</Tooltip.Trigger>
      <Tooltip.Portal>
        <Tooltip.Content className="part-toggle-tooltip-content" sideOffset={4}>
          {tooltip}
        </Tooltip.Content>
      </Tooltip.Portal>
    </Tooltip.Root>
  )
}

interface TogglePillProps {
  label: string
  displayName: string
  /** Lyric parts only sing along to their target; they get a distinct label. */
  lyric: boolean
  visible: boolean
  soloed: boolean
  anySoloed: boolean
  onVisibleChange: (visible: boolean) => void
  onSoloChange: (soloed: boolean) => void
}

/** One pill with two buttons: the label shows/hides the part, the headphones
 * solo it. */
function TogglePill({
  label,
  displayName,
  lyric,
  visible,
  soloed,
  anySoloed,
  onVisibleChange,
  onSoloChange,
}: TogglePillProps) {
  return (
    <div
      className={[
        'part-toggle-pill',
        lyric ? 'part-toggle-pill--lyric' : '',
        anySoloed && !soloed ? 'part-toggle-pill--silenced' : '',
      ].join(' ')}
    >
      <TooltipLabel
        tooltip={
          <>
            <span>{displayName}</span>
            <span className="part-toggle-tooltip-hint">
              {visible ? 'Click to hide' : 'Click to show'}
            </span>
          </>
        }
      >
        <label className="part-toggle-segment part-toggle-segment--eye part-toggle-abbr">
          <input
            type="checkbox"
            checked={visible}
            onChange={(event) => onVisibleChange(event.target.checked)}
          />
          {label}
        </label>
      </TooltipLabel>
      <TooltipLabel tooltip="Solo">
        <label className="part-toggle-segment part-toggle-segment--headphones">
          <input
            type="checkbox"
            checked={soloed}
            onChange={(event) => onSoloChange(event.target.checked)}
          />
          <Headphones size={14} aria-hidden="true" />
        </label>
      </TooltipLabel>
    </div>
  )
}

export function PartToggles({
  parts,
  hiddenParts,
  presetMenu,
  soloedParts,
  onPartToggle,
  onSoloToggle,
}: PartTogglesProps) {
  const [collapsed, setCollapsed] = useState(false)

  if (parts.length === 0) {
    return null
  }

  return (
    <Tooltip.Provider delayDuration={400}>
      <fieldset
        className={[
          'part-toggles',
          collapsed ? 'part-toggles--collapsed' : '',
        ].join(' ')}
      >
        <legend className="visually-hidden">Parts</legend>
        <button
          type="button"
          className={[
            'workspace-toolbar-label',
            'workspace-toolbar-label--toggle',
            collapsed ? 'workspace-toolbar-label--toggle-fill' : '',
          ].join(' ')}
          onClick={() => setCollapsed((value) => !value)}
          aria-expanded={!collapsed}
        >
          {collapsed ? (
            <ChevronRight size={12} aria-hidden="true" />
          ) : (
            <ChevronDown size={12} aria-hidden="true" />
          )}
          Parts
        </button>
        {presetMenu}
        {collapsed ? null : (
          <ul className="part-toggles-list toolbar-scroll-list">
            {parts.map((part) => (
              <li key={part.abbreviation}>
                <TogglePill
                  label={part.abbreviation}
                  displayName={part.displayName}
                  lyric={!part.sounds}
                  visible={!hiddenParts.has(part.abbreviation)}
                  soloed={soloedParts.has(part.abbreviation)}
                  anySoloed={soloedParts.size > 0}
                  onVisibleChange={(visible) =>
                    onPartToggle(part.abbreviation, visible)
                  }
                  onSoloChange={(soloed) =>
                    onSoloToggle(part.abbreviation, soloed)
                  }
                />
              </li>
            ))}
          </ul>
        )}
      </fieldset>
    </Tooltip.Provider>
  )
}
