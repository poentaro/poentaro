import { useEffect, useMemo, useState } from 'react'
import type { FileStoreState } from '../fileStore'
import { folderStaticShare } from '../fileStoreQueries'
import { buildFolderShareUrl, buildShareUrl } from '../shareUrl'

/** A static (snapshot) link as the Share dialog shows it. */
export interface StaticShareLink {
  /** `null` when the snapshot is too long to be offered as a link. */
  url: string | null
  tooLong: boolean
  /** Files and folders captured; only known for a folder. */
  fileCount?: number
  folderCount?: number
}

/** The `#share=` link of the open file; `null` until it is built. */
export function useFileStaticShareLink(
  open: boolean,
  filename: string,
  content: string | undefined,
): StaticShareLink | null {
  const [url, setUrl] = useState<string | null>(null)
  useEffect(() => {
    if (!open || content === undefined) return
    let cancelled = false
    void buildShareUrl(filename, content).then((built) => {
      if (!cancelled) setUrl(built)
    })
    return () => {
      cancelled = true
    }
  }, [open, filename, content])
  return url === null ? null : { url, tooLong: false }
}

/** The `#share-folder=` link of a folder, encoded in Rust. `null` when the
 * dialog is closed or the folder does not exist. Needs the wasm component
 * initialised, which it is by the time a folder row exists. */
export function useFolderStaticShareLink(
  open: boolean,
  store: FileStoreState,
  folder: string | null,
): StaticShareLink | null {
  return useMemo(() => {
    if (!open || folder === null) return null
    const share = folderStaticShare(store, folder)
    if (!share) return null
    return {
      url:
        share.suffix === undefined ? null : buildFolderShareUrl(share.suffix),
      tooLong: share.tooLong,
      fileCount: share.fileCount,
      folderCount: share.folderCount,
    }
  }, [open, store, folder])
}
