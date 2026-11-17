import { useCallback, useMemo } from 'react'
import type { PresetMenuProps } from '../components/PresetMenu'
import {
  type Notation,
  type PresetSummary,
  poentaroWasm,
} from '../poentaroWasm'
import type { PartInfo, ResolvedPartVisibility } from '../types'
import type { StaffNotation } from './useStaffNotation'

interface PresetMenuInputs {
  source: string
  readOnly: boolean
  wasmReady: boolean
  parts: PartInfo[]
  visibility: ResolvedPartVisibility
  staffNotation: StaffNotation
  setChosenPreset: (presetName: string | null) => void
  handlePartToggle: (abbreviation: string, enabled: boolean) => void
  handleSoloToggle: (abbreviation: string, soloed: boolean) => void
  handleSourceChange: (value: string) => void
}

/** The preset menu's props, and the part pill handlers: under a preset in a
 * file you can edit, a pill rewrites the preset's `visible_parts` and its
 * solo button the preset's `soloed_parts`; on Default or in a read-only
 * file they only change this browser's toggles. The notation menu works
 * the same way on the preset's `notation` line. */
export function usePresetMenu({
  source,
  readOnly,
  wasmReady,
  parts,
  visibility,
  staffNotation,
  setChosenPreset,
  handlePartToggle,
  handleSoloToggle,
  handleSourceChange,
}: PresetMenuInputs) {
  const presets = useMemo<PresetSummary[]>(
    () => (wasmReady ? poentaroWasm().listPresets(source) : []),
    [wasmReady, source],
  )
  const presetInForce = visibility.presetName ?? null

  const handlePartPillToggle = useCallback(
    (abbreviation: string, enabled: boolean) => {
      if (presetInForce == null || readOnly) {
        handlePartToggle(abbreviation, enabled)
        return
      }
      // A part hidden in this browser before the preset was chosen stays
      // off until shown again, so showing it also clears that.
      if (enabled) handlePartToggle(abbreviation, true)
      handleSourceChange(
        poentaroWasm().setPresetPartShown(
          source,
          presetInForce,
          parts.map((part) => part.abbreviation),
          { abbreviation, shown: enabled },
        ),
      )
    },
    [
      presetInForce,
      readOnly,
      source,
      parts,
      handlePartToggle,
      handleSourceChange,
    ],
  )

  const handleSoloPillToggle = useCallback(
    (abbreviation: string, soloed: boolean) => {
      if (presetInForce == null || readOnly) {
        handleSoloToggle(abbreviation, soloed)
        return
      }
      // A part soloed in this browser before the preset was chosen stays
      // soloed until unsoloed again, so unsoloing it also clears that.
      if (!soloed) handleSoloToggle(abbreviation, false)
      handleSourceChange(
        poentaroWasm().setPresetPartSoloed(
          source,
          presetInForce,
          parts.map((part) => part.abbreviation),
          { abbreviation, soloed },
        ),
      )
    },
    [
      presetInForce,
      readOnly,
      source,
      parts,
      handleSoloToggle,
      handleSourceChange,
    ],
  )

  const { setNotation } = staffNotation
  const handleNotationChange = useCallback(
    (notation: Notation) => {
      if (presetInForce == null || readOnly) {
        setNotation(notation)
        return
      }
      handleSourceChange(
        poentaroWasm().updatePresetField(source, presetInForce, {
          tag: 'notation',
          val: notation,
        }),
      )
    },
    [presetInForce, readOnly, source, setNotation, handleSourceChange],
  )

  const presetMenuProps: PresetMenuProps = {
    presets,
    presetInForce,
    onChoose: setChosenPreset,
    nameError: (name) => poentaroWasm().newPresetNameError(source, name),
    onCreatePreset: readOnly
      ? undefined
      : (name) => {
          const hidden = new Set(visibility.hiddenParts)
          handleSourceChange(
            poentaroWasm().addPresetSection(source, {
              name,
              visibleParts: parts
                .map((part) => part.abbreviation)
                .filter((abbreviation) => !hidden.has(abbreviation)),
              soloedParts: visibility.soloedParts,
              notation: staffNotation.notation,
            }),
          )
          setChosenPreset(name)
        },
  }

  return {
    handlePartPillToggle,
    handleSoloPillToggle,
    presetMenuProps,
    staffNotation: { ...staffNotation, setNotation: handleNotationChange },
  }
}
