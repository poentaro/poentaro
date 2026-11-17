import { useCallback, useEffect, useRef, useState } from 'react'
import { poentaroWasm } from '../poentaroWasm'

export const CHOSEN_PRESET_KEY = 'poentaro:chosen-preset:v1'

/** File id → the preset chosen in the preset menu. Default has no entry. */
type ChosenPresetCache = Record<string, string>

function readCache(): ChosenPresetCache {
  try {
    const raw = localStorage.getItem(CHOSEN_PRESET_KEY)
    if (raw != null) {
      const parsed = JSON.parse(raw) as ChosenPresetCache
      if (parsed && typeof parsed === 'object') return parsed
    }
  } catch {
    // ignore corrupt storage
  }
  return {}
}

function writeCache(fileId: string, presetName: string | null): void {
  try {
    const cache = readCache()
    if (presetName == null) delete cache[fileId]
    else cache[fileId] = presetName
    localStorage.setItem(CHOSEN_PRESET_KEY, JSON.stringify(cache))
  } catch {
    // ignore quota errors
  }
}

/** The preset chosen for `fileId`, remembered per browser like the part
 * toggles. `null` is Default. Renaming the chosen preset in the editor
 * keeps it chosen. A name the file no longer has is kept; Rust resolves it
 * as Default (`resolvePartVisibility`'s `presetName`). */
export function useChosenPreset(
  fileId: string,
  source: string,
  wasmReady: boolean,
) {
  const [chosenPreset, setChosenPresetState] = useState<string | null>(
    () => readCache()[fileId] ?? null,
  )
  // The last source that still had the chosen preset. Kept while a rename
  // is half typed, so the rename is followed once the header parses again.
  const sourceWithPresetRef = useRef<string | null>(null)

  useEffect(() => {
    setChosenPresetState(readCache()[fileId] ?? null)
    sourceWithPresetRef.current = null
  }, [fileId])

  const setChosenPreset = useCallback(
    (presetName: string | null) => {
      setChosenPresetState(presetName)
      writeCache(fileId, presetName)
    },
    [fileId],
  )

  useEffect(() => {
    if (!wasmReady || chosenPreset == null) return
    const followed = poentaroWasm().followPresetRename(
      sourceWithPresetRef.current ?? source,
      source,
      chosenPreset,
    )
    if (followed == null) return
    sourceWithPresetRef.current = source
    if (followed !== chosenPreset) setChosenPreset(followed)
  }, [wasmReady, source, chosenPreset, setChosenPreset])

  return { chosenPreset, setChosenPreset }
}
