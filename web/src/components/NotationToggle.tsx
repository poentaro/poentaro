import { CheckIcon, ChevronDownIcon } from '@radix-ui/react-icons'
import { CaseUpper, Guitar } from 'lucide-react'
import type { ReactNode } from 'react'
import type { Notation } from '../hooks/useStaffNotation'
import { HarmonicaIcon } from './icons/HarmonicaIcon'
import { ResponsiveMenu } from './ResponsiveMenu'

interface NotationToggleProps {
  notation: Notation
  onChange: (notation: Notation) => void
}

// Numbered and the clefs are notation glyphs; the rest are line icons sized to
// match them (see `.notation-toggle-icon svg`).
const OPTIONS: { value: Notation; icon: ReactNode; label: string }[] = [
  { value: 'numbered', icon: '簡', label: '123' },
  { value: 'violin', icon: '𝄞', label: 'Violin' },
  { value: 'cello', icon: '𝄢', label: 'Cello' },
  { value: 'abc', icon: <CaseUpper />, label: 'ABC' },
  { value: 'guitar', icon: <Guitar />, label: 'Guitar' },
  { value: 'harmonica', icon: <HarmonicaIcon />, label: 'Harmonica' },
]

function OptionLabel({ icon, label }: { icon: ReactNode; label: string }) {
  return (
    <>
      <span className="notation-toggle-icon" aria-hidden="true">
        {icon}
      </span>
      {label}
    </>
  )
}

/** Switches the preview between numbered notation, western staff notation for a violin
 * (treble clef) or a cello (bass clef), ABC (the
 * numbered layout with letter names in place of degree numbers), Guitar (the
 * numbered layout with each degree followed by its letter name, `1 (C)`, and
 * a chord diagram at each chord symbol), and Harmonica
 * (the numbered layout with chromatic harmonica tabs). A dropdown rather than a
 * row of buttons so it stays compact as views are added; `ResponsiveMenu`
 * opens it as a full-screen drawer on mobile. */
export function NotationToggle({ notation, onChange }: NotationToggleProps) {
  const current = OPTIONS.find((option) => option.value === notation)
  return (
    <ResponsiveMenu.Root>
      <ResponsiveMenu.Trigger asChild>
        <button
          type="button"
          className="preview-export-btn notation-toggle-trigger"
          aria-label={`Notation: ${current?.label ?? notation}`}
          data-testid="notation-toggle"
        >
          {current && <OptionLabel icon={current.icon} label={current.label} />}
          <ChevronDownIcon className="export-menu-caret" aria-hidden="true" />
        </button>
      </ResponsiveMenu.Trigger>
      <ResponsiveMenu.Content
        className="export-menu-list"
        align="end"
        sideOffset={4}
        title="Notation"
      >
        {OPTIONS.map(({ value, icon, label }) => (
          <ResponsiveMenu.Item
            key={value}
            className="export-menu-item notation-toggle-item"
            onSelect={() => onChange(value)}
            data-testid={`notation-toggle-${value}`}
          >
            <OptionLabel icon={icon} label={label} />
            {value === notation && (
              <CheckIcon
                className="notation-toggle-check"
                aria-label="selected"
              />
            )}
          </ResponsiveMenu.Item>
        ))}
      </ResponsiveMenu.Content>
    </ResponsiveMenu.Root>
  )
}
