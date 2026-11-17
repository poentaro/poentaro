import * as Tooltip from '@radix-ui/react-tooltip'
import { ListMusic } from 'lucide-react'
import { useState } from 'react'
import { poentaroWasm, type StreamablePart } from '../../poentaroWasm'
import { ensureWasmInit } from '../../wasmInit'
import { ResponsiveMenu } from '../ResponsiveMenu'
import './PartFocusEditor.css'

const LABEL = 'Edit part as stream'

interface PartFocusMenuProps {
  source: string
  /** Called with the chosen part and its extracted stream. */
  onChoose: (part: StreamablePart, stream: string) => void
}

/** The editor toolbar's part picker for the part focus editor: every
 * declared part, in declaration order; parts Rust marks ineligible are
 * listed but disabled, with Rust's reason. */
export function PartFocusMenu({ source, onChoose }: PartFocusMenuProps) {
  const [parts, setParts] = useState<StreamablePart[] | null>(null)

  const loadParts = (open: boolean) => {
    if (!open) return
    void ensureWasmInit().then(() =>
      setParts(poentaroWasm().listStreamableParts(source)),
    )
  }

  const choose = (part: StreamablePart) => {
    const response = poentaroWasm().extractPartStream(source, part.abbreviation)
    if (response.tag === 'ok') onChoose(part, response.val.text)
  }

  return (
    <ResponsiveMenu.Root onOpenChange={loadParts}>
      <Tooltip.Root>
        <Tooltip.Trigger asChild>
          <ResponsiveMenu.Trigger asChild>
            <button
              type="button"
              className="editor-toolbar-button"
              aria-label={LABEL}
            >
              <ListMusic size={14} aria-hidden="true" />
            </button>
          </ResponsiveMenu.Trigger>
        </Tooltip.Trigger>
        <Tooltip.Portal>
          <Tooltip.Content
            className="editor-toolbar-tooltip-content"
            sideOffset={4}
          >
            {LABEL}
          </Tooltip.Content>
        </Tooltip.Portal>
      </Tooltip.Root>
      <ResponsiveMenu.Content
        className="export-menu-list"
        align="start"
        sideOffset={4}
        title={LABEL}
      >
        {parts?.length === 0 && (
          <ResponsiveMenu.Label className="export-menu-section-title">
            No parts declared
          </ResponsiveMenu.Label>
        )}
        {parts?.map((part) => (
          <ResponsiveMenu.Item
            key={part.abbreviation}
            className="export-menu-item part-focus-menu-item"
            disabled={part.ineligibleReason !== undefined}
            onSelect={() => choose(part)}
            data-testid="part-focus-part-item"
            data-abbreviation={part.abbreviation}
          >
            <span className="part-focus-menu-name">
              {part.displayName} [{part.abbreviation}]
            </span>
            <span className="part-focus-menu-kind">{part.kind}</span>
            {part.ineligibleReason !== undefined && (
              <span className="part-focus-menu-reason">
                {part.ineligibleReason}
              </span>
            )}
          </ResponsiveMenu.Item>
        ))}
      </ResponsiveMenu.Content>
    </ResponsiveMenu.Root>
  )
}
