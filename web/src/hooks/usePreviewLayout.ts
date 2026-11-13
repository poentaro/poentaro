import { useCallback, useMemo, useState } from 'react'
import type { PageLayout } from '../poentaroWasm'

/** `pages`: the score on A4 pages, as it prints. `max`: one continuous page
 * as wide as the preview, so systems wrap to the screen. */
export type PreviewView = 'pages' | 'max'

export interface PreviewLayout {
  view: PreviewView
  setView: (view: PreviewView) => void
  /** Reported by `Preview` whenever its pages area resizes. */
  setContainerWidth: (widthPx: number) => void
  /** What the preview renders on; exports always use A4. */
  pageLayout: PageLayout
}

const STORAGE_KEY = 'poentaro:preview-view'

function storedView(): PreviewView {
  try {
    return localStorage.getItem(STORAGE_KEY) === 'max' ? 'max' : 'pages'
  } catch {
    return 'pages'
  }
}

const A4: PageLayout = { tag: 'a4' }

/** The preview's page view, remembered across visits, plus the page layout
 * to render it on. Only max view follows the preview's width, so resizing
 * re-renders nothing on A4 pages. */
export function usePreviewLayout(): PreviewLayout {
  const [view, setViewState] = useState<PreviewView>(storedView)
  const [containerWidth, setContainerWidthState] = useState(0)
  const setView = useCallback((next: PreviewView) => {
    setViewState(next)
    try {
      localStorage.setItem(STORAGE_KEY, next)
    } catch {
      // A remembered view is a convenience; the choice still applies now.
    }
  }, [])
  const setContainerWidth = useCallback(
    (widthPx: number) => setContainerWidthState(Math.floor(widthPx)),
    [],
  )
  const maxWidth = view === 'max' ? containerWidth : 0
  const pageLayout = useMemo<PageLayout>(
    () =>
      maxWidth > 0 ? { tag: 'continuous', val: { widthPx: maxWidth } } : A4,
    [maxWidth],
  )
  return { view, setView, setContainerWidth, pageLayout }
}
