import { isValidElement, type ReactElement, type ReactNode } from 'react'
import { describe, expect, it, vi } from 'vitest'
import {
  DiscardConfirmDialog,
  type DiscardConfirmDialogProps,
} from './DiscardConfirmDialog.tsx'

// Radix portals render nothing on the server and the test env is node, so the
// component is called directly and its returned element tree is inspected.

function makeProps(
  overrides: Partial<DiscardConfirmDialogProps> = {},
): DiscardConfirmDialogProps {
  return {
    open: true,
    fileName: 'song.poentaro',
    onDownloadFirst: vi.fn(),
    onConfirm: vi.fn(),
    onCancel: vi.fn(),
    ...overrides,
  }
}

function flatten(node: ReactNode): ReactElement[] {
  if (Array.isArray(node)) return node.flatMap(flatten)
  if (!isValidElement(node)) return []
  const element = node as ReactElement<{ children?: ReactNode }>
  return [element, ...flatten(element.props.children)]
}

function text(node: ReactNode): string {
  if (Array.isArray(node)) return node.map(text).join('')
  if (typeof node === 'string' || typeof node === 'number') return `${node}`
  if (!isValidElement(node)) return ''
  return text((node as ReactElement<{ children?: ReactNode }>).props.children)
}

type RootProps = { open: boolean; onOpenChange: (open: boolean) => void }

function root(props: DiscardConfirmDialogProps): ReactElement<RootProps> {
  return DiscardConfirmDialog(props) as ReactElement<RootProps>
}

function button(props: DiscardConfirmDialogProps, label: string) {
  const found = flatten(root(props)).find(
    (el) => el.type === 'button' && text(el) === label,
  ) as ReactElement<{ onClick: () => void }> | undefined
  if (!found) throw new Error(`no button ${label}`)
  return found
}

describe('DiscardConfirmDialog', () => {
  it('passes open through to the alert dialog', () => {
    expect(root(makeProps({ open: true })).props.open).toBe(true)
    expect(root(makeProps({ open: false })).props.open).toBe(false)
  })

  it('states the revert and loss consequences with the file name', () => {
    const body = text(root(makeProps()))
    expect(body).toContain('song.poentaro')
    expect(body).toContain("server's version")
    expect(body).toContain('lost')
  })

  it('calls onDownloadFirst only from Download my copy first', () => {
    const props = makeProps()
    button(props, 'Download my copy first').props.onClick()
    expect(props.onDownloadFirst).toHaveBeenCalledTimes(1)
    expect(props.onConfirm).not.toHaveBeenCalled()
    expect(props.onCancel).not.toHaveBeenCalled()
  })

  it('calls onConfirm only from Discard my changes', () => {
    const props = makeProps()
    button(props, 'Discard my changes').props.onClick()
    expect(props.onConfirm).toHaveBeenCalledTimes(1)
    expect(props.onDownloadFirst).not.toHaveBeenCalled()
    expect(props.onCancel).not.toHaveBeenCalled()
  })

  it('routes onOpenChange(false), which Escape triggers, to onCancel', () => {
    const props = makeProps()
    root(props).props.onOpenChange(false)
    expect(props.onCancel).toHaveBeenCalledTimes(1)
    root(props).props.onOpenChange(true)
    expect(props.onCancel).toHaveBeenCalledTimes(1)
  })
})
