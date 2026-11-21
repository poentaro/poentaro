import { useCallback, useEffect, useRef, useState } from 'react'
import { type FileStoreState, mergeLoadedTexts, selectFile } from '../fileStore'
import type { StorageBackend } from '../storage/types'
import type { FileOpError } from './useFileOperations'

type SetStore = (
  value: FileStoreState | ((prev: FileStoreState) => FileStoreState),
) => void

function openError(error: unknown): FileOpError {
  return {
    title: 'Could not open file',
    message: error instanceof Error ? error.message : String(error),
    stack: error instanceof Error ? error.stack : undefined,
  }
}

/**
 * Opens a file whose text may not be in memory yet (cloud files are listed
 * without their text). `openFile` reads the text first and only then makes
 * the file active, so a failed read leaves the previous file open and shows
 * the error. `opening` is true while that read is in flight.
 *
 * Any other way the active file ends up unloaded (e.g. deleting the open
 * file selects a neighbour) is covered by an effect that reads its text.
 */
export function useOpenFile(
  store: FileStoreState,
  setStore: SetStore,
  backend: StorageBackend,
  setFileOpError: (error: FileOpError | null) => void,
) {
  const [opening, setOpening] = useState(false)
  const storeRef = useRef(store)
  storeRef.current = store

  const openFile = useCallback(
    async (name: string): Promise<void> => {
      const base = storeRef.current
      setOpening(true)
      try {
        const next = await backend.ensureLoaded(base, name)
        setStore((prev) => selectFile(mergeLoadedTexts(prev, next), name))
      } catch (error) {
        setFileOpError(openError(error))
      } finally {
        setOpening(false)
      }
    },
    [backend, setStore, setFileOpError],
  )

  const active = store.active
  const activeUnloaded = store.userFiles[active]?.tag === 'not-loaded'
  useEffect(() => {
    if (!activeUnloaded) return
    let cancelled = false
    const base = storeRef.current
    backend.ensureLoaded(base, active).then(
      (next) => {
        if (!cancelled) setStore((prev) => mergeLoadedTexts(prev, next))
      },
      (error: unknown) => {
        if (!cancelled) setFileOpError(openError(error))
      },
    )
    return () => {
      cancelled = true
    }
  }, [activeUnloaded, active, backend, setStore, setFileOpError])

  /** Reads the text of every file under `folder`, for what needs them all
   * (the folder's static link). */
  const loadFolderText = useCallback(
    async (folder: string): Promise<void> => {
      const base = storeRef.current
      const names = Object.keys(base.userFiles).filter((name) =>
        name.startsWith(`${folder}/`),
      )
      try {
        const loaded = await names.reduce(
          async (state, name) => backend.ensureLoaded(await state, name),
          Promise.resolve(base),
        )
        setStore((prev) => mergeLoadedTexts(prev, loaded))
      } catch (error) {
        setFileOpError(openError(error))
      }
    },
    [backend, setStore, setFileOpError],
  )

  return { openFile, opening, loadFolderText }
}
