import {
  CheckIcon,
  ChevronDownIcon,
  FileIcon,
  WidthIcon,
} from '@radix-ui/react-icons'
import type { ReactNode } from 'react'
import type { PreviewView } from '../hooks/usePreviewLayout'
import { ResponsiveMenu } from './ResponsiveMenu'

interface PageViewToggleProps {
  view: PreviewView
  onChange: (view: PreviewView) => void
}

const OPTIONS: { value: PreviewView; icon: ReactNode; label: string }[] = [
  { value: 'pages', icon: <FileIcon />, label: 'A4' },
  { value: 'max', icon: <WidthIcon />, label: 'Max' },
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

/** Switches the preview between A4 pages and max view, one continuous page
 * as wide as the preview. Sits beside `NotationToggle` and shares its look. */
export function PageViewToggle({ view, onChange }: PageViewToggleProps) {
  const current = OPTIONS.find((option) => option.value === view)
  return (
    <ResponsiveMenu.Root>
      <ResponsiveMenu.Trigger asChild>
        <button
          type="button"
          className="preview-export-btn notation-toggle-trigger"
          aria-label={`Page view: ${current?.label ?? view}`}
          data-testid="page-view-toggle"
        >
          {current && <OptionLabel icon={current.icon} label={current.label} />}
          <ChevronDownIcon className="export-menu-caret" aria-hidden="true" />
        </button>
      </ResponsiveMenu.Trigger>
      <ResponsiveMenu.Content
        className="export-menu-list"
        align="end"
        sideOffset={4}
        title="Page view"
      >
        {OPTIONS.map(({ value, icon, label }) => (
          <ResponsiveMenu.Item
            key={value}
            className="export-menu-item notation-toggle-item"
            onSelect={() => onChange(value)}
            data-testid={`page-view-toggle-${value}`}
          >
            <OptionLabel icon={icon} label={label} />
            {value === view && (
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
