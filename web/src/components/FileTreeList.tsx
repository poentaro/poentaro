import * as Checkbox from '@radix-ui/react-checkbox'
import {
  CheckIcon,
  ChevronDownIcon,
  ChevronRightIcon,
  DividerHorizontalIcon,
  Link2Icon,
} from '@radix-ui/react-icons'
import { type CSSProperties, useState } from 'react'
import { displayFileName } from '../fileStore'
import type { CheckState, FileTreeViewRow } from '../poentaroWasm'
import { FileTabName } from './FileTabName'
import { type FolderActions, FolderRowMenu } from './FolderRowMenu'
import { HighlightedName } from './HighlightedName'

function FolderGlyph() {
  return (
    <svg
      className="file-tree-folder-icon"
      viewBox="0 0 15 15"
      fill="none"
      stroke="currentColor"
      strokeWidth="1"
      aria-hidden="true"
    >
      <path d="M1.5 3.5h4l1.5 1.5h6.5v6.5a1 1 0 0 1-1 1h-10a1 1 0 0 1-1-1z" />
    </svg>
  )
}

/** The muted "via <Folder>" tag of a row inside a shared folder. */
function ViaTag({ name }: { name: string }) {
  return (
    <span
      className="file-tree-via-tag"
      aria-hidden="true"
    >{`via ${name}`}</span>
  )
}

/** A row's checkbox in select mode. Its hit area is stretched over the whole
 * row (see `.file-tree-check::after`), so any click on the row toggles it. */
function RowCheckbox({
  check,
  label,
  onToggle,
}: {
  check: CheckState
  label: string
  onToggle: () => void
}) {
  return (
    <Checkbox.Root
      className="file-tree-check"
      checked={check === 'mixed' ? 'indeterminate' : check === 'checked'}
      onCheckedChange={onToggle}
      aria-label={label}
    >
      <Checkbox.Indicator className="file-tree-check-indicator">
        {check === 'mixed' ? <DividerHorizontalIcon /> : <CheckIcon />}
      </Checkbox.Indicator>
    </Checkbox.Root>
  )
}

/** What a folder row shows after its chevron. */
function FolderLabel({ row }: { row: FileTreeViewRow }) {
  return (
    <>
      <FolderGlyph />
      <span className="file-tree-name">
        <HighlightedName text={row.name} match={row.nameMatch} />
      </span>
      {row.shareId ? (
        <Link2Icon className="file-tree-share-marker" aria-hidden="true" />
      ) : null}
      {!row.shareId && row.sharedVia ? (
        <ViaTag name={row.sharedVia.name} />
      ) : null}
      <span className="file-tree-count" aria-hidden="true">
        {row.fileCount}
      </span>
    </>
  )
}

export interface FileTreeListProps {
  /** Already-visible rows, computed in Rust (`fileTreeView`). */
  rows: FileTreeViewRow[]
  onToggleFolder: (key: string) => void
  onSelect: (name: string) => void
  /** A shared folder seen by a viewer: only folding and selecting, no rename
   * and no folder menus. The editing props below are then not needed. */
  readOnly?: boolean
  renamingName?: string | null
  onRename?: (from: string, to: string) => void
  /** Rust's reason a rename/move would be rejected, if it would. */
  renameError?: (from: string, to: string) => string | undefined
  /** The folder rows' `···` menu; the read-only `Demo/` row has none. */
  folderActions?: FolderActions
  creating?: boolean
  creatingFolder?: boolean
  importing?: boolean
  /** Select mode: user rows get a checkbox and a click anywhere on them
   * toggles it (given the row's key) instead of opening or renaming. */
  onToggleSelect?: (key: string) => void
}

/** Extra classes of a row in select mode: it is clickable as a whole, and a
 * checked one is highlighted. */
function rowSelectionClass(
  selecting: boolean,
  check: CheckState | undefined,
): string {
  if (!selecting) return ''
  return ` file-tree-row--selecting${check === 'checked' ? ' file-tree-row--checked' : ''}`
}

/** A row's identity in the list; keys alone repeat across kinds (a demo
 * file and a top-level file can share a name). */
function rowId(row: FileTreeViewRow): string {
  return row.kind === 'empty-folder-hint'
    ? `hint:${row.depth}:${row.name}:${row.key}`
    : `${row.kind}:${row.key}`
}

