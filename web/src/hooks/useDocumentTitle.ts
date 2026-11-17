import { useEffect } from 'react'
import { displayFileName } from '../fileStore'

export function documentTitleFor(activeFileName: string): string {
  return `${displayFileName(activeFileName)} · Poentaro`
}

/** Keeps the browser tab title in sync with the active file's display name,
 * so switching files/tabs is visible without looking at the app itself. */
export function useDocumentTitle(activeFileName: string): void {
  useEffect(() => {
    document.title = documentTitleFor(activeFileName)
  }, [activeFileName])
}
