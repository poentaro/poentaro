import type { FileStoreState } from '../fileStore'
import type { CloudBackend, CloudBackendError } from '../storage/cloudBackend'
import type { StorageBackend } from '../storage/types'

export type ConflictResolution = 'overwrite-mine' | 'discard-mine'

/**
 * Resolves a `409` conflict on the active file with a minimal "last write
 * wins" choice (no 3-way merge, per the v1 limitations): `overwrite-mine`
 * realigns the tracked revision to the server's reported current one and
 * re-pushes the current in-memory content (`CloudBackend.forceOverwrite`);
 * `discard-mine` reloads the backend's file listing and replaces the active
 * file's in-memory content with whatever is now stored in the cloud.
 */
export async function resolveCloudConflict(
  resolution: ConflictResolution,
  backend: CloudBackend,
  store: FileStoreState,
): Promise<FileStoreState> {
  if (resolution === 'overwrite-mine') {
    await backend.forceOverwrite(store)
    return store
  }
  const reloaded = await backend.load()
  const remoteContent = reloaded.userFiles[store.active] ?? ''
  return backend.updateActiveContent(store, remoteContent)
}

export function isCloudBackend(
  backend: StorageBackend,
): backend is CloudBackend {
  return backend.kind === 'cloud'
}

export function errorBannerMessage(
  error: CloudBackendError | null,
): string | null {
  if (!error) return null
  if (error.kind === 'network') {
    return "You appear to be offline. Changes will save once you're back online."
  }
  if (error.kind === 'unknown') {
    return `Couldn't save to the cloud: ${error.message}`
  }
  return null
}
