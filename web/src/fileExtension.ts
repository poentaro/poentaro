import fileExtension from '../../file-extension.json'

/** Every file the app saves ends in this. Rust's `file_tree::FILE_EXTENSION`
 * is generated from the same `file-extension.json`. */
export const FILE_EXTENSION = fileExtension.extension

/** What files ended in before the app was renamed to Poentaro. They still
 * open, and are renamed where the app can (see "Rename to Poentaro" in
 * `ARCHITECTURE.md`). */
export const LEGACY_FILE_EXTENSION = fileExtension.legacyExtension

/** `name` without its extension, current or legacy. */
export function stripFileExtension(name: string): string {
  const extension = [FILE_EXTENSION, LEGACY_FILE_EXTENSION].find((candidate) =>
    name.endsWith(candidate),
  )
  return extension === undefined ? name : name.slice(0, -extension.length)
}

/** `name`, with the extension appended unless it already has one (current
 * or legacy). */
export function withFileExtension(name: string): string {
  return stripFileExtension(name) === name ? `${name}${FILE_EXTENSION}` : name
}
