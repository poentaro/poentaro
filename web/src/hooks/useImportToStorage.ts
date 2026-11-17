import { useCallback } from 'react'
import { type FileStoreState, mergeBackendResult } from '../fileStore'
import type { FolderSnapshot } from '../poentaroWasm'
import type { StorageBackend } from '../storage/types'

interface FileOpError {
  title: string
  message: string
  stack?: string
}

/**
 * Imports a filename/content pair into the active file store via the
 * backend, surfacing failures through `setFileOpError`. Shared by
 * `useSharedPreview` (`#share=` links) and the synced-share-view import
 * action (`#synced=` links), which both hand it the same `{filename,
 * content}` shape.
 */
export function useImportToStorage(
  store: FileStoreState,
  backend: StorageBackend,
  setStore: (
    value: FileStoreState | ((prev: FileStoreState) => FileStoreState),
  ) => void,
  setFileOpError: (error: FileOpError | null) => void,
) {
  const importFolder = useCallback(
    async (snapshot: FolderSnapshot) => {
      const base = store
      try {
        const next = await backend.importFolder(base, snapshot)
        setStore((prev) => mergeBackendResult(prev, base, next))
        return next
      } catch (error) {
        setFileOpError({
          title: 'Could not import shared folder',
          message: error instanceof Error ? error.message : String(error),
          stack: error instanceof Error ? error.stack : undefined,
        })
        throw error
      }
    },
    [store, backend, setStore, setFileOpError],
  )
  const importFile = useCallback(
    async (filename: string, content: string) => {
      const base = store
      try {
        const next = await backend.importFile(base, filename, content)
        setStore((prev) => mergeBackendResult(prev, base, next))
      } catch (error) {
        setFileOpError({
          title: 'Could not import shared score',
          message: error instanceof Error ? error.message : String(error),
          stack: error instanceof Error ? error.stack : undefined,
        })
        throw error
      }
    },
    [store, backend, setStore, setFileOpError],
  )
  return { importFile, importFolder }
}
