import { useCallback, useState } from 'react'
import { type FileStoreState, mergeBackendResult } from '../fileStore'
import { moveDestination } from '../fileStoreQueries'
import type { StorageBackend } from '../storage/types'
import type { FileOpError } from './useFileOperations'

/** Recovers a lost `.poentaro` source from a previously exported `.svg`/`.pdf`
 * file and imports it as a new user file. Kept separate from
 * `useFileOperations` since it needs the worker's `importFromFile`
 * extraction step before it has a source to hand to the backend. */
export function useFileImport(
  store: FileStoreState,
  backend: StorageBackend,
  setStore: (
    value: FileStoreState | ((prev: FileStoreState) => FileStoreState),
  ) => void,
  setFileOpError: (error: FileOpError | null) => void,
  importFromFile: (file: File) => Promise<string>,
) {
  const [importingFile, setImportingFile] = useState(false)

  const handleImportFile = useCallback(
    async (file: File, folder?: string) => {
      const base = store
      setImportingFile(true)
      try {
        const importedSource = await importFromFile(file)
        const derivedName = file.name.replace(/\.(svg|pdf)$/i, '')
        // Rust puts the name inside the folder (the top level when omitted).
        const name = moveDestination(derivedName, folder) ?? derivedName
        const next = await backend.importFile(base, name, importedSource)
        setStore((prev) => mergeBackendResult(prev, base, next))
      } catch (error) {
        setFileOpError({
          title: 'Could not import file',
          message: error instanceof Error ? error.message : String(error),
          stack: error instanceof Error ? error.stack : undefined,
        })
      } finally {
        setImportingFile(false)
      }
    },
    [store, backend, setStore, setFileOpError, importFromFile],
  )

  return { importingFile, handleImportFile }
}
