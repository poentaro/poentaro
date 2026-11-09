import type {
  FontBytesByFamily,
  NoteTiming,
  StaffClef,
  StaffMeasure,
  SvgDocument,
  Notation as WasmNotation,
} from '../poentaroWasm'
import type {
  Diagnostic,
  DiagnosticViewZone,
  EditorSelection,
  LyricSpan,
  MeasureSpan,
  NoteSpan,
  PartDeclaration,
  PartInfo,
  PartSettings,
  PitchDescription,
  RangeEditOperation,
  ResolvedPartVisibility,
  SectionRange,
  SequenceEntry,
} from '../types'
import type { NoteCell } from '../utils/noteSpanSelection'

/** Which notation the preview (and so PDF export) shows. `violin` and `cello`
 * are western staff notation, every staff in treble or bass clef. `abc` is the numbered
 * layout with letter names (`D E F♯`) in place of degree numbers; `guitar` is
 * the numbered layout with each degree followed by its letter name (`1 (D)`)
 * and a guitar chord diagram at each chord symbol; and
 * `harmonica` the numbered layout with chromatic harmonica tabs (`1 (1) 2`). */
export type Notation = WasmNotation

/** Verovio's pages for MEI Rust exported (see `staffMessageHandlers.ts`). */
export type StaffEngraveResult =
  | { status: 'ok'; pages: string[] }
  | { status: 'err'; message: string }

/** The staff-notation view: Verovio's pages, plus Rust's list of which SVG
 * element engraves which score measure. */
export interface StaffScore {
  pages: string[]
  measures: StaffMeasure[]
}

export type StaffRenderResult =
  | { status: 'ok'; score: StaffScore }
  | { status: 'err'; message: string }

export type WorkerRequest =
  | { type: 'wasmModule'; module: WebAssembly.Module }
  | {
      type: 'loadSoundfont'
      soundfont: ArrayBuffer
    }
  | {
      type: 'loadPdfFonts'
      fonts: FontBytesByFamily
    }
  | {
      type: 'render'
      source: string
      id: number
      visibility: ResolvedPartVisibility
      notation: Notation
    }
  | { type: 'listParts'; source: string; id: number }
  | {
      type: 'updatePartDeclaration'
      source: string
      abbreviation: string
      settings: PartSettings
      id: number
    }
  | {
      type: 'generatePdf'
      source: string
      id: number
      visibility: ResolvedPartVisibility
      /** The preview's notation; the PDF is engraved in the same one. */
      notation: Notation
    }
  | {
      type: 'generateSplitPdf'
      source: string
      id: number
      baseName: string
      /** The preset in force: only its parts are split, and the files are
       * named after it. */
      presetName?: string
      notation: Notation
    }
  | {
      type: 'generateMidi'
      source: string
      id: number
      enabledTracks?: string[]
    }
  | {
      type: 'generateSplitMidi'
      source: string
      id: number
      baseName: string
      presetName?: string
    }
  | {
      type: 'generateSplitWav'
      source: string
      id: number
      baseName: string
      presetName?: string
    }
  | {
      type: 'generateMp3'
      source: string
      id: number
      enabledTracks?: string[]
    }
  | {
      type: 'generateSplitMp3'
      source: string
      id: number
      baseName: string
      presetName?: string
    }
  | {
      type: 'generateAudio'
      source: string
      id: number
      enabledTracks?: string[]
    }
  | {
      type: 'generateMeasureRangeAudio'
      source: string
      id: number
      startMeasureIndex: number
      endMeasureIndex: number
      extendToLastOccurrence: boolean
      respectSequence: boolean
      /**
       * 0-based index range into `sequence` (the order entries are
       * written in `sequence`) naming the exact entry/entries selected,
       * so a repeated label (e.g. `A, B(-x), B`) resolves to the clicked
       * occurrence instead of always the first one sharing that written
       * measure range. Omit both when the range isn't a `sequence`
       * selection (e.g. "play current measure").
       */
      sequenceEntryStartIndex?: number
      sequenceEntryEndIndex?: number
      enabledTracks?: string[]
      /** Whether a click sounds on every beat of the generated clip. */
      metronome: boolean
      /**
       * The part-visibility toggle's current state — unlike `enabledTracks`
       * (which "play selection" overrides down to just the range-selected
       * parts, muting this clip's audio), this is always the same set the
       * currently-rendered SVG was compacted against, never a playback-only
       * override. Passed straight through to WASM's own `visible_tracks`
       * parameter (see `list_note_timings_for_range`), so each returned
       * `NoteTiming`'s `source_part_index` already agrees with the SVG's
       * `data-part-index` — including block/note-id structure such as a
       * `MultiMeasureRest` run only created once a hidden sibling part's
       * notes are removed (see **Note identity** in `ARCHITECTURE.md`).
       * `undefined` means no part is hidden.
       */
      visibleTracks?: string[]
      /**
       * When present, narrows the generated clip down to exactly these
       * range-selected `(sourcePartIndex, noteId)` cells' elapsed-seconds
       * span (sample-accurately trimmed/faded in Rust — see
       * `poentaro::wav::TrimWindow`) instead of playing the whole
       * `[startMeasureIndex, endMeasureIndex]` range — what the web app's
       * "play selection" needs. Omit for every other measure-range playback
       * (e.g. "play current measure"/"play from current measure"/"play
       * all"), which always plays the range in full.
       */
      trimToSelectedNoteCells?: NoteCell[]
    }
  | {
      type: 'renderWithHighlightRange'
      source: string
      id: number
      /** Disjoint, inclusive measure-index ranges to highlight — a `#
       * sequence` chain selection spanning out-of-order entries can
       * highlight several disjoint measures at once (e.g. "C" and a later
       * repeat of "A", but not "B" in between). */
      ranges: { start: number; end: number }[]
      visibility: ResolvedPartVisibility
      notation: Notation
    }
  | { type: 'listMeasureSpans'; source: string; id: number }
  | {
      type: 'listNoteSpans'
      source: string
      id: number
      enabledTracks?: string[]
    }
  | {
      type: 'listLyricSpans'
      source: string
      id: number
      enabledTracks?: string[]
    }
  | { type: 'previewInstrument'; id: number; programNumber: number }
  | { type: 'previewPercussion'; id: number; key: number }
  | {
      type: 'importFromFile'
      id: number
      bytes: ArrayBuffer
      kind: 'svg' | 'pdf'
    }
  | {
      type: 'formatScore'
      source: string
      id: number
    }
  | {
      type: 'shiftPartOctave'
      source: string
      abbreviation: string
      delta: number
      id: number
    }
  | {
      type: 'editRange'
      source: string
      // A disjoint set of ranges, not one min/max span — a multicursor
      // selection (e.g. clicking a part label, which selects that part's
      // notes across every measure in its system) is generally disjoint;
      // collapsing it to one span would sweep in unrelated notes/parts
      // sitting between the disjoint pieces (see
      // `source_edit::shift_range_octave`'s doc comment).
      ranges: EditorSelection[]
      operation: RangeEditOperation
      id: number
    }
  | {
      type: 'describeSelection'
      source: string
      ranges: EditorSelection[]
      id: number
    }
  | {
      type: 'renderStaff'
      source: string
      id: number
      visibility: ResolvedPartVisibility
      clef: StaffClef
    }

