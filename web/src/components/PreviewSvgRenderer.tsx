import type { ReactNode } from 'react'
import {
  type DataVariant,
  groupAttributesForTag,
  tagHasPointerCursor,
} from '../dataAttributes'
import { previewFontFamilyCss } from '../fontRoles'
import type { SvgDocument } from '../poentaroWasm'

/** Renders `doc.elements[index]` — the Rust side flattens the element tree
 * into this arena in pre-order, with each group naming its children by
 * index (`SvgGroupKind.childIndices`). */
function renderSvgElement(doc: SvgDocument, index: number): ReactNode {
  // biome-ignore lint/style/noNonNullAssertion: index always comes from the arena's own childIndices/rootElementIndices
  const el = doc.elements[index]!
  const key = index
  const { tag, val: kind } = el.kind
  switch (tag) {
    case 'text':
      return (
        <text
          key={key}
          x={el.x}
          y={el.y}
          data-variant={el.variantTag satisfies DataVariant | undefined}
          fontSize={kind.fontSize}
          textAnchor={
            kind.anchor === 'start'
              ? 'start'
              : kind.anchor === 'middle'
                ? 'middle'
                : 'end'
          }
          dominantBaseline={
            kind.baseline === 'middle'
              ? 'middle'
              : kind.baseline === 'hanging'
                ? 'hanging'
                : 'ideographic'
          }
          fontFamily={previewFontFamilyCss(kind.font)}
          fontWeight={kind.weight === 'normal' ? 'normal' : 'bold'}
          fontStyle={kind.italic ? 'italic' : undefined}
          textDecoration={kind.underline ? 'underline' : undefined}
        >
          {kind.content}
        </text>
      )
    case 'text-with-tspans':
      return (
        <text
          key={key}
          x={el.x}
          y={el.y}
          data-variant={el.variantTag satisfies DataVariant | undefined}
          fontSize={kind.fontSize}
          textAnchor={
            kind.anchor === 'start'
              ? 'start'
              : kind.anchor === 'middle'
                ? 'middle'
                : 'end'
          }
          dominantBaseline={
            kind.baseline === 'middle'
              ? 'middle'
              : kind.baseline === 'hanging'
                ? 'hanging'
                : 'ideographic'
          }
          fontFamily={previewFontFamilyCss(kind.font)}
        >
          {kind.spans.map((span, spanIndex) => (
            <tspan
              // biome-ignore lint/suspicious/noArrayIndexKey: tspans have no stable identifier
              key={spanIndex}
              fontWeight={span.bold ? 'bold' : undefined}
              fontStyle={span.italic ? 'italic' : undefined}
              textDecoration={span.underline ? 'underline' : undefined}
              fontSize={span.fontSize}
            >
              {span.content}
            </tspan>
          ))}
        </text>
      )
    case 'line':
      return (
        <line
          key={key}
          x1={el.x}
          y1={el.y}
          x2={kind.x2}
          y2={kind.y2}
          data-variant={el.variantTag satisfies DataVariant | undefined}
          stroke="black"
          strokeWidth={kind.strokeWidth}
        />
      )
    case 'circle':
      return <circle key={key} cx={el.x} cy={el.y} r={kind.r} fill="black" />
    case 'ring-circle':
      return (
        <circle
          key={key}
          cx={el.x}
          cy={el.y}
          r={kind.r}
          fill="none"
          stroke="black"
          strokeWidth={kind.strokeWidth}
        />
      )
    case 'rounded-bar':
      return (
        <rect
          key={key}
          x={el.x}
          y={el.y}
          width={kind.width}
          height={kind.height}
          rx={kind.height / 2}
          fill="black"
        />
      )
    case 'path':
      return (
        <path
          key={key}
          d={`M ${el.x} ${el.y} Q ${kind.controlX} ${kind.controlY} ${kind.endX} ${kind.endY}`}
          fill="none"
          stroke="black"
          strokeWidth={kind.strokeWidth}
        />
      )
    case 'rect':
      return (
        <rect
          key={key}
          data-testid="measure-highlight"
          x={el.x}
          y={el.y}
          width={kind.width}
          height={kind.height}
          fill="rgba(255,200,0,0.25)"
          rx={2}
        />
      )
    case 'error-rect':
      return (
        <rect
          key={key}
          data-testid="error-highlight"
          x={el.x}
          y={el.y}
          width={kind.width}
          height={kind.height}
          fill="rgba(255,0,0,0.15)"
          rx={2}
        />
      )
    case 'transparent-rect':
      return (
        <rect
          key={key}
          x={el.x}
          y={el.y}
          width={kind.width}
          height={kind.height}
          data-variant={kind.role satisfies DataVariant}
          fill="transparent"
          stroke={
            kind.role === 'section-label-background' ? 'black' : undefined
          }
          strokeWidth={kind.role === 'section-label-background' ? 1 : undefined}
          rx={2}
          style={{
            cursor:
              kind.role === 'bar-line-click-target' ? 'col-resize' : 'pointer',
          }}
        />
      )
    case 'playback-cursor-rect':
      return (
        <rect
          key={key}
          data-variant={tag satisfies DataVariant}
          x={el.x}
          y={el.y}
          width={kind.width}
          height={kind.height}
          fill="transparent"
          rx={2}
          style={{ pointerEvents: 'none' }}
        />
      )
    case 'group': {
      return (
        <g
          key={key}
          {...groupAttributesForTag(kind.tag)}
          style={
            tagHasPointerCursor(kind.tag) ? { cursor: 'pointer' } : undefined
          }
        >
          {Array.from(kind.childIndices, (child) =>
            renderSvgElement(doc, child),
          )}
        </g>
      )
    }
  }
}

export function renderSvgDocument(doc: SvgDocument, key: number): ReactNode {
  return (
    // biome-ignore lint/a11y/noSvgWithoutTitle: synthesized score SVG; title would be redundant with surrounding page context
    <svg
      key={key}
      xmlns="http://www.w3.org/2000/svg"
      width={`${doc.widthPt}pt`}
      height={`${doc.heightPt}pt`}
      viewBox={`0 0 ${Math.round(doc.widthPt)} ${Math.round(doc.heightPt)}`}
    >
      {Array.from(doc.rootElementIndices, (index) =>
        renderSvgElement(doc, index),
      )}
    </svg>
  )
}