export function FileTreeList({
  rows,
  readOnly = false,
  renamingName = null,
  onToggleFolder,
  onSelect,
  onRename,
  renameError,
  folderActions,
  creating = false,
  creatingFolder = false,
  importing = false,
  onToggleSelect,
}: FileTreeListProps) {
  // The hovered row, else the active file, has its folders highlighted.
  const [hoveredRowId, setHoveredRowId] = useState<string | null>(null)
  const focusRow =
    rows.find((row) => rowId(row) === hoveredRowId) ??
    rows.find((row) => row.isActive)
  const ancestorKeys = new Set(focusRow?.ancestorKeys ?? [])
  return (
    <ul
      className="file-tabs"
      aria-label="Files"
      onMouseLeave={() => setHoveredRowId(null)}
    >
      {rows.map((row) => {
        const rowStyle = { '--tree-depth': row.depth } as CSSProperties
        const onMouseEnter = () => setHoveredRowId(rowId(row))
        switch (row.kind) {
          case 'folder':
          case 'demo-folder': {
            const demo = row.kind === 'demo-folder'
            const selecting =
              onToggleSelect !== undefined && row.check !== undefined
            const chevron = row.expanded ? (
              <ChevronDownIcon aria-hidden="true" />
            ) : (
              <ChevronRightIcon aria-hidden="true" />
            )
            return (
              <li
                key={rowId(row)}
                className={`file-tree-row file-tree-folder-row${demo ? ' file-tree-demo-row' : ''}${ancestorKeys.has(row.key) ? ' file-tree-row--ancestor' : ''}${rowSelectionClass(selecting, row.check)}`}
                style={rowStyle}
                onMouseEnter={onMouseEnter}
              >
                {selecting ? (
                  <>
                    <RowCheckbox
                      check={row.check ?? 'unchecked'}
                      label={`Select folder ${row.name}`}
                      onToggle={() => onToggleSelect(row.key)}
                    />
                    <div className="file-tree-folder file-tree-folder--selecting">
                      <button
                        type="button"
                        className="file-tree-chevron"
                        aria-expanded={row.expanded}
                        aria-label={`${row.expanded ? 'Collapse' : 'Expand'} folder ${row.name}`}
                        onClick={() => onToggleFolder(row.key)}
                      >
                        {chevron}
                      </button>
                      <FolderLabel row={row} />
                    </div>
                  </>
                ) : (
                  <button
                    type="button"
                    className="file-tree-folder"
                    aria-expanded={row.expanded}
                    onClick={() => onToggleFolder(row.key)}
                  >
                    {chevron}
                    <FolderLabel row={row} />
                  </button>
                )}
                {row.kind === 'folder' &&
                !selecting &&
                !readOnly &&
                folderActions ? (
                  <FolderRowMenu
                    folder={row.key}
                    name={row.name}
                    actions={folderActions}
                    creating={creating}
                    creatingFolder={creatingFolder}
                    importing={importing}
                    shareId={row.shareId}
                    sharedVia={row.sharedVia}
                  />
                ) : null}
              </li>
            )
          }
          case 'empty-folder-hint':
            return (
              <li
                key={rowId(row)}
                className="file-tree-row file-tree-empty-hint"
                style={rowStyle}
                onMouseEnter={onMouseEnter}
              >
                {row.name}
              </li>
            )
          default: {
            const selecting =
              onToggleSelect !== undefined && row.check !== undefined
            return (
              <li
                key={rowId(row)}
                className={`file-tab file-tree-row${row.isActive ? ' file-tab--active' : ''}${rowSelectionClass(selecting, row.check)}`}
                style={rowStyle}
                onMouseEnter={onMouseEnter}
              >
                {selecting ? (
                  <RowCheckbox
                    check={row.check ?? 'unchecked'}
                    label={`Select ${displayFileName(row.name)}`}
                    onToggle={() => onToggleSelect(row.key)}
                  />
                ) : null}
                <span className="file-tree-spacer" aria-hidden="true" />
                {selecting ? (
                  <span className="file-tab-name">
                    <HighlightedName
                      text={displayFileName(row.name)}
                      match={row.nameMatch}
                    />
                  </span>
                ) : readOnly ? (
                  <button
                    type="button"
                    className="file-tab-name"
                    aria-current={row.isActive ? 'true' : undefined}
                    onClick={() => {
                      if (!row.isActive) onSelect(row.key)
                    }}
                  >
                    <HighlightedName
                      text={displayFileName(row.name)}
                      match={row.nameMatch}
                    />
                  </button>
                ) : (
                  <FileTabName
                    name={row.key}
                    label={
                      <HighlightedName
                        text={displayFileName(row.name)}
                        match={row.nameMatch}
                      />
                    }
                    active={row.isActive}
                    onSelect={onSelect}
                    onRename={onRename ?? (() => {})}
                    renameError={renameError}
                    renaming={renamingName === row.key}
                  />
                )}
                {row.sharedVia ? <ViaTag name={row.sharedVia.name} /> : null}
              </li>
            )
          }
        }
      })}
    </ul>
  )
}
