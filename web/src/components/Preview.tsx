import { useEffect, useRef, useState } from 'react'
import type { PreviewProps } from './Preview.types'
import { PreviewPages } from './PreviewPages'
import { handlePreviewClick } from './previewClickHandler'
import {
  applyPersistedLyricLabelHighlights,
  applyPersistedPartLabelHighlights,
} from './previewLabelRangeHighlights'
import {
  applyPersistedLyricHighlights,
  applyPersistedNoteHighlights,
} from './previewRangeHighlights'
import { findRevealTarget } from './previewRevealTarget'
import { useElementWidth } from './useElementWidth'
import { usePlaybackCursor } from './usePlaybackCursor'
import { usePreviewClickSelection } from './usePreviewClickSelection'

export type { LyricCell, NoteCell } from './previewSelection'

export function Preview({
  documents,
  staffScore = null,
  floatingControls = null,
  maxView = false,
  onPagesWidthChange,
  highlightedDocuments = [],
  rendering,
  audioGenerating = false,
  wavUrl = null,
  wavFilename = 'audio.wav',
  mp3Exporting = false,
  mp3Url = null,
  mp3Filename = 'audio.mp3',
  noteTimings,
  measureAudioNoteTimings,
  measureAudioElement,
  emptyMessage = 'No preview yet.',
  onRequestAudioDownload,
  onSectionLabelClick,
  onNoteRangeSelect,
  selectedNoteCells = [],
  noteSpans = [],
  onLyricRangeSelect,
  selectedLyricCells = [],
  lyricSpans = [],
  onMeasureRangeSelect,
  selectedMeasureRange = null,
  onPendingSecondClickChange,
}: PreviewProps) {
  const previewPagesRef = useRef<HTMLDivElement>(null)
  useElementWidth(previewPagesRef, onPagesWidthChange)
  const [audioElement, setAudioElement] = useState<HTMLAudioElement | null>(
    null,
  )
  const noteSpansRef = useRef(noteSpans)
  noteSpansRef.current = noteSpans
  const lyricSpansRef = useRef(lyricSpans)
  lyricSpansRef.current = lyricSpans
  // See the scroll-to-selection effect below for why this exists alongside
  // `suppressNextRevealRef`.
  const suppressedRangeRef = useRef<typeof selectedMeasureRange>(null)

  // wavUrl/mp3Url are mutually exclusive (see `usePoentaroWorkerAudioActions`),
  // so whichever is set is the one inline player below renders. noteTimings
  // applies to whichever format that is — the worker's `audio`/`mp3`
  // responses each pair their bytes with a matching `noteTimings` computed
  // straight from source (codec-independent), so the cursor animates the
  // same way regardless of format.
  const audioUrl = wavUrl ?? mp3Url
  const audioFilename = wavUrl ? wavFilename : mp3Filename
  const audioBusy = wavUrl ? audioGenerating : mp3Exporting
  const audioNoteTimings = audioUrl ? noteTimings : undefined

  usePlaybackCursor(previewPagesRef, audioElement, audioNoteTimings)
  usePlaybackCursor(
    previewPagesRef,
    measureAudioElement,
    measureAudioNoteTimings,
  )
  const {
    anchorStateRef,
    suppressNextRevealRef,
    pendingSecondClick,
    setPendingSecondClick,
  } = usePreviewClickSelection(
    previewPagesRef,
    noteSpans,
    onNoteRangeSelect,
    onLyricRangeSelect,
    lyricSpans,
    onMeasureRangeSelect,
    staffScore,
    selectedMeasureRange,
  )
  useEffect(() => {
    onPendingSecondClickChange?.(pendingSecondClick)
  }, [pendingSecondClick, onPendingSecondClickChange])
  const onSectionLabelClickRef = useRef(onSectionLabelClick)
  onSectionLabelClickRef.current = onSectionLabelClick

  useEffect(() => {
    if (audioBusy && audioElement && !audioElement.paused) audioElement.pause()
  }, [audioBusy, audioElement])

  // Scrolls the preview to the current selection. Prefers the amber
  // caret-only highlight rect (present only when `highlightedDocuments` is
  // populated) so the highlighted measure lands dead center; otherwise
  // falls back to the plain document's own `[data-tag="measure"]` group for
  // `selectedMeasureRange.revealMeasureIndex`, which exists regardless of
  // highlight state — covering range selections (e.g. section/sequence
  // jumps) that deliberately opt out of the caret-only highlight but still
  // need the preview to scroll to them. `revealMeasureIndex` matters
  // because a chain selection's own `start` is wherever the chain's
  // earliest entry sits in document order, not necessarily where the user
  // navigated to (see `measureRangeInSpanWithReveal`).
  //
  // Skipped once whenever `suppressNextRevealRef` is armed (see
  // `HandlePreviewClickArgs`'s doc comment): a click-and-click gesture's
  // anchoring click self-commits a Monaco selection just like any other,
  // which — after `notifySelection`'s debounce — lands here and would
  // otherwise auto-scroll the preview back to the anchor's own measure.
  // That fights the user's manual scroll toward their second click (e.g.
  // onto another page), silently relocating whatever's under their pointer
  // before that second click lands — see
  // `note-range-select-crosses-page.feature`'s regression coverage. A
  // one-shot flag rather than a persistent `anchorStateRef.current !== null`
  // check: the anchor stays live until a second click or Escape, so gating
  // on that directly would keep suppressing every *later*, unrelated reveal
  // too, for as long as an old anchor from a single, never-followed-up
  // click happens to still be sitting there.
  //
  // The one-shot flag alone isn't enough, though: `selectedMeasureRange`
  // landing is only the *first* of potentially several effect re-runs that
  // one self-commit sets off — `usePoentaroWorkerRenderRequests`'s
  // highlight-render effect reacts to that same `selectedMeasureRange`
  // change and (whether or not it ends up posting a worker request) calls
  // `setHighlightedDocuments`, which is this effect's other dependency and
  // so re-runs it a second time with the flag already consumed. Consuming
  // the flag once would let that second run scroll unsuppressed, right back
  // to wherever the (empty, until this commit) `selectedMeasureRange` was
  // last pointing — see `click-and-click-range-selection-preserves-scroll.feature`'s
  // regression coverage. `suppressedRangeRef` remembers *which*
  // `selectedMeasureRange` the flag was armed for and keeps suppressing
  // every re-run that still carries that same reference, however many
  // ticks the commit's `documents`/`highlightedDocuments` settling takes —
  // only a genuinely different `selectedMeasureRange` (a new, unrelated
  // selection) clears it and lets reveals resume.
  // biome-ignore lint/correctness/useExhaustiveDependencies: documents/highlightedDocuments aren't read in the body, but must stay listed so this re-runs after they swap in fresh SVG DOM (see comment above effect near the top of this file).
  useEffect(() => {
    if (selectedMeasureRange === null) {
      suppressedRangeRef.current = null
      return
    }
    if (suppressNextRevealRef.current) {
      suppressNextRevealRef.current = false
      suppressedRangeRef.current = selectedMeasureRange
      return
    }
    if (suppressedRangeRef.current === selectedMeasureRange) return
    suppressedRangeRef.current = null

    const frameId = requestAnimationFrame(() => {
      const container = previewPagesRef.current
      if (!container) return
      const target = findRevealTarget(
        container,
        selectedMeasureRange,
        staffScore,
      )
      target?.scrollIntoView({
        block: 'center',
        inline: 'nearest',
      })
    })

    return () => cancelAnimationFrame(frameId)
  }, [selectedMeasureRange, documents, highlightedDocuments, staffScore])

  // Re-applies the note range-select highlight declaratively from
  // `selectedNoteCells` on every relevant render, rather than leaving it as
  // a one-shot imperative toggle on the commit click — a re-render can swap
  // in fresh SVG DOM (e.g. `documents`/`highlightedDocuments` changing after
  // the Monaco selection this range-select pushed), which would silently
  // wipe any dataset attribute set only during the selection itself.
  // biome-ignore lint/correctness/useExhaustiveDependencies: documents/highlightedDocuments aren't read in the body, but must stay listed so this re-runs after they swap in fresh SVG DOM (see comment above).
  useEffect(() => {
    const container = previewPagesRef.current
    if (!container) return
    applyPersistedNoteHighlights(container, selectedNoteCells)
    applyPersistedPartLabelHighlights(
      container,
      noteSpansRef.current,
      selectedNoteCells,
    )
  }, [selectedNoteCells, documents, highlightedDocuments])

  // Mirrors the effect above for lyric syllable selection — independent of
  // `selectedNoteCells`, since a lyric selection never drives note
  // highlighting and vice versa (see `useLyricSelection`).
  // biome-ignore lint/correctness/useExhaustiveDependencies: documents/highlightedDocuments aren't read in the body, but must stay listed so this re-runs after they swap in fresh SVG DOM (see comment above).
  useEffect(() => {
    const container = previewPagesRef.current
    if (!container) return
    applyPersistedLyricHighlights(container, selectedLyricCells)
    applyPersistedLyricLabelHighlights(
      container,
      lyricSpansRef.current,
      selectedLyricCells,
    )
  }, [selectedLyricCells, documents, highlightedDocuments])

  const activeDocs =
    highlightedDocuments.length > 0 ? highlightedDocuments : documents

  return (
    <div className="preview">
      {audioUrl ? (
        <div
          className={
            audioBusy
              ? 'preview-audio preview-audio--generating'
              : 'preview-audio'
          }
          aria-busy={audioBusy || undefined}
        >
          {/* biome-ignore lint/a11y/useMediaCaption: synthesized score preview has no captions track */}
          <audio
            ref={setAudioElement}
            className="preview-audio-player"
            controls
            src={audioUrl}
            tabIndex={audioBusy ? -1 : undefined}
          />
          <button
            type="button"
            className="preview-audio-download"
            data-testid="preview-audio-download-button"
            onClick={() => onRequestAudioDownload?.(audioUrl, audioFilename)}
            tabIndex={audioBusy ? -1 : undefined}
          >
            Download
          </button>
        </div>
      ) : null}
      <div className="preview-pages-wrapper">
        {floatingControls ? (
          <div className="preview-floating-controls">{floatingControls}</div>
        ) : null}
        {pendingSecondClick ? (
          <div
            className="pending-second-click-banner"
            data-testid="pending-second-click-banner"
          >
            Click again to select a range
          </div>
        ) : null}
        {/* biome-ignore lint/a11y/noStaticElementInteractions: click-and-click select notes uses mousedown (not the browser's synthesized `click` — see `handlePreviewClick`'s doc comment for why) plus a document-level mousemove for the hover preview — not a standard interactive role */}
        <div
          className={
            maxView ? 'preview-pages preview-pages--max' : 'preview-pages'
          }
          ref={previewPagesRef}
          onMouseDown={(e) =>
            handlePreviewClick(e, {
              anchorStateRef,
              suppressNextRevealRef,
              previewPagesRef,
              onPendingSecondClickChange: setPendingSecondClick,
              noteSpans,
              lyricSpans,
              onSectionLabelClick: onSectionLabelClickRef.current,
              staffMeasures: staffScore?.measures,
              onNoteRangeSelect,
              onLyricRangeSelect,
              onMeasureRangeSelect,
            })
          }
        >
          {(staffScore?.pages ?? activeDocs).length === 0 && !rendering ? (
            <p className="preview-empty">{emptyMessage}</p>
          ) : null}
          <PreviewPages documents={activeDocs} staffScore={staffScore} />
        </div>
        {rendering ? (
          <div
            className="preview-render-spinner"
            role="status"
            aria-label="Rendering"
          />
        ) : null}
      </div>
    </div>
  )
}
