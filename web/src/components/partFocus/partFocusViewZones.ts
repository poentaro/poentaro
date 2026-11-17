import type { editor } from 'monaco-editor'
import type { FocusMeasure } from '../../poentaroWasm'
import { byteOffsetToStringIndex } from '../../utils/byteSpan'
import { createMeasureViewZoneDomNode } from '../editorMeasureViewZones'

const LINE_HEIGHT_PX = 21

function createGhostDomNode(
  lines: string[],
  measureIndex: number,
  position: 'above' | 'below',
): HTMLElement {
  const domNode = document.createElement('div')
  domNode.className = 'part-focus-ghost'
  domNode.dataset.testid = 'part-focus-ghost'
  domNode.dataset.measureIndex = `${measureIndex}`
  domNode.dataset.position = position
  for (const line of lines) {
    const row = document.createElement('div')
    row.className = 'part-focus-ghost-line'
    row.style.height = `${LINE_HEIGHT_PX}px`
    row.textContent = line
    domNode.appendChild(row)
  }
  return domNode
}

/** The line holding the stream's byte `offset`. */
function lineAt(model: editor.ITextModel, text: string, offset: number) {
  return model.getPositionAt(byteOffsetToStringIndex(text, offset)).lineNumber
}

/**
 * Replaces `previousIds` with every measure's view zones: its header row
 * and the ghost lines Rust put above its line, then the ghost lines below
 * it. A measure past `measures` (new, past the score's end) gets a header
 * only; an empty one (a lyric measure without slots) puts all its ghosts
 * above the next measure's line. Returns the new zone ids.
 */
export function applyPartFocusViewZones(
  ed: editor.IStandaloneCodeEditor,
  previousIds: string[],
  measureStarts: Uint32Array,
  measures: FocusMeasure[],
): string[] {
  const model = ed.getModel()
  if (!model) return previousIds
  const text = model.getValue()
  const textByteLength = new TextEncoder().encode(text).length
  const ids: string[] = []
  ed.changeViewZones((accessor) => {
    for (const id of previousIds) accessor.removeZone(id)
    const addZone = (afterLineNumber: number, domNode: HTMLElement) => {
      ids.push(
        accessor.addZone({
          afterLineNumber,
          // Keeps zones sharing a line in the order they're added.
          ordinal: ids.length,
          heightInLines: Math.max(1, domNode.childElementCount),
          domNode,
        }),
      )
    }
    measureStarts.forEach((start, index) => {
      const measure = measures[index]
      const nextStart = measureStarts[index + 1] ?? textByteLength
      const isEmpty = nextStart === start
      const line = lineAt(model, text, start)
      const header = createMeasureViewZoneDomNode(
        measure?.sectionLabel ?? null,
        index,
      )
      header.dataset.testid = 'part-focus-header'
      header.dataset.measureIndex = `${index}`
      addZone(line - 1, header)
      if (!measure) return
      const above = isEmpty
        ? [...measure.linesAbove, ...measure.linesBelow]
        : measure.linesAbove
      if (above.length > 0) {
        addZone(line - 1, createGhostDomNode(above, index, 'above'))
      }
      if (!isEmpty && measure.linesBelow.length > 0) {
        const chunk = text.slice(
          byteOffsetToStringIndex(text, start),
          byteOffsetToStringIndex(text, nextStart),
        )
        const lastLine = line + chunk.trimEnd().split('\n').length - 1
        addZone(
          lastLine,
          createGhostDomNode(measure.linesBelow, index, 'below'),
        )
      }
    })
  })
  return ids
}
