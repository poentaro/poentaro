import type { editor } from 'monaco-editor'
import { type RefObject, useCallback, useLayoutEffect, useRef } from 'react'

/**
 * Writes `value` into the Monaco editor only when it is a change from
 * outside the editor (formatting, restoring a version, a cloud update),
 * keeping every cursor where it was.
 *
 * `value` is not handed to `@monaco-editor/react`, because it syncs it in a
 * passive `useEffect`. Passive effects can run after the next keystroke has
 * already landed, so it would compare an older `value` with newer text and
 * write the older text back, dropping or reordering what was typed (issue
 * #149). A layout effect runs in the same task as the render that read
 * `value`, so no keystroke can land in between: when `value` differs from
 * the model there, it really is an outside change.
 *
 * Returns `syncModel`, which the editor also calls when it switches to
 * another file's model, since that model may hold older text.
 */
export function useEditorValueSync(
  editorRef: RefObject<editor.IStandaloneCodeEditor | null>,
  value: string,
): () => void {
  const valueRef = useRef(value)
  valueRef.current = value

  const syncModel = useCallback(() => {
    const ed = editorRef.current
    const model = ed?.getModel()
    if (!ed || !model || model.getValue() === valueRef.current) return
    const selections = ed.getSelections()
    model.pushStackElement()
    model.pushEditOperations(
      selections ?? [],
      [{ range: model.getFullModelRange(), text: valueRef.current }],
      () => selections,
    )
    model.pushStackElement()
    if (selections) ed.setSelections(selections)
  }, [editorRef])

  // biome-ignore lint/correctness/useExhaustiveDependencies: value is the trigger; syncModel reads it from the ref
  useLayoutEffect(syncModel, [value, syncModel])

  return syncModel
}
