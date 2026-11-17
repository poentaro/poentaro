import { type RefObject, useMemo, useRef } from 'react'
import type { EditorHandle, PartFocusSelectionHandle } from '../types'

/** The part focus editor's selection handle, and the editor handle the
 * preview selects through. While the part focus editor is open, the preview
 * selects in it rather than in the main editor underneath. */
export function usePartFocusSelectionRouting(
  editorRef: RefObject<EditorHandle | null>,
) {
  const partFocusHandleRef = useRef<PartFocusSelectionHandle | null>(null)
  const selectionEditorRef = useMemo<RefObject<EditorHandle | null>>(
    () => ({
      get current() {
        const main = editorRef.current
        const partFocus = partFocusHandleRef.current
        return main && partFocus ? { ...main, ...partFocus } : main
      },
    }),
    [editorRef],
  )
  return { partFocusHandleRef, selectionEditorRef }
}
