import { useCallback, useState } from 'react'
import { type MetadataEdit, poentaroWasm } from '../poentaroWasm'
import type { PartSettings } from '../types'
import { useMetadataFields } from './useMetadataFields'

/** Open/closed state for the edit-parts, edit-metadata, storage-settings,
 * bin, sync and history panels, plus the handlers that route their edits back to the
 * `.poentaro` source. */
export function useAppPanels(
  source: string,
  updatePartDeclaration: (
    abbreviation: string,
    settings: PartSettings,
  ) => Promise<string>,
  shiftPartOctave: (abbreviation: string, delta: number) => Promise<string>,
  handleSourceChange: (value: string) => void,
  /** The preset in force, whose section Edit Metadata edits. */
  presetName: string | null,
) {
  const [editPartsOpen, setEditPartsOpen] = useState(false)
  const [editMetadataOpen, setEditMetadataOpen] = useState(false)
  const [storageSettingsOpen, setStorageSettingsOpen] = useState(false)
  const [binOpen, setBinOpen] = useState(false)
  const [syncPanelOpen, setSyncPanelOpen] = useState(false)
  const [historyOpen, setHistoryOpen] = useState(false)

  const handlePartDeclarationChange = useCallback(
    (abbreviation: string, settings: PartSettings) => {
      void updatePartDeclaration(abbreviation, settings).then(
        handleSourceChange,
      )
    },
    [updatePartDeclaration, handleSourceChange],
  )

  const handleShiftPartOctave = useCallback(
    (abbreviation: string, delta: number) => {
      void shiftPartOctave(abbreviation, delta).then(handleSourceChange)
    },
    [shiftPartOctave, handleSourceChange],
  )

  const parsedMetadata = useMetadataFields(source, presetName, editMetadataOpen)

  // Only reachable from the Edit Metadata modal, which renders its fields
  // once `parsedMetadata` is non-null, i.e. once the wasm is ready.
  const handleMetadataFieldChange = useCallback(
    (edit: MetadataEdit) => {
      handleSourceChange(
        poentaroWasm().updateMetadataField(
          source,
          presetName ?? undefined,
          edit,
        ),
      )
    },
    [source, presetName, handleSourceChange],
  )

  return {
    editPartsOpen,
    setEditPartsOpen,
    editMetadataOpen,
    setEditMetadataOpen,
    storageSettingsOpen,
    setStorageSettingsOpen,
    binOpen,
    setBinOpen,
    syncPanelOpen,
    setSyncPanelOpen,
    historyOpen,
    setHistoryOpen,
    handlePartDeclarationChange,
    handleShiftPartOctave,
    parsedMetadata,
    metadataPresetName: presetName,
    handleMetadataFieldChange,
  }
}
