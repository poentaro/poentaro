import { useCallback, useState } from 'react'
import { MOBILE_BREAKPOINT_QUERY, useMediaQuery } from './useMediaQuery'

export type DiffLayout = 'side-by-side' | 'inline'

// Named for the part stream editor, the first diff view; kept so the
// choice people already made carries over to every diff view.
const STORAGE_KEY = 'poentaro:part-stream-diff-layout:v1'

function load(): DiffLayout | null {
  try {
    const stored = localStorage.getItem(STORAGE_KEY)
    return stored === 'side-by-side' || stored === 'inline' ? stored : null
  } catch {
    return null
  }
}

function save(layout: DiffLayout) {
  try {
    localStorage.setItem(STORAGE_KEY, layout)
  } catch {
    // Storage may be blocked; the choice just won't persist.
  }
}

/** Whether diffs are shown side by side or inline (one choice for every
 * `PoentaroDiffEditor`). Until the
 * user picks one (remembered in this browser only), it follows the viewport:
 * inline on mobile, side by side otherwise. */
export function useDiffLayout() {
  const isMobile = useMediaQuery(MOBILE_BREAKPOINT_QUERY)
  const [chosen, setChosen] = useState<DiffLayout | null>(load)
  const layout = chosen ?? (isMobile ? 'inline' : 'side-by-side')
  const toggle = useCallback(() => {
    const next = layout === 'inline' ? 'side-by-side' : 'inline'
    save(next)
    setChosen(next)
  }, [layout])
  return { layout, toggle }
}
