import { type RefObject, useEffect } from 'react'

/** How long a resize must settle before `onWidthChange` hears of it, so
 * dragging a pane divider doesn't re-render the score on every frame. */
const SETTLE_MS = 150

/** Reports `ref`'s content width (padding excluded) on mount and whenever it
 * settles on a new width. */
export function useElementWidth(
  ref: RefObject<HTMLElement | null>,
  onWidthChange: ((widthPx: number) => void) | undefined,
): void {
  useEffect(() => {
    const element = ref.current
    if (!element || !onWidthChange) return
    let timer: number | undefined
    const observer = new ResizeObserver(([entry]) => {
      if (!entry) return
      const width = entry.contentRect.width
      window.clearTimeout(timer)
      timer = window.setTimeout(() => onWidthChange(width), SETTLE_MS)
    })
    observer.observe(element)
    onWidthChange(element.clientWidth)
    return () => {
      window.clearTimeout(timer)
      observer.disconnect()
    }
  }, [ref, onWidthChange])
}
