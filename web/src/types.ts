export type {
  Diagnostic,
  DiagnosticMessage,
  DiagnosticViewZone,
  GeneratePdfResponse as GeneratePdfResult,
  GenerateSplitPdfsResponse as GenerateSplitPdfResult,
  GenerateWavResponse as GenerateWavResult,
  ListLyricSpansResponse as ListLyricSpansResult,
  ListMeasureSpansResponse as ListMeasureSpansResult,
  ListNoteSpansResponse as ListNoteSpansResult,
  ListPartsResponse as ListPartsResult,
  LyricSpan,
  MeasureAtOffsetResponse as MeasureAtOffsetResult,
  MeasureSpan,
  NoteSpan,
  Part as PartInfo,
  PartDeclaration,
  PartMode,
  PartSettings,
  PartToggleState,
  PitchDescription,
  RenderResponse as RenderResult,
  ResolvedPartVisibility,
  SectionRange,
  SequenceEntry,
  Span as ByteSpan,
} from './poentaroWasm'

export interface EditorSelection {
  start: number
  end: number
}

/** A selection-scoped editor toolbar action applied to the current
 * (possibly multicursor) selection: "Octave up"/"Octave down"
 * (`source_edit::shift_range_octave`), "Slur/Unslur"
 * (`source_edit::toggle_range_slur`) or "Tie/Untie"
 * (`source_edit::toggle_range_tie`). */
export type RangeEditOperation =
  | { kind: 'shiftOctave'; delta: number }
  | { kind: 'toggleSlur' }
  | { kind: 'toggleTie' }

export interface EditorHandle {
  /** Insert text at the current cursor, replacing any selection. */
  insertAtCursor: (text: string) => void
  getSelection: () => EditorSelection
  setSelection: (start: number, end: number) => void
  /**
   * Select a disjoint set of ranges (Monaco multicursor) by UTF-8 byte
   * offset, matching `setSelection`'s convention. Reveals the first range.
   */
  setSelections: (ranges: Array<{ start: number; end: number }>) => void
  /**
   * Replace the entire model content with `newSource` and select `ranges`
   * (byte offsets into `newSource`) — both synchronously, in the same call,
   * so no other effect can observe the new text with a stale selection in
   * between. Used by the "shift selection octave" toolbar action: unlike
   * `onChange` + a later `setSelections` call, this closes the race where
   * `Editor.tsx`'s own generic post-edit selection restore (see its
   * `savedSelectionsRef` effect pair) would otherwise re-apply the
   * *pre-edit* selection's stale line/column positions over top of the
   * correct one (see `HANDOFF-octave-toolbar-part-label-selection-bug.md`).
   */
  replaceContentWithSelections: (
    newSource: string,
    ranges: Array<{ start: number; end: number }>,
  ) => void
  /** Replace the entire model content with `newSource` as one undo step
   * (an edit operation on the model, not a `value` swap), so Ctrl+Z in the
   * editor restores the previous source. Used by the part stream editor's
   * Apply. */
  replaceContent: (newSource: string) => void
  /** Select a range of lines by 1-indexed line numbers and reveal the start. */
  setSelectionByLines: (startLine: number, endLine: number) => void
  /**
   * Select a disjoint set of whole-line ranges (Monaco multicursor) by
   * 1-indexed line numbers — the line-range analogue of `setSelections`,
   * for callers (e.g. a `sequence` chain selection spanning out-of-order
   * sections) that resolve to lines rather than byte offsets. Reveals
   * `revealStartLine` (defaulting to the first range's start) — two
   * disjoint ranges can sit too far apart in the source to both fit on
   * screen at once, so callers should pass whichever one the user is
   * currently pointing at.
   */
  setSelectionsByLines: (
    ranges: Array<{ startLine: number; endLine: number }>,
    revealStartLine?: number,
  ) => void
  /** Move the cursor to the given JS string char offset and reveal the line. */
  jumpToOffset: (charOffset: number) => void
  focus: () => void
  getEditor: () => import('monaco-editor').editor.IStandaloneCodeEditor | null
}

/** The calls the preview makes to select in the editor, as the part focus
 * editor answers them: it takes draft source offsets and lines, and selects
 * the matching text in its part stream. */
export type PartFocusSelectionHandle = Pick<
  EditorHandle,
  'setSelections' | 'setSelectionByLines' | 'setSelectionsByLines' | 'focus'
>