export type WorkerResponse =
  | {
      type: 'ready'
      audioAvailable: boolean
      pdfAvailable: boolean
      midiAvailable: boolean
      mp3Available: boolean
    }
  | {
      type: 'ok'
      id: number
      documents: SvgDocument[]
      diagnostics: Diagnostic[]
      diagnosticViewZones: DiagnosticViewZone[]
    }
  | {
      type: 'audio'
      id: number
      wav: ArrayBuffer
      noteTimings: NoteTiming[]
    }
  | { type: 'audioErr'; id: number }
  | {
      type: 'err'
      id: number
      diagnostics: Diagnostic[]
      diagnosticViewZones: DiagnosticViewZone[]
    }
  | {
      type: 'parts'
      id: number
      parts: PartInfo[]
      declarations: PartDeclaration[]
    }
  | {
      type: 'partDeclarationUpdated'
      id: number
      source: string
      declarations: PartDeclaration[]
    }
  | { type: 'pdf'; id: number; pdf: ArrayBuffer }
  | { type: 'pdfErr'; id: number; diagnostics: Diagnostic[] }
  | { type: 'splitPdf'; id: number; zip: ArrayBuffer }
  | { type: 'splitPdfErr'; id: number; diagnostics: Diagnostic[] }
  | { type: 'midi'; id: number; midi: ArrayBuffer }
  | { type: 'midiErr'; id: number; diagnostics: Diagnostic[] }
  | { type: 'splitMidi'; id: number; zip: ArrayBuffer }
  | { type: 'splitMidiErr'; id: number; diagnostics: Diagnostic[] }
  | { type: 'splitWav'; id: number; zip: ArrayBuffer }
  | { type: 'splitWavErr'; id: number; diagnostics: Diagnostic[] }
  | { type: 'mp3'; id: number; mp3: ArrayBuffer; noteTimings: NoteTiming[] }
  | { type: 'mp3Err'; id: number; diagnostics: Diagnostic[] }
  | { type: 'splitMp3'; id: number; zip: ArrayBuffer }
  | { type: 'splitMp3Err'; id: number; diagnostics: Diagnostic[] }
  | {
      type: 'measureRangeAudio'
      id: number
      wav: ArrayBuffer
      noteTimings: NoteTiming[]
    }
  | { type: 'measureRangeAudioErr'; id: number }
  | { type: 'instrumentPreview'; id: number; wav: ArrayBuffer }
  | { type: 'instrumentPreviewErr'; id: number }
  | { type: 'percussionPreview'; id: number; wav: ArrayBuffer }
  | { type: 'percussionPreviewErr'; id: number }
  | { type: 'highlightRangeOk'; id: number; documents: SvgDocument[] }
  | { type: 'highlightRangeErr'; id: number; diagnostics: Diagnostic[] }
  | {
      type: 'measureSpans'
      id: number
      status: 'ok' | 'err'
      spans: MeasureSpan[]
      sectionRanges: SectionRange[]
      sequenceEntries: SequenceEntry[]
    }
  | {
      type: 'noteSpans'
      id: number
      status: 'ok' | 'err'
      spans: NoteSpan[]
    }
  | {
      type: 'lyricSpans'
      id: number
      status: 'ok' | 'err'
      spans: LyricSpan[]
    }
  | { type: 'importOk'; id: number; source: string }
  | { type: 'importErr'; id: number }
  | { type: 'scoreFormatted'; id: number; source: string }
  | { type: 'partOctaveShifted'; id: number; source: string }
  | {
      type: 'rangeEdited'
      id: number
      source: string
      /** The request's own ranges remapped onto the *new* `source`,
       * synchronously computed by `source_edit::RangeEditResult` — lets
       * the caller restore the editor selection in the same tick it applies
       * the new source, closing a race where an async re-derivation could
       * lose to a second click landing first (see
       * `HANDOFF-octave-toolbar-part-label-selection-bug.md`). */
      ranges: { start: number; end: number }[]
    }
  | {
      type: 'selectionDescribed'
      id: number
      description: PitchDescription | null
    }
  | { type: 'staffRendered'; id: number; result: StaffRenderResult }
