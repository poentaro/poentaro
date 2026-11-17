import type { ReactElement, ReactNode } from 'react'
import { renderToStaticMarkup } from 'react-dom/server'
import { describe, expect, it, vi } from 'vitest'
import { SyncPanel } from './SyncPanel.tsx'
import type {
  LaneView,
  LaneViewStatus,
  SyncPanelProps,
} from './syncPanelTypes.ts'

const STATUSES: LaneViewStatus[] = [
  'syncing',
  'waiting',
  'possibly-stuck',
  'needs-sign-in',
  'needs-merge',
  'failed',
]

function lane(status: LaneViewStatus): LaneView {
  return {
    fileId: `f-${status}`,
    fileName: `${status}.poentaro`,
    status,
    sentence: `Sentence for ${status}`,
    messages: [{ kindLabel: 'Edit', sizeLabel: '2 KB', createdLabel: '10:01' }],
    attempts: [{ atLabel: '10:02', outcomeLabel: 'Timed out' }],
  }
}

function makeProps(lanes: LaneView[]): SyncPanelProps {
  return {
    lanes,
    onRetryNow: vi.fn(),
    onReviewAndMerge: vi.fn(),
    onKeepBoth: vi.fn(),
    onDiscard: vi.fn(),
    onDownloadCopy: vi.fn(),
  }
}

function textOf(node: ReactNode): string {
  if (node === null || node === undefined || typeof node === 'boolean')
    return ''
  if (typeof node === 'string' || typeof node === 'number') return String(node)
  if (Array.isArray(node)) return node.map(textOf).join('')
  return textOf((node as ReactElement<{ children?: ReactNode }>).props.children)
}

// SyncPanel is hook-free and made of host elements only, so its element tree
// can be walked directly to find buttons and invoke their handlers.
function findButtons(node: ReactNode): Map<string, () => void> {
  const found = new Map<string, () => void>()
  const walk = (current: ReactNode): void => {
    if (Array.isArray(current)) {
      current.forEach(walk)
      return
    }
    if (current === null || typeof current !== 'object') return
    const element = current as ReactElement<{
      children?: ReactNode
      onClick?: () => void
    }>
    if (element.type === 'button' && element.props.onClick) {
      found.set(textOf(element.props.children), element.props.onClick)
    }
    walk(element.props.children)
  }
  walk(node)
  return found
}

describe('SyncPanel', () => {
  it.each(
    STATUSES,
  )('renders the %s lane with its sentence and pill', (status) => {
    const html = renderToStaticMarkup(
      <SyncPanel {...makeProps([lane(status)])} />,
    )
    expect(html).toContain(`${status}.poentaro`)
    expect(html).toContain(`Sentence for ${status}`)
    expect(html).toContain(`data-status="${status}"`)
  })

  it('renders collapsed details with messages and attempts', () => {
    const html = renderToStaticMarkup(
      <SyncPanel {...makeProps([lane('waiting')])} />,
    )
    expect(html).toContain('<details>')
    expect(html).not.toContain('<details open')
    expect(html).toContain('Edit, 2 KB, 10:01')
    expect(html).toContain('10:02: Timed out')
  })

  it('fires retry for a retryable lane', () => {
    const props = makeProps([lane('failed')])
    const buttons = findButtons(SyncPanel(props))
    buttons.get('Retry now')?.()
    expect(props.onRetryNow).toHaveBeenCalledWith('f-failed')
    buttons.get('Download my copy')?.()
    expect(props.onDownloadCopy).toHaveBeenCalledWith('f-failed')
  })

  it('fires the merge actions for a needs-merge lane', () => {
    const props = makeProps([lane('needs-merge')])
    const buttons = findButtons(SyncPanel(props))
    buttons.get('Review and merge')?.()
    buttons.get('Keep both files')?.()
    buttons.get('Discard')?.()
    buttons.get('Download my copy')?.()
    expect(props.onReviewAndMerge).toHaveBeenCalledWith('f-needs-merge')
    expect(props.onKeepBoth).toHaveBeenCalledWith('f-needs-merge')
    expect(props.onDiscard).toHaveBeenCalledWith('f-needs-merge')
    expect(props.onDownloadCopy).toHaveBeenCalledWith('f-needs-merge')
  })

  it('renders nothing when every file is synced', () => {
    expect(SyncPanel(makeProps([]))).toBeNull()
  })

  it('offers no retry while syncing', () => {
    const buttons = findButtons(SyncPanel(makeProps([lane('syncing')])))
    expect(buttons.has('Retry now')).toBe(false)
  })
})
