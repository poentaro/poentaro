import { ChevronDownIcon } from '@radix-ui/react-icons'
import { forwardRef } from 'react'
import { displayFileName } from '../fileStore'
import type { FileActiveLabel } from '../poentaroWasm'

interface FileSwitcherTriggerProps {
  activeLabel: FileActiveLabel
  isLoadingCloud: boolean
}

/** The toolbar button showing the active file (its folders as a muted
 * prefix); it opens the file browser. Forwards its ref and extra props so
 * the dialog trigger can wrap it. */
export const FileSwitcherTrigger = forwardRef<
  HTMLButtonElement,
  FileSwitcherTriggerProps & React.ComponentPropsWithoutRef<'button'>
>(function FileSwitcherTrigger(
  { activeLabel, isLoadingCloud, ...buttonProps },
  ref,
) {
  return (
    <button
      {...buttonProps}
      ref={ref}
      type="button"
      className="preview-export-btn"
    >
      {activeLabel.folderNames.length > 0 ? (
        <>
          <span className="file-switcher-folder-prefix">
            {`${activeLabel.folderNames.join(' / ')} /`}
          </span>{' '}
        </>
      ) : null}
      {displayFileName(activeLabel.fileName)}
      {isLoadingCloud ? (
        <span className="file-tab-bar-spinner" aria-hidden="true" />
      ) : (
        <ChevronDownIcon className="export-menu-caret" aria-hidden="true" />
      )}
    </button>
  )
})
