import type { RefObject } from 'react'
import type { StaffNotation } from '../hooks/useStaffNotation'
import type {
  MetadataEdit,
  MetadataFields,
  NoteTiming,
  PartToggleState,
  SvgDocument,
} from '../poentaroWasm'
import type {
  Diagnostic,
  DiagnosticViewZone,
  EditorHandle,
  EditorSelection,
  LyricSpan,
  MeasureSpan,
  NoteSpan,
  PartDeclaration,
  PartFocusSelectionHandle,
  PartInfo,
  PartSettings,
  PitchDescription,
  RangeEditOperation,
} from '../types'
import type { LyricCell, NoteCell } from './Preview'

export interface MeasureRange {
  start: number
  end: number
  /** Which measure the preview should scroll to for this selection, when
   * it differs from `start` — see `Preview`'s matching prop doc comment. */
  revealMeasureIndex: number
  /** The exact disjoint measure ranges to highlight in the SVG preview —
   * see `Preview`'s matching prop doc comment. */
  highlightRanges?: { start: number; end: number }[]
}

export interface AppWorkspaceProps {
  editorCollapsed: boolean
  setEditorCollapsed: (updater: (collapsed: boolean) => boolean) => void
  /** True while viewing a `#share=` or `#synced=` read-only preview — hides
   * the `Editor` entirely and the pane-divider toggle, since there is
   * nothing to edit or expand back into. */
  hideEditor: boolean
  editorRef: RefObject<EditorHandle | null>
  /** Set by the part focus editor while it is open, so the preview selects
   * in it. */
  partFocusHandleRef: RefObject<PartFocusSelectionHandle | null>
  fileId: string
  source: string
  handleSourceChange: (value: string) => void
  /** "Format" toolbar action: drops redundant `score` lines and
   * normalizes whitespace. */
  handleFormatScore: () => void
  /** "Octave up"/"Octave down"/"Slur/Unslur"/"Tie/Untie" toolbar actions:
   * applies `operation` to every note whose span overlaps any of `ranges` —
   * the editor's current selection, which a multicursor selection (e.g. a
   * clicked part label) surfaces as a disjoint set rather than one span (see
   * `source_edit::shift_range_octave`/`source_edit::toggle_range_slur`/
   * `source_edit::toggle_range_tie`). */
  handleEditSelection: (
    ranges: EditorSelection[],
    operation: RangeEditOperation,
  ) => void
  /** Describes the one note or chord `ranges` covers in letter names, for
   * the pitch drawer (see `pitch_description::describe_selection`). */
  describeSelection: (
    ranges: EditorSelection[],
  ) => Promise<PitchDescription | null>
  readOnly: boolean
  diagnostics: Diagnostic[]
  diagnosticViewZones: DiagnosticViewZone[]
  measureSpans: MeasureSpan[]
  setSelectedLineRange: (
    range: { firstLine: number; lastLine: number } | null,
  ) => void
  notifySelection: (
    startLine: number,
    endLine: number,
    isEmpty: boolean,
    revealLine?: number,
    measureRanges?: { start: number; end: number }[],
    /** Skip the debounce and update the preview highlight right away. For
     * discrete clicks (section/sequence buttons) that have no rapid-fire
     * stream of events to coalesce. */
    immediate?: boolean,
  ) => void
  setEditPartsOpen: (open: boolean) => void
  setEditMetadataOpen: (open: boolean) => void
  forceSave: () => void
  measureAudioPlaying: boolean
  stopMeasurePlayback: () => void
  selectedMeasureRange: MeasureRange | null
  measureAudioGenerating: boolean
  soundfontReady: boolean
  playSelectedMeasures: () => void
  /** True while a note range-select (see `useNoteSelection`) is active; when
   * set, the editor's Cmd/Ctrl+Enter shortcut plays the selected notes
   * instead of the measure(s) under the cursor. */
  notePlaybackSelectionActive: boolean
  playNoteSelection: () => void
  editPartsOpen: boolean
  partDeclarations: PartDeclaration[]
  parts: PartInfo[]
  handlePartDeclarationChange: (
    abbreviation: string,
    settings: PartSettings,
  ) => void
  handleShiftPartOctave: (abbreviation: string, delta: number) => void
  previewInstrument: (programNumber: number) => void
  previewPercussion: (key: number) => void
  stopPreviewInstrument: () => void
  previewAudioPlaying: boolean
  editMetadataOpen: boolean
  parsedMetadata: MetadataFields | null
  metadataPresetName: string | null
  handleMetadataFieldChange: (edit: MetadataEdit) => void
  documents: SvgDocument[]
  highlightedDocuments: SvgDocument[]
  rendering: boolean
  handleSectionJump: (label: string) => void
  handleNoteRangeSelect: (selectedCells: NoteCell[]) => void
  /** Keeps the preview's note highlight in sync with the editor's own
   * current selection (see `useNoteSelection`'s
   * `handleEditorSelectionChange`), the reverse direction of
   * `handleNoteRangeSelect`. */
  handleEditorSelectionChange: (ranges: EditorSelection[]) => void
  selectedNoteCells: NoteCell[]
  /** The byte ranges behind a no-mounted-editor preview selection, read in
   * place of the editor's selection when `hideEditor` leaves no Monaco
   * selection to report one (see `useMeasureRangeSelection`'s
   * `measureRangeNoteByteRanges`). */
  selectedNoteByteRanges: EditorSelection[]
  /** Per-note/rest `(source_part_index, note_id) → measure_index` mapping,
   * passed to wasm, which resolves every selection gesture into the note
   * cells it covers without relying on pixel geometry. */
  noteSpans: NoteSpan[]
  /** Fired on the second click after a lyric-syllable range-select (see
   * `useLyricSelection`). Independent of `handleNoteRangeSelect` — a lyric
   * range-select never selects/highlights notes and vice versa. */
  handleLyricRangeSelect: (selectedCells: LyricCell[]) => void
  /** Keeps the preview's lyric highlight in sync with the editor's own
   * current selection, the reverse direction of `handleLyricRangeSelect`
   * (mirrors `handleEditorSelectionChange`, kept fully separate). */
  handleLyricEditorSelectionChange: (ranges: EditorSelection[]) => void
  selectedLyricCells: LyricCell[]
  /** Per-lyric-syllable `(source_part_index, note_id, verse) → measure_index`
   * mapping, passed to wasm alongside `noteSpans` to resolve every
   * selection gesture's lyric cells. */
  lyricSpans: LyricSpan[]
  /** Fired for a measure/bar-line click or range-select with both the note cells and
   * lyric cells it resolved — see `Preview.tsx`'s `onMeasureRangeSelect`. */
  handleMeasureRangeSelect: (
    noteCells: NoteCell[],
    lyricCells: LyricCell[],
  ) => void
  audioGenerating: boolean
  wavUrl: string | null
  wavFilename: string
  mp3Exporting: boolean
  mp3Url: string | null
  mp3Filename: string
  onRequestAudioDownload: (url: string, filename: string) => void
  noteTimings: NoteTiming[]
  measureAudioNoteTimings: NoteTiming[]
  measureAudioElement: HTMLAudioElement | null
  noPartsSelected: boolean
  staffNotation: StaffNotation
  /** Passed through to the part focus editor's ghost lines. */
  partToggles: PartToggleState
  /** The part focus editor's draft source, which the preview follows. */
  onPartFocusDraftChange: (draftSource: string | null) => void
}
