import { useCallback, useEffect, useState } from 'react'
import { activeFileLabel } from '../fileStoreQueries'
import { useWasmReady } from './useWasmReady'

const STORAGE_KEY = 'poentaro:file-tree-expanded:v1'

function load(): string[] {
  try {
    const parsed: unknown = JSON.parse(
      localStorage.getItem(STORAGE_KEY) ?? '[]',
    )
    return Array.isArray(parsed)
      ? parsed.filter((key): key is string => typeof key === 'string')
      : []
  } catch {
    return []
  }
}

function save(keys: string[]) {
  try {
    localStorage.setItem(STORAGE_KEY, JSON.stringify(keys))
  } catch {
    // Storage may be blocked; the expanded state just won't persist.
  }
}

/** Folder expansion in the file list, remembered in this browser only. A per-viewer UI preference: deliberately not part of the
 * `FileStoreState` that is persisted to the storage backend and synced to the
 * cloud. Rust decides what is actually visible from these raw keys. */
export function useExpandedFolders(activeFile: string) {
  const [expanded, setExpanded] = useState<string[]>(load)
  const toggle = useCallback((key: string) => {
    setExpanded((previous) => {
      const next = previous.includes(key)
        ? previous.filter((existing) => existing !== key)
        : [...previous, key]
      save(next)
      return next
    })
  }, [])
  /** Opens the given folders (no-op for those already open). */
  const expand = useCallback((keys: string[]) => {
    setExpanded((previous) => {
      const added = keys.filter((key) => !previous.includes(key))
      if (added.length === 0) return previous
      const next = [...previous, ...added]
      save(next)
      return next
    })
  }, [])
  const ready = useWasmReady()
  useEffect(() => {
    // Reveal the active file once per selection so later toggles can hide it.
    if (ready) expand(activeFileLabel(activeFile).ancestorKeys)
  }, [activeFile, expand, ready])

  return { expanded, toggle, expand }
}
