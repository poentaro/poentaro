import type { ReactNode } from 'react'
import type { NoteTiming, SvgDocument } from '../poentaroWasm'
import type { LyricSpan, NoteSpan } from '../types'
import type { StaffScore } from '../worker/messages'
import type { LyricCell, NoteCell } from './previewSelection'

export interface PreviewProps {
  documents: SvgDocument[]
  /** Verovio's staff notation (`useStaffNotation`); replaces `documents`. */
  staffScore?: StaffScore | null
  /** Controls floated over the pages' top-right corner (`NotationToggle`). */
  floatingControls?: ReactNode
  /** Max view: one edge-to-edge page instead of A4 pages on a backdrop. */
  maxView?: boolean
  /** Hears the pages area's content width, which max view lays out to. */
  onPagesWidthChange?: (widthPx: number) => void
  highlightedDocuments?: SvgDocument[]
  rendering: boolean
  audioGenerating?: boolean
  wavUrl?: string | null
  wavFilename?: string
  /** Mirrors `audioGenerating`/`wavUrl`/`wavFilename` for the MP3 format —
   * mutually exclusive with the WAV trio (only one of `wavUrl`/`mp3Url` is
   * ever non-null), so the inline player below always shows whichever
   * format was most recently generated. */
  mp3Exporting?: boolean
  mp3Url?: string | null
  mp3Filename?: string
  /** Elapsed-seconds start/end of every sounding note/rest for whichever of `wavUrl`/`mp3Url` is set, keyed by `(source_part_index, note_id)`. */
  noteTimings?: NoteTiming[]
  /** Elapsed-seconds start/end of every sounding note/rest for the selected range's audio, keyed by `(source_part_index, note_id)`. */
  measureAudioNoteTimings?: NoteTiming[]
  /** The `<audio>` element currently playing the selected measure range, if any. */
  measureAudioElement?: HTMLAudioElement | null
  emptyMessage?: string
  /** Opens the rename-before-download modal for the inline audio player's
   * "Download" button, instead of downloading `audioUrl` immediately —
   * see `PendingDownload` in `usePoentaroWorkerTypes.ts`. */
  onRequestAudioDownload?: (url: string, filename: string) => void
  onSectionLabelClick?: (label: string) => void
  /** Fired on the commit click of a note-level click-and-click select (see
   * `getNoteAtPoint`), with every note/rest cell the gesture resolved to —
   * but only when `onMeasureRangeSelect` isn't supplied (see
   * `onLyricRangeSelect` below for why: a note selection can also cover
   * lyric syllables, which then routes through `onMeasureRangeSelect`
   * instead). */
  onNoteRangeSelect?: (selectedCells: NoteCell[]) => void
  /** The note/rest cells from the most recent note range-select (see
   * `onNoteRangeSelect`), echoed back so the highlight can be re-applied
   * declaratively — including after a re-render swaps in fresh SVG DOM
   * (e.g. the Monaco selection this range-select pushed triggering a
   * highlighted re-render), which would otherwise silently drop the
   * highlight. */
  selectedNoteCells?: NoteCell[]
  /** Per-note/rest `(source_part_index, note_id) → measure_index` mapping,
   * passed to wasm, which resolves every selection gesture by index rather
   * than pixel geometry. */
  noteSpans?: NoteSpan[]
  /** Fired on the commit click of a lyric-syllable click-and-click select
   * (see `getLyricAtPoint`), with every syllable cell the gesture resolved
   * to — but only when `onMeasureRangeSelect` isn't supplied. A lyric
   * selection can also cover notes (and vice versa for a note selection),
   * so whenever `onMeasureRangeSelect` is wired up it's used instead — with
   * both the note cells and the lyric cells the gesture resolved to, empty
   * array or not — since a mounted
   * editor's Monaco selection can only take one combined push per gesture
   * rather than one from each of this and `onNoteRangeSelect` independently
   * (see `usePreviewClickSelection`'s commit handling and
   * `onMeasureRangeSelect` below). */
  onLyricRangeSelect?: (selectedCells: LyricCell[]) => void
  /** The lyric syllable cells from the most recent lyric range-select (see
   * `onLyricRangeSelect`), echoed back so the highlight can be re-applied
   * declaratively, mirroring `selectedNoteCells`. */
  selectedLyricCells?: LyricCell[]
  /** Per-lyric-syllable `(source_part_index, note_id, verse) → measure_index`
   * mapping, passed to wasm alongside `noteSpans` so a selection also
   * covers the lyrics under it. */
  lyricSpans?: LyricSpan[]
  /** Fired instead of `onNoteRangeSelect`/`onLyricRangeSelect` for a
   * measure/bar-line click, and also for a note- or lyric-level click
   * whose range covers the other cell type too (see
   * `usePreviewClickSelection`), with every note cell and every lyric cell
   * the gesture resolved to. Any of the three gestures can select both cell
   * types at once, so this is a single combined callback rather than one
   * call to each of the other two — see `useAppController`'s
   * `handleMeasureRangeSelect` for why calling both independently doesn't
   * work. */
  onMeasureRangeSelect?: (
    noteCells: NoteCell[],
    lyricCells: LyricCell[],
  ) => void
  /** The measure range backing the current selection (caret or range),
   * regardless of whether it's caret-only — used to scroll the preview to
   * the selection even when `highlightedDocuments` isn't populated (e.g. a
   * section/sequence jump, which selects a real range and therefore opts
   * out of the amber caret-only highlight). See `selectedNoteCells` above
   * for the note/lyric-level analogue. `revealMeasureIndex` is which
   * measure to scroll to, when it isn't `start` — a section/sequence chain
   * selection can resolve to a written-measure range whose document-order
   * start isn't where the user actually navigated to. */
  selectedMeasureRange?: {
    start: number
    end: number
    revealMeasureIndex: number
    /** The exact disjoint measure ranges highlighted in the SVG preview for
     * this selection (a `sequence` chain), when it differs from the
     * single `[start, end]` span above. `Preview` only checks whether it's
     * set (see `findRevealTarget`); the highlight rects it drives come back
     * through `highlightedDocuments`, already rendered. */
    highlightRanges?: { start: number; end: number }[]
  } | null
  /** Fired whenever a click-and-click gesture starts or stops waiting on its
   * second click, so the pitch drawer can hold off until it commits. */
  onPendingSecondClickChange?: (pending: boolean) => void
}
