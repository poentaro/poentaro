import type { FileStoreState } from './fileStore'
import { NOT_LOADED } from './fileText'

function mergeAddedRemoved<T>(
  prevRecord: Record<string, T>,
  baseRecord: Record<string, T>,
  nextRecord: Record<string, T>,
  resolveAdded: (key: string) => T,
): Record<string, T> {
  const result = { ...prevRecord }
  for (const key of Object.keys(baseRecord)) {
    if (!(key in nextRecord)) delete result[key]
  }
  for (const key of Object.keys(nextRecord)) {
    if (!(key in baseRecord)) result[key] = resolveAdded(key)
  }
  return result
}

/**
 * Reconciles the result of an async structural operation (create/duplicate/
 * rename/delete/restore) with whatever the store has become since the
 * operation's `base` snapshot was read — instead of blindly replacing state
 * with `next`, which would discard any edits or other operations applied to
 * `prev` while the async call was in flight. Only the keys the operation
 * actually added or removed (relative to `base`) are applied to `prev`;
 * every other key is left exactly as `prev` has it. A file moved by the
 * operation (rename's `from` -> `to`, delete, restore) is recognised by its
 * file id, and the destination takes `prev`'s current text of the source key
 * when still present, so a concurrent edit to the file being
 * renamed/restored isn't lost either.
 */
export function mergeBackendResult(
  prev: FileStoreState,
  base: FileStoreState,
  next: FileStoreState,
): FileStoreState {
  const removedKeys = [
    ...Object.keys(base.userFiles).filter((key) => !(key in next.userFiles)),
    ...Object.keys(base.bin).filter((key) => !(key in next.bin)),
  ]
  const movedSource = (key: string): string | undefined => {
    const id = next.fileIds[key]
    return id === undefined
      ? undefined
      : removedKeys.find((removed) => base.fileIds[removed] === id)
  }
  const resolve = (record: 'userFiles' | 'bin') => (key: string) => {
    const source = movedSource(key)
    const carried =
      source === undefined
        ? undefined
        : (prev.userFiles[source] ?? prev.bin[source])
    return carried ?? next[record][key] ?? NOT_LOADED
  }

  const foldersChanged =
    JSON.stringify(next.folders ?? []) !== JSON.stringify(base.folders ?? [])
  const folders = foldersChanged ? next.folders : prev.folders
  return {
    ...(folders && folders.length > 0 ? { folders } : {}),
    active: next.active !== base.active ? next.active : prev.active,
    userFiles: mergeAddedRemoved(
      prev.userFiles,
      base.userFiles,
      next.userFiles,
      resolve('userFiles'),
    ),
    bin: mergeAddedRemoved(prev.bin, base.bin, next.bin, resolve('bin')),
    fileIds: mergeAddedRemoved(
      prev.fileIds,
      base.fileIds,
      next.fileIds,
      (key) => next.fileIds[key] ?? '',
    ),
  }
}
