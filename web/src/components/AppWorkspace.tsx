import * as Tooltip from '@radix-ui/react-tooltip'
import { AlignLeft, ChevronDown, ChevronUp } from 'lucide-react'
import { useEffect, useState } from 'react'
import { MOBILE_BREAKPOINT_QUERY, useMediaQuery } from '../hooks/useMediaQuery'
import { useSelectionPitchDescription } from '../hooks/useSelectionPitchDescription'
import type { StreamablePart } from '../poentaroWasm'
import type { EditorSelection } from '../types'
import { staffClefFor } from '../worker/staffClef'
import type { AppWorkspaceProps } from './AppWorkspace.types'
import { EditMetadataModal } from './EditMetadataModal'
import { Editor } from './Editor'
import { EditorToolbarButton } from './EditorToolbarButton'
import { EditPartsModal } from './EditPartsModal'
import { NotationToggle } from './NotationToggle'
import { PageViewToggle } from './PageViewToggle'
import { PitchDrawer } from './PitchDrawer'
import { Preview } from './Preview'
import { PartFocusEditor } from './partFocus/PartFocusEditor'
import { PartFocusMenu } from './partFocus/PartFocusMenu'
import { SlurIcon } from './SlurIcon'
import { TieIcon } from './TieIcon'

export function AppWorkspace({
  editorCollapsed,
  setEditorCollapsed,
  hideEditor,
  editorRef,
  partFocusHandleRef,
  fileId,
  source,
  handleSourceChange,
  handleFormatScore,
  handleEditSelection,
  describeSelection,
  readOnly,
  diagnostics,
  diagnosticViewZones,
  measureSpans,
  setSelectedLineRange,
  notifySelection,
  setEditPartsOpen,
  setEditMetadataOpen,
  forceSave,
  measureAudioPlaying,
  stopMeasurePlayback,
  selectedMeasureRange,
  measureAudioGenerating,
  soundfontReady,
  playSelectedMeasures,
  notePlaybackSelectionActive,
  playNoteSelection,
  editPartsOpen,
  partDeclarations,
  parts,
  handlePartDeclarationChange,
  handleShiftPartOctave,
  previewInstrument,
  previewPercussion,
  stopPreviewInstrument,
  previewAudioPlaying,
  editMetadataOpen,
  parsedMetadata,
  metadataPresetName,
  handleMetadataFieldChange,
  documents,
  highlightedDocuments,
  rendering,
  handleSectionJump,
  handleNoteRangeSelect,
  handleEditorSelectionChange,
  selectedNoteCells,
  selectedNoteByteRanges,
  noteSpans,
  handleLyricRangeSelect,
  handleLyricEditorSelectionChange,
  selectedLyricCells,
  lyricSpans,
  handleMeasureRangeSelect,
  audioGenerating,
  wavUrl,
  wavFilename,
  mp3Exporting,
  mp3Url,
  mp3Filename,
  onRequestAudioDownload,
  noteTimings,
  measureAudioNoteTimings,
  measureAudioElement,
  noPartsSelected,
  staffNotation,
  previewLayout,
  partToggles,
  onPartFocusDraftChange,
}: AppWorkspaceProps) {
  const staffView = staffClefFor(staffNotation.notation) !== undefined
  const [editorPaneEl, setEditorPaneEl] = useState<HTMLDivElement | null>(null)
  // The editor's current selection, in byte offsets — `null` for a plain
  // caret (nothing to shift), tracked purely to enable/disable the "Octave
  // up"/"Octave down" toolbar buttons and supply their byte ranges on click.
  // A disjoint list, not one min/max range: a Monaco multicursor selection
  // (e.g. clicking a part label, which selects that part's notes across
  // every measure in its system) is generally disjoint, so collapsing it to
  // a single span would sweep in unrelated notes/parts sitting between the
  // disjoint pieces.
  const [selectionByteRanges, setSelectionByteRanges] = useState<
    EditorSelection[] | null
  >(null)
  const [previewSelectionPending, setPreviewSelectionPending] = useState(false)
  const [focusedPart, setFocusedPart] = useState<{
    part: StreamablePart
    stream: string
  } | null>(null)
  const describedSelection = useSelectionPitchDescription(
    // Synced/shared views never mount the editor, so its selection never
    // reports in; fall back to the preview's own note selection there.
    hideEditor
      ? selectedNoteByteRanges.length === 0
        ? null
        : selectedNoteByteRanges
      : selectionByteRanges,
    previewSelectionPending,
    describeSelection,
  )
  const isMobile = useMediaQuery(MOBILE_BREAKPOINT_QUERY)
  // Shared by the main editor and the part focus editor, which reports in
  // draft source terms.
  const handleSelectionLinesChange = (
    firstLine: number,
    lastLine: number,
    isEmpty: boolean,
  ) => {
    setSelectedLineRange(null)
    notifySelection(firstLine, lastLine, isEmpty)
  }
  const handleSelectionOffsetChange = (
    ranges: EditorSelection[],
    isEmpty: boolean,
  ) => {
    // Every disjoint range is passed through, not just the primary one — a
    // multicursor selection (e.g. a clicked part label's per-measure ranges,
    // or the "shift selection octave" toolbar action's remapped ranges) must
    // keep every range's notes/lyrics highlighted, not just whichever one
    // Monaco calls the anchor.
    handleEditorSelectionChange(ranges)
    handleLyricEditorSelectionChange(ranges)
    setSelectionByteRanges(isEmpty ? null : ranges)
  }
  // Below the mobile breakpoint only one pane is visible at a time, so
  // showing the editor must collapse the preview instead of sitting beside it.
  const previewCollapsed = isMobile && !hideEditor && !editorCollapsed
  // On mobile the divider sits between a stacked editor (above) and preview
  // (below), so the chevron points up/down instead of left/right.
  const toggleIconRotationDeg = isMobile
    ? editorCollapsed
      ? -90
      : 90
    : editorCollapsed
      ? 180
      : 0

  // Default to showing the preview on mobile, where only one pane fits.
  useEffect(() => {
    if (window.matchMedia(MOBILE_BREAKPOINT_QUERY).matches) {
      setEditorCollapsed(() => true)
    }
  }, [setEditorCollapsed])

  return (
    <main className="workspace">
      <section
        className={[
          'pane',
          'pane--editor',
          'pane--collapsible',
          editorCollapsed ? 'pane--editor-collapsed' : '',
        ]
          .filter(Boolean)
          .join(' ')}
      >
        <div className="editor-layout">
          <div className="editor-main" ref={setEditorPaneEl}>
            {hideEditor ? null : (
              // Stays mounted under the part focus editor, so its model and
              // undo history survive; `inert` keeps focus out of it.
              <div className="editor-main-source" inert={focusedPart !== null}>
                <Editor
                  ref={editorRef}
                  path={fileId}
                  value={source}
                  onChange={handleSourceChange}
                  toolbar={
                    <Tooltip.Provider delayDuration={0}>
                      <EditorToolbarButton
                        label="Format score"
                        icon={<AlignLeft size={14} aria-hidden="true" />}
                        onClick={handleFormatScore}
                      />
                      <PartFocusMenu
                        source={source}
                        onChoose={(part, stream) =>
                          setFocusedPart({ part, stream })
                        }
                      />
                      <EditorToolbarButton
                        label="Octave up"
                        icon={<ChevronUp size={14} aria-hidden="true" />}
                        disabled={selectionByteRanges === null}
                        onClick={() => {
                          if (selectionByteRanges === null) return
                          handleEditSelection(selectionByteRanges, {
                            kind: 'shiftOctave',
                            delta: 1,
                          })
                        }}
                      />
                      <EditorToolbarButton
                        label="Octave down"
                        icon={<ChevronDown size={14} aria-hidden="true" />}
                        disabled={selectionByteRanges === null}
                        onClick={() => {
                          if (selectionByteRanges === null) return
                          handleEditSelection(selectionByteRanges, {
                            kind: 'shiftOctave',
                            delta: -1,
                          })
                        }}
                      />
                      <EditorToolbarButton
                        label="Slur/Unslur"
                        icon={<SlurIcon size={14} />}
                        disabled={selectionByteRanges === null}
                        onClick={() => {
                          if (selectionByteRanges === null) return
                          handleEditSelection(selectionByteRanges, {
                            kind: 'toggleSlur',
                          })
                        }}
                      />
                      <EditorToolbarButton
                        label="Tie/Untie"
                        icon={<TieIcon size={14} />}
                        disabled={selectionByteRanges === null}
                        onClick={() => {
                          if (selectionByteRanges === null) return
                          handleEditSelection(selectionByteRanges, {
                            kind: 'toggleTie',
                          })
                        }}
                      />
                    </Tooltip.Provider>
                  }
                  readOnly={readOnly}
                  diagnostics={diagnostics}
                  diagnosticViewZones={diagnosticViewZones}
                  measureSpans={measureSpans}
                  onSelectionChange={handleSelectionLinesChange}
                  onSelectionOffsetChange={handleSelectionOffsetChange}
                  onEditPartsClick={() => setEditPartsOpen(true)}
                  onEditMetadataClick={() => setEditMetadataOpen(true)}
                  onForceSave={forceSave}
                  onPlayMeasure={
                    measureAudioPlaying
                      ? stopMeasurePlayback
                      : !measureAudioGenerating && soundfontReady
                        ? notePlaybackSelectionActive
                          ? playNoteSelection
                          : selectedMeasureRange !== null
                            ? playSelectedMeasures
                            : undefined
                        : undefined
                  }
                />
              </div>
            )}
            <EditPartsModal
              open={editPartsOpen}
              onOpenChange={setEditPartsOpen}
              partDeclarations={partDeclarations}
              allParts={parts}
              onPartDeclarationChange={handlePartDeclarationChange}
              onShiftPartOctave={handleShiftPartOctave}
              previewInstrument={previewInstrument}
              previewPercussion={previewPercussion}
              stopPreviewInstrument={stopPreviewInstrument}
              previewAudioPlaying={previewAudioPlaying}
            />
            {focusedPart !== null && !hideEditor && (
              <PartFocusEditor
                fileId={fileId}
                source={source}
                part={focusedPart.part}
                initialStream={focusedPart.stream}
                partToggles={partToggles}
                onDraftChange={onPartFocusDraftChange}
                onApply={(newSource) => {
                  editorRef.current?.replaceContent(newSource)
                  setFocusedPart(null)
                }}
                onCancel={() => setFocusedPart(null)}
                handleRef={partFocusHandleRef}
                onSelectionChange={handleSelectionLinesChange}
                onSelectionOffsetChange={handleSelectionOffsetChange}
              />
            )}
            <EditMetadataModal
              open={editMetadataOpen}
              onOpenChange={setEditMetadataOpen}
              metadata={parsedMetadata}
              presetName={metadataPresetName}
              onFieldChange={handleMetadataFieldChange}
              container={editorPaneEl}
            />
          </div>
        </div>
      </section>
      <div className="pane-divider">
        {hideEditor ? null : (
          <div className="pane-divider-toggles">
            <button
              type="button"
              className="pane-divider-toggle"
              onClick={() => setEditorCollapsed((collapsed) => !collapsed)}
              title={editorCollapsed ? 'Show editor' : 'Hide editor'}
              aria-label={editorCollapsed ? 'Show editor' : 'Hide editor'}
            >
              <span
                className="pane-divider-toggle-icon"
                style={{
                  transform: `rotate(${toggleIconRotationDeg}deg)`,
                }}
                aria-hidden="true"
              >
                ‹
              </span>
            </button>
          </div>
        )}
      </div>
      <section
        className={[
          'pane',
          'pane--preview',
          'pane--collapsible',
          previewCollapsed ? 'pane--preview-collapsed' : '',
        ]
          .filter(Boolean)
          .join(' ')}
      >
        <Preview
          documents={documents}
          floatingControls={
            <>
              <PageViewToggle
                view={previewLayout.view}
                onChange={previewLayout.setView}
              />
              <NotationToggle
                notation={staffNotation.notation}
                onChange={staffNotation.setNotation}
              />
            </>
          }
          maxView={previewLayout.view === 'max'}
          onPagesWidthChange={previewLayout.setContainerWidth}
          staffScore={staffView ? staffNotation.score : null}
          highlightedDocuments={highlightedDocuments}
          rendering={staffView ? staffNotation.loading : rendering}
          onSectionLabelClick={handleSectionJump}
          onNoteRangeSelect={handleNoteRangeSelect}
          selectedNoteCells={selectedNoteCells}
          noteSpans={noteSpans}
          onLyricRangeSelect={handleLyricRangeSelect}
          selectedLyricCells={selectedLyricCells}
          lyricSpans={lyricSpans}
          onMeasureRangeSelect={handleMeasureRangeSelect}
          selectedMeasureRange={selectedMeasureRange}
          audioGenerating={audioGenerating}
          wavUrl={wavUrl}
          wavFilename={wavFilename}
          mp3Exporting={mp3Exporting}
          mp3Url={mp3Url}
          mp3Filename={mp3Filename}
          onRequestAudioDownload={onRequestAudioDownload}
          noteTimings={noteTimings}
          measureAudioNoteTimings={measureAudioNoteTimings}
          measureAudioElement={measureAudioElement}
          emptyMessage={
            noPartsSelected
              ? 'No parts selected.'
              : ((staffView ? staffNotation.error : null) ?? 'No preview yet.')
          }
          onPendingSecondClickChange={setPreviewSelectionPending}
        />
        <PitchDrawer described={describedSelection} />
      </section>
    </main>
  )
}
