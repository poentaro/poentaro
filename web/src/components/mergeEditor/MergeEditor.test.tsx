import type { ReactElement, ReactNode } from 'react'
import { renderToStaticMarkup } from 'react-dom/server'
import { describe, expect, it, vi } from 'vitest'

vi.mock('@monaco-editor/react', () => ({
  DiffEditor: () => <div data-testid="diff-editor-stub" />,
}))

import { MergeEditor, MergeEditorView } from './MergeEditor.tsx'
import {
  containsConflictMarkers,
  type MergeEditorProps,
} from './mergeEditorTypes.ts'

type ButtonProps = {
  children?: ReactNode
  disabled?: boolean
  onClick?: () => void
}

function findButtons(node: ReactNode): Map<string, ButtonProps> {
  const found = new Map<string, ButtonProps>()
  const walk = (current: ReactNode): void => {
    if (Array.isArray(current)) {
      current.forEach(walk)
      return
    }
    if (current === null || typeof current !== 'object') return
    const element = current as ReactElement<ButtonProps>
    if (element.type === 'button') {
      found.set(String(element.props.children), element.props)
    }
    walk(element.props.children)
  }
  walk(node)
  return found
}

function makeProps(
  overrides: Partial<MergeEditorProps> = {},
): MergeEditorProps {
  return {
    fileName: 'song.poentaro',
    theirs: 'theirs',
    initialResult: 'mine',
    hasConflictMarkers: false,
    onSave: vi.fn(),
    onKeepBoth: vi.fn(),
    onDiscard: vi.fn(),
    ...overrides,
  }
}

function view(props: MergeEditorProps, result: string): ReactElement {
  return MergeEditorView({
    ...props,
    result,
    markersRemain: containsConflictMarkers(result),
    onResultChange: vi.fn(),
  })
}

describe('containsConflictMarkers', () => {
  it.each([
    '<<<<<<< ours\nx',
    'a\n=======\nb',
    'a\n>>>>>>> theirs',
  ])('detects a marker in %j', (text) => {
    expect(containsConflictMarkers(text)).toBe(true)
  })

  it('is false for clean text', () => {
    expect(containsConflictMarkers('1 2 3\n= not a marker')).toBe(false)
  })
})

describe('MergeEditor', () => {
  it('renders with Save disabled and a hint while markers are initially present', () => {
    const html = renderToStaticMarkup(
      <MergeEditor {...makeProps({ hasConflictMarkers: true })} />,
    )
    expect(html).toContain('song.poentaro')
    expect(html).toContain('merge-editor__hint')
    expect(html).toMatch(/<button[^>]*disabled[^>]*>Save<\/button>/)
  })

  it('renders Save enabled without a hint when no markers', () => {
    const html = renderToStaticMarkup(<MergeEditor {...makeProps()} />)
    expect(html).not.toContain('merge-editor__hint')
    expect(html).not.toMatch(/<button[^>]*disabled/)
  })
})

describe('MergeEditorView', () => {
  it('disables Save while the live result has markers, enables it once gone', () => {
    const props = makeProps()
    expect(findButtons(view(props, '<<<<<<< x')).get('Save')?.disabled).toBe(
      true,
    )
    expect(findButtons(view(props, 'clean')).get('Save')?.disabled).toBe(false)
  })

  it('passes the live result to onSave', () => {
    const props = makeProps()
    findButtons(view(props, 'resolved')).get('Save')?.onClick?.()
    expect(props.onSave).toHaveBeenCalledWith('resolved')
  })

  it('fires onKeepBoth and onDiscard', () => {
    const props = makeProps()
    const buttons = findButtons(view(props, 'x'))
    buttons.get('Keep both files')?.onClick?.()
    buttons.get('Discard')?.onClick?.()
    expect(props.onKeepBoth).toHaveBeenCalledTimes(1)
    expect(props.onDiscard).toHaveBeenCalledTimes(1)
  })
})
