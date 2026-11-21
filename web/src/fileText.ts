import type { FileText } from './poentaroWasm'

/** A file's text that is in memory. Never empty-by-default: see
 * {@link FileText}. */
export function loadedFileText(text: string): FileText {
  return { tag: 'loaded', val: text }
}

/** Every text of a plain name -> text record as loaded. */
export function loadedFiles(
  files: Record<string, string>,
): Record<string, FileText> {
  return Object.fromEntries(
    Object.entries(files).map(([name, text]) => [name, loadedFileText(text)]),
  )
}

export const NOT_LOADED: FileText = { tag: 'not-loaded' }
