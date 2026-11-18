import { CheckIcon, ChevronDownIcon } from '@radix-ui/react-icons'
import * as R from 'remeda'
import type { PreviewLayout } from '../hooks/usePreviewLayout'
import type { ScoreOrder } from '../poentaroWasm'
import { ResponsiveMenu } from './ResponsiveMenu'

const LABELS: Record<ScoreOrder, string> = {
  written: 'Written',
  expanded: 'Expanded',
}

const OPTIONS = R.entries(LABELS)

export function ScoreOrderToggle({
  layout: { scoreOrder, setScoreOrder },
}: {
  layout: Pick<PreviewLayout, 'scoreOrder' | 'setScoreOrder'>
}) {
  return (
    <ResponsiveMenu.Root>
      <ResponsiveMenu.Trigger asChild>
        <button
          type="button"
          className="preview-export-btn notation-toggle-trigger"
          aria-label={`Score order: ${LABELS[scoreOrder]}`}
        >
          {LABELS[scoreOrder]}
          <ChevronDownIcon className="export-menu-caret" aria-hidden="true" />
        </button>
      </ResponsiveMenu.Trigger>
      <ResponsiveMenu.Content
        className="export-menu-list"
        align="end"
        sideOffset={4}
        title="Score order"
      >
        {OPTIONS.map(([value, label]) => (
          <ResponsiveMenu.Item
            key={value}
            className="export-menu-item notation-toggle-item"
            onSelect={() => setScoreOrder(value)}
          >
            {label}
            {value === scoreOrder && (
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
