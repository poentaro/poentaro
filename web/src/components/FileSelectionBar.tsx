import {
  CheckIcon,
  Cross2Icon,
  MoveIcon,
  Pencil1Icon,
  TrashIcon,
} from '@radix-ui/react-icons'
import { useState } from 'react'
import type { FileStoreState } from '../fileStore'
import {
  fileMoveTargets,
  folderMoveError,
  foldersToReveal,
  renameError,
  selectionAll,
  selectionDeleteSummary,
  selectionMovePlan,
  selectionUnits,
} from '../fileStoreQueries'
import type { FileSelection } from '../hooks/useFileSelection'
import type {
  SelectionMove,
  SelectionUnit,
  ShareOverview,
} from '../poentaroWasm'
import { DeleteSelectionDialog } from './DeleteSelectionDialog'
import { MoveToDialog } from './MoveToDialog'
import { RenameFileDialog } from './RenameFileDialog'

export interface FileSelectionBarProps {
  store: FileStoreState
  selection: FileSelection
  /** The owner's live links, for the delete warning. */
  shares: ShareOverview
  onRename: (from: string, to: string) => void
  onMoveFolder: (from: string, to: string) => void
  onMoveSelection: (moves: SelectionMove[]) => void
  onDeleteSelection: (units: SelectionUnit[]) => void
  expandFolders: (folders: string[]) => void
}

type SelectionDialog = 'rename' | 'move' | 'delete'

/** The bar shown in select mode: how much is ticked, Select all / Clear, and
 * the Rename / Move to / Delete buttons with their dialogs. What a ticked
 * folder means, whether a move clashes and what a delete lists all come from
 * Rust; this only wires them to the existing backend operations. */
export function FileSelectionBar({
  store,
  selection,
  shares,
  onRename,
  onMoveFolder,
  onMoveSelection,
  onDeleteSelection,
  expandFolders,
}: FileSelectionBarProps) {
  const [dialog, setDialog] = useState<SelectionDialog | null>(null)
  const { selected } = selection
  const units = selected.length > 0 ? selectionUnits(store, selected) : []
  const single = units.length === 1 ? units[0] : undefined
  const allSelected =
    selected.length > 0 && selected.length === selectionAll(store).length
  const nothing = selected.length === 0
  const onOpenChange = (open: boolean) => {
    if (!open) setDialog(null)
  }
  const moveAndReveal = (from: string, to: string) => {
    onMoveFolder(from, to)
    expandFolders(foldersToReveal(to))
    selection.stop()
  }
  const moveTo = (folder: string) => {
    const target = folder === '' ? undefined : folder
    onMoveSelection(selectionMovePlan(store, selected, target).moves)
    if (target !== undefined) expandFolders(foldersToReveal(target))
    selection.stop()
  }

  return (
    <div className="file-browser-selection" data-testid="file-selection-bar">
      <div className="file-browser-selection-inner">
        {allSelected ? null : (
          <button
            type="button"
            className="file-browser-btn"
            onClick={selection.selectAll}
          >
            <CheckIcon aria-hidden="true" />
            Select all
          </button>
        )}
        {nothing ? null : (
          <button
            type="button"
            className="file-browser-btn"
            onClick={selection.clear}
          >
            <Cross2Icon aria-hidden="true" />
            Clear
          </button>
        )}
        <span className="file-browser-selection-count" aria-live="polite">
          {nothing ? 'Nothing selected' : `${selected.length} selected`}
        </span>
        <button
          type="button"
          className="file-browser-btn"
          disabled={single === undefined}
          onClick={() => setDialog('rename')}
        >
          <Pencil1Icon aria-hidden="true" />
          Rename
        </button>
        <button
          type="button"
          className="file-browser-btn"
          disabled={nothing}
          onClick={() => setDialog('move')}
        >
          <MoveIcon aria-hidden="true" />
          Move to…
        </button>
        <button
          type="button"
          className="file-browser-btn file-browser-btn--danger"
          disabled={nothing}
          onClick={() => setDialog('delete')}
        >
          <TrashIcon aria-hidden="true" />
          Delete
        </button>
        <button
          type="button"
          className="file-browser-btn"
          onClick={selection.stop}
        >
          Done
        </button>
      </div>
      <RenameFileDialog
        open={dialog === 'rename' && single !== undefined}
        onOpenChange={onOpenChange}
        path={single?.path ?? ''}
        kind={single?.isFolder ? 'folder' : 'file'}
        onRename={(from, to) => {
          if (single?.isFolder) {
            moveAndReveal(from, to)
          } else {
            onRename(from, to)
            selection.stop()
          }
        }}
        renameError={(from, to) =>
          single?.isFolder
            ? folderMoveError(store, from, to)
            : renameError(store, from, to)
        }
      />
      <MoveToDialog
        open={dialog === 'move'}
        onOpenChange={onOpenChange}
        path=""
        subject={`${selected.length} selected ${selected.length === 1 ? 'file' : 'files'}`}
        currentFolder={undefined}
        hasCurrentFolder={false}
        targets={dialog === 'move' ? fileMoveTargets(store) : []}
        destination={(folder) => folder ?? ''}
        moveError={(to) =>
          selectionMovePlan(store, selected, to === '' ? undefined : to).error
        }
        onMove={(_from, to) => moveTo(to)}
      />
      <DeleteSelectionDialog
        open={dialog === 'delete'}
        onOpenChange={onOpenChange}
        summary={
          dialog === 'delete'
            ? selectionDeleteSummary(store, selected, shares)
            : undefined
        }
        onConfirm={() => {
          onDeleteSelection(units)
          selection.stop()
        }}
      />
    </div>
  )
}
