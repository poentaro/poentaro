import { CheckIcon, ChevronDownIcon } from '@radix-ui/react-icons'
import { useState } from 'react'
import type { PresetSummary } from '../poentaroWasm'
import { NewPresetDialog } from './NewPresetDialog'
import { ResponsiveMenu } from './ResponsiveMenu'

const DEFAULT_LABEL = 'Default'

export interface PresetMenuProps {
  presets: PresetSummary[]
  /** The preset in force; `null` is Default. */
  presetInForce: string | null
  onChoose: (presetName: string | null) => void
  /** Absent in a read-only file, which can't gain a preset. */
  onCreatePreset?: (name: string) => void
  /** Rust's reason a new preset can't take this name, if it can't. */
  nameError: (name: string) => string | undefined
}

/** Chooses which `preset` the preview, playback and exports follow. Sits
 * in the Parts row, since the pills then edit that preset. Hidden while the
 * file has no presets and can't gain one. */
export function PresetMenu({
  presets,
  presetInForce,
  onChoose,
  onCreatePreset,
  nameError,
}: PresetMenuProps) {
  const [naming, setNaming] = useState(false)
  if (presets.length === 0 && !onCreatePreset) return null
  const entries: Array<string | null> = [
    null,
    ...presets.map((preset) => preset.name),
  ]
  return (
    <>
      <ResponsiveMenu.Root>
        <ResponsiveMenu.Trigger asChild>
          <button
            type="button"
            className="preset-menu-trigger"
            aria-label={`Preset: ${presetInForce ?? DEFAULT_LABEL}`}
            data-testid="preset-menu"
          >
            {presetInForce ?? DEFAULT_LABEL}
            <ChevronDownIcon className="export-menu-caret" aria-hidden="true" />
          </button>
        </ResponsiveMenu.Trigger>
        <ResponsiveMenu.Content
          className="export-menu-list"
          align="start"
          sideOffset={4}
          title="Preset"
        >
          {entries.map((name) => (
            <ResponsiveMenu.Item
              key={name ?? ''}
              className="export-menu-item notation-toggle-item"
              onSelect={() => onChoose(name)}
              data-testid="preset-menu-item"
            >
              {name ?? DEFAULT_LABEL}
              {name === presetInForce && (
                <CheckIcon
                  className="notation-toggle-check"
                  aria-label="selected"
                />
              )}
            </ResponsiveMenu.Item>
          ))}
          {onCreatePreset && (
            <ResponsiveMenu.Item
              className="export-menu-item preset-menu-action"
              onSelect={() => setNaming(true)}
            >
              New preset…
            </ResponsiveMenu.Item>
          )}
        </ResponsiveMenu.Content>
      </ResponsiveMenu.Root>
      {onCreatePreset && (
        <NewPresetDialog
          open={naming}
          onOpenChange={setNaming}
          onCreate={onCreatePreset}
          nameError={nameError}
        />
      )}
    </>
  )
}
