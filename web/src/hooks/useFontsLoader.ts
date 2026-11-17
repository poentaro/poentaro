import fontsManifest from '../../../fonts/fonts.json'
import type { FontBytesByFamily } from '../poentaroWasm'
import type { AssetLoaderState, AssetStatus } from './useAssetLoader'
import { useAssetLoader } from './useAssetLoader'

export interface FontsLoaderState {
  fonts: FontBytesByFamily | null
  status: AssetStatus
  loadedBytes: number
  totalBytes: number
}

type FontRole = keyof FontBytesByFamily

function useFontRoleLoader(role: FontRole): AssetLoaderState {
  return useAssetLoader(`/fonts/${fontsManifest[role].filename}`)
}

/** Fetches every `font-family` role's font (filenames from
 * `fonts/fonts.json`, keyed by the same role names as the wasm component's
 * generated `FontBytesByFamily`). Which role measures or renders which text
 * is decided on the Rust side. */
export function useFontsLoader(): FontsLoaderState {
  const loaders: Record<FontRole, AssetLoaderState> = {
    serif: useFontRoleLoader('serif'),
    sansSerif: useFontRoleLoader('sansSerif'),
    monospace: useFontRoleLoader('monospace'),
  }
  const { serif, sansSerif, monospace } = loaders
  const all = Object.values(loaders)

  const status: AssetStatus = all.some((loader) => loader.status === 'error')
    ? 'error'
    : all.every((loader) => loader.status === 'ready')
      ? 'ready'
      : 'loading'

  const fonts: FontBytesByFamily | null =
    serif.bytes && sansSerif.bytes && monospace.bytes
      ? {
          serif: serif.bytes,
          sansSerif: sansSerif.bytes,
          monospace: monospace.bytes,
        }
      : null

  return {
    fonts,
    status,
    loadedBytes: all.reduce((sum, loader) => sum + loader.loadedBytes, 0),
    totalBytes: all.reduce((sum, loader) => sum + loader.totalBytes, 0),
  }
}
