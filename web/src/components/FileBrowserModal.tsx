import * as Dialog from '@radix-ui/react-dialog'
import {
  CheckIcon,
  Cross1Icon,
  FilePlusIcon,
  MagnifyingGlassIcon,
  PlusIcon,
} from '@radix-ui/react-icons'
import { type ReactNode, useRef, useState } from 'react'
import { displayFileName } from '../fileStore'
import type { CloudLoadFailure } from '../hooks/useCloudStoreLoader'
import type { FileSelection } from '../hooks/useFileSelection'
import type { FileTreeViewRow } from '../poentaroWasm'
import './file-browser.css'
import { FileTreeList, type FileTreeListProps } from './FileTreeList'
import { ImportButton } from './ImportButton'
import { ResponsiveMenuContainerContext } from './ResponsiveMenu'
import { SpinnerLabel } from './SpinnerLabel'
import { TraceId } from './TraceId'

export interface FileBrowserModalProps {
  open: boolean
  onOpenChange: (open: boolean) => void
  /** The button that opens the browser (rendered as the dialog trigger). */
  trigger: ReactNode
  /** Path of the file being edited, shown in the header. */
  activePath: string
  query: string
  onQueryChange: (query: string) => void
  /** Rows for the current `query`, computed in Rust. */
  rows: FileTreeViewRow[]
  isLoadingCloud: boolean
  cloudLoadFailure: CloudLoadFailure | null
  /** The store has no files at all (as opposed to no search match). */
  storeIsEmpty: boolean
  treeProps: Omit<FileTreeListProps, 'rows'>
  creating: boolean
  creatingFolder: boolean
  importing: boolean
  onNewFile: () => void
  onNewFolder: () => void
  onImportFile?: (file: File) => void
  /** The multi-select: the Select button, the checkboxes and Escape. */
  selection: FileSelection
  /** The bar (with its dialogs) shown while `selection.selecting`. */
  selectionBar: ReactNode
}

/** The full-screen file browser opened by the file switcher's trigger
 * button: a search box, the New / New folder / Import buttons and the file
 * tree, which has the room the old dropdown lacked. */
export function FileBrowserModal({
  open,
  onOpenChange,
  trigger,
  activePath,
  query,
  onQueryChange,
  rows,
  isLoadingCloud,
  cloudLoadFailure,
  storeIsEmpty,
  treeProps,
  creating,
  creatingFolder,
  importing,
  onNewFile,
  onNewFolder,
  onImportFile,
  selection,
  selectionBar,
}: FileBrowserModalProps) {
  const searchRef = useRef<HTMLInputElement>(null)
  const bodyRef = useRef<HTMLDivElement>(null)
  const [menuContainer, setMenuContainer] = useState<HTMLElement | null>(null)
  const searching = query.trim() !== ''

  return (
    <Dialog.Root open={open} onOpenChange={onOpenChange}>
      <Dialog.Trigger asChild>{trigger}</Dialog.Trigger>
      <Dialog.Portal>
        <Dialog.Content
          ref={setMenuContainer}
          className="file-browser"
          data-testid="file-browser"
          aria-describedby={undefined}
          onOpenAutoFocus={(event) => {
            event.preventDefault()
            // Focusing the search box would raise the keyboard on a phone.
            if (window.matchMedia('(hover: hover)').matches) {
              searchRef.current?.focus()
            }
            requestAnimationFrame(() =>
              bodyRef.current
                ?.querySelector('[aria-current="true"]')
                ?.scrollIntoView({ block: 'center' }),
            )
          }}
          onInteractOutside={(event) => {
            // The prompts opened from the tree (rename, move, delete, new
            // folder) are separate dialogs, so Radix sees a click or focus
            // inside them as outside this one. They sit on top of the
            // browser, which must stay open underneath.
            const target = event.target
            if (target instanceof Element && target.closest('[role="dialog"]'))
              event.preventDefault()
          }}
          onEscapeKeyDown={(event) => {
            // The first Escape clears the search, the next leaves select mode;
            // the one after closes the browser.
            if (query !== '') {
              event.preventDefault()
              onQueryChange('')
            } else if (selection.selecting) {
              event.preventDefault()
              selection.stop()
            }
          }}
        >
          <header className="file-browser-header">
            <div className="file-browser-header-inner">
              <Dialog.Title className="file-browser-title">Files</Dialog.Title>
              <span className="file-browser-where">
                {displayFileName(activePath)}
              </span>
              <Dialog.Close asChild>
                <button
                  type="button"
                  className="file-browser-close"
                  aria-label="Close files"
                >
                  <Cross1Icon aria-hidden="true" />
                </button>
              </Dialog.Close>
            </div>
          </header>
          <div className="file-browser-toolbar">
            <div className="file-browser-toolbar-inner">
              <label className="file-browser-search">
                <MagnifyingGlassIcon aria-hidden="true" />
                <input
                  ref={searchRef}
                  type="search"
                  value={query}
                  placeholder="Search files"
                  aria-label="Search files"
                  autoComplete="off"
                  onChange={(event) => onQueryChange(event.target.value)}
                />
              </label>
              {selection.selecting ? null : (
                <div className="file-browser-actions">
                  <button
                    type="button"
                    className="file-browser-btn file-browser-btn--primary"
                    disabled={creating}
                    onClick={onNewFile}
                  >
                    <PlusIcon aria-hidden="true" />
                    <SpinnerLabel pending={creating} label="New file" />
                  </button>
                  <button
                    type="button"
                    className="file-browser-btn"
                    disabled={creatingFolder}
                    onClick={onNewFolder}
                  >
                    <FilePlusIcon aria-hidden="true" />
                    <SpinnerLabel pending={creatingFolder} label="New folder" />
                  </button>
                  {onImportFile ? (
                    <ImportButton
                      className="file-browser-btn"
                      disabled={isLoadingCloud}
                      importing={importing}
                      onImportFile={onImportFile}
                    />
                  ) : null}
                  <button
                    type="button"
                    className="file-browser-btn"
                    disabled={isLoadingCloud || storeIsEmpty}
                    onClick={selection.start}
                  >
                    <CheckIcon aria-hidden="true" />
                    Select
                  </button>
                </div>
              )}
            </div>
          </div>
          {selection.selecting ? selectionBar : null}
          <div className="file-browser-body" ref={bodyRef}>
            <div className="file-browser-body-inner">
              {isLoadingCloud && cloudLoadFailure ? (
                <p className="file-browser-hint">
                  Couldn't load files from the cloud. Retrying…
                  <TraceId traceId={cloudLoadFailure.traceId} />
                </p>
              ) : isLoadingCloud ? (
                <p className="file-browser-hint">
                  Loading files from the cloud…
                </p>
              ) : storeIsEmpty ? (
                <p className="file-browser-hint">
                  No files yet. Click New file to create one.
                </p>
              ) : searching && rows.length === 0 ? (
                <p className="file-browser-hint">{`No files match “${query.trim()}”.`}</p>
              ) : null}
              <ResponsiveMenuContainerContext.Provider value={menuContainer}>
                <FileTreeList
                  rows={rows}
                  {...treeProps}
                  onToggleSelect={
                    selection.selecting ? selection.toggle : undefined
                  }
                />
              </ResponsiveMenuContainerContext.Provider>
            </div>
          </div>
        </Dialog.Content>
      </Dialog.Portal>
    </Dialog.Root>
  )
}
