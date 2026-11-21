import { useEffect, useRef, useState } from 'react'
import type { FileStoreState } from '../fileStore'
import { parseSyncedShare } from '../syncedShareUrl'
import { readFileNameFromUrl, writeFileNameToUrl } from '../urlFileParam'

/**
 * Keeps the active file and the `?file=` URL param in sync: on first load,
 * opens whichever file the URL names (if any), and thereafter keeps the
 * URL updated to whichever file is active, so reloading the page lands back
 * on the same file instead of the picker/default file.
 *
 * Initial selection is deferred until `isLoadingCloud` is false (relevant
 * for the cloud backend, whose file list isn't known synchronously) so the
 * file is actually found rather than silently skipped — opening an unknown
 * name does nothing. The "ready" flag is set once the file is open (its text
 * read), so the URL-sync effect only ever observes the *resolved*
 * active file, never a stale default it would otherwise briefly (or, if the
 * URL's file turns out not to exist, permanently) overwrite the URL with.
 *
 * Skipped entirely while a synced-share link (`?s=`) is open: a viewer's `store.active`
 * is just whatever local file happened to be selected before/independent of
 * the synced share, and it must not leak into the address bar as `?file=`.
 */
export function useUrlFileSync(
  store: FileStoreState,
  openFile: (name: string) => Promise<void>,
  isLoadingCloud: boolean,
): void {
  const initialUrlFileAppliedRef = useRef(false)
  const [initialSelectionReady, setInitialSelectionReady] = useState(false)
  const isSyncedShareViewer = parseSyncedShare() !== null

  useEffect(() => {
    if (
      initialUrlFileAppliedRef.current ||
      isLoadingCloud ||
      isSyncedShareViewer
    )
      return
    initialUrlFileAppliedRef.current = true
    const urlFile = readFileNameFromUrl()
    if (!urlFile) {
      setInitialSelectionReady(true)
      return
    }
    // Ready only once the file is open, or the URL would be rewritten to
    // the default file while its text is still being read.
    void openFile(urlFile).finally(() => setInitialSelectionReady(true))
  }, [isLoadingCloud, isSyncedShareViewer, openFile])

  useEffect(() => {
    if (!initialSelectionReady || isSyncedShareViewer) return
    writeFileNameToUrl(store.active)
  }, [initialSelectionReady, isSyncedShareViewer, store.active])
}
