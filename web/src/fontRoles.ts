import fonts from '../../fonts/fonts.json'
import type { FontFamily } from './poentaroWasm'

/** One `fonts/fonts.json` role entry (see that file's own comments). */
export interface FontRole {
  filename: string
  familyCss: string
  /** CSS `font-family` the preview renders this role with instead of
   * pinning `filename` via `@font-face`. */
  previewFamilyCss?: string
}

/** `fonts/fonts.json`'s entry for each `FontFamily` role. */
export const fontRoles: Record<FontFamily, FontRole> = {
  serif: fonts.serif,
  'sans-serif': fonts.sansSerif,
  monospace: fonts.monospace,
}

/** Whether the preview pins `role` to its bundled font file via an
 * injected `@font-face` rule (see `injectFontFaces`). */
function isPinnedInPreview(role: FontRole): boolean {
  return role.previewFamilyCss == null
}

/** Every role the preview pins to its bundled font file. */
export const pinnedPreviewRoles: FontRole[] =
  Object.values(fontRoles).filter(isPinnedInPreview)

/** The CSS `font-family` the preview renders `family`'s text with. */
export function previewFontFamilyCss(family: FontFamily): string {
  const role = fontRoles[family]
  return role.previewFamilyCss ?? role.familyCss
}

/** Whether a `bold`/`italic` toggle visibly changes `family`'s preview text.
 * A pinned role renders from its single bundled Regular file with
 * `font-synthesis: none` (`index.css`), so there's no bold/italic face to
 * switch to; an unpinned role resolves to the viewer's system font, which
 * ships real faces. */
export function hasPreviewStyleFaces(family: FontFamily): boolean {
  return !isPinnedInPreview(fontRoles[family])
}
