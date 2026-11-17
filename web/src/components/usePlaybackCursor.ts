import { useEffect } from 'react'
import { rectVariantSelector, tagSelector } from '../dataAttributes'
import type { NoteTiming } from '../poentaroWasm'
import {
  groupNoteTimingsByPart,
  resolveActiveNotes,
} from './playbackCursorPosition'

export const PLAYBACK_CURSOR_FILL = 'rgba(220,38,38,0.25)'

function noteKey(sourcePartIndex: number, noteId: number): string {
  return `${sourcePartIndex}:${noteId}`
}

// Minimal DOM surface `clearStaleHighlights` needs — kept narrow (rather
// than typed against `Element`/`SVGRectElement` directly) so it's testable
// with plain duck-typed objects, no jsdom required.
interface CursorGroupLike {
  getAttribute(name: string): string | null
}
interface CursorRectLike {
  closest(selector: string): CursorGroupLike | null
  setAttribute(name: string, value: string): void
}
interface CursorContainerLike {
  querySelectorAll(selector: string): Iterable<CursorRectLike>
}

/**
 * Ground-truth sweep: finds every rect the DOM currently shows as
 * highlighted (fill === `PLAYBACK_CURSOR_FILL`) and turns off whichever
 * ones aren't in `keep` (a set of `"sourcePartIndex:noteId"` keys).
 *
 * Deliberately doesn't trust a remembered set of "currently active" keys
 * instead — React keys the note `<g>` groups by array index
 * (`renderSvgElement` in `PreviewSvgRenderer.tsx`), and always renders
 * `fill="transparent"` as a literal prop on `playbackCursorRect`. Since the
 * highlight is applied imperatively via `setAttribute` (bypassing React), a
 * re-render that doesn't change that literal prop leaves the manual fill
 * untouched; if the score also reshuffles note order, the same DOM node can
 * get reused for a *different* note's data, and a lookup keyed on the old
 * (sourcePartIndex, noteId) would never find it again — leaving a stale
 * cursor stuck on an unrelated note. Reading each highlighted rect's own
 * current attributes instead sidesteps that entirely.
 */
export function clearStaleHighlights(
  container: CursorContainerLike,
  keep: Set<string>,
): void {
  const highlighted = container.querySelectorAll(
    `${tagSelector('note')} ${rectVariantSelector('playback-cursor-rect')}[fill="${PLAYBACK_CURSOR_FILL}"]`,
  )
  for (const rect of highlighted) {
    const group = rect.closest(tagSelector('note'))
    const partIndex = group?.getAttribute('data-part-index')
    const noteId = group?.getAttribute('data-note-id')
    const key = partIndex && noteId ? `${partIndex}:${noteId}` : null
    if (!key || !keep.has(key)) {
      rect.setAttribute('fill', 'transparent')
    }
  }
}

/**
 * Imperatively toggles the `fill` of each currently-sounding note/rest's
 * background rect (`data-tag="note"` groups, see `PreviewSvgRenderer.tsx`)
 * in sync with `audio`'s playback position, independently per part — the
 * per-note replacement for the old measure-level playhead. Runs outside
 * React state/rendering (rAF, direct attribute writes) since it updates
 * every animation frame.
 */
export function usePlaybackCursor(
  containerRef: React.RefObject<HTMLDivElement | null>,
  audio: HTMLAudioElement | null | undefined,
  noteTimings: NoteTiming[] | undefined,
) {
  useEffect(() => {
    const container = containerRef.current
    if (!audio || !container || !noteTimings || noteTimings.length === 0) {
      return
    }

    const timingsByPart = groupNoteTimingsByPart(noteTimings)
    let activeKeys = new Set<string>()
    let rafId: number | null = null

    const setHighlight = (
      sourcePartIndex: number,
      noteId: number,
      on: boolean,
    ) => {
      const rects = container.querySelectorAll<SVGRectElement>(
        `${tagSelector('note', { sourcePartIndex, noteId })} ${rectVariantSelector('playback-cursor-rect')}`,
      )
      for (const rect of rects) {
        rect.setAttribute('fill', on ? PLAYBACK_CURSOR_FILL : 'transparent')
      }
    }

    const scrollIntoViewIfNeeded = (
      sourcePartIndex: number,
      noteId: number,
    ) => {
      const rect = container.querySelector<SVGRectElement>(
        `${tagSelector('note', { sourcePartIndex, noteId })} ${rectVariantSelector('playback-cursor-rect')}`,
      )
      if (!rect) return
      const noteBounds = rect.getBoundingClientRect()
      const viewBounds = container.getBoundingClientRect()
      const isVisible =
        noteBounds.top >= viewBounds.top &&
        noteBounds.bottom <= viewBounds.bottom
      if (!isVisible) {
        rect.scrollIntoView({ block: 'center', inline: 'nearest' })
      }
    }

    const clearActive = () => {
      clearStaleHighlights(container, new Set())
      activeKeys = new Set()
    }

    const updatePosition = () => {
      const active = resolveActiveNotes(audio.currentTime, timingsByPart)
      const nextKeys = new Set(
        active.map((a) => noteKey(a.sourcePartIndex, a.noteId)),
      )
      clearStaleHighlights(container, nextKeys)
      let newlyActive: { sourcePartIndex: number; noteId: number } | null = null
      for (const a of active) {
        const key = noteKey(a.sourcePartIndex, a.noteId)
        if (!activeKeys.has(key)) {
          setHighlight(a.sourcePartIndex, a.noteId, true)
          newlyActive ??= a
        }
      }
      activeKeys = nextKeys
      if (newlyActive) {
        scrollIntoViewIfNeeded(newlyActive.sourcePartIndex, newlyActive.noteId)
      }
    }

    const tick = () => {
      updatePosition()
      rafId = requestAnimationFrame(tick)
    }
    const start = () => {
      if (rafId === null) rafId = requestAnimationFrame(tick)
    }
    const stop = () => {
      if (rafId !== null) {
        cancelAnimationFrame(rafId)
        rafId = null
      }
      clearActive()
    }

    audio.addEventListener('play', start)
    audio.addEventListener('pause', stop)
    audio.addEventListener('ended', stop)
    if (!audio.paused) start()

    return () => {
      audio.removeEventListener('play', start)
      audio.removeEventListener('pause', stop)
      audio.removeEventListener('ended', stop)
      stop()
    }
  }, [containerRef, audio, noteTimings])
}
