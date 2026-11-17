import { UploadIcon } from '@radix-ui/react-icons'
import { useRef } from 'react'

interface ImportButtonProps {
  disabled?: boolean
  importing?: boolean
  /** Menu item text; "Import here" on a folder row. */
  label?: string
  /** Extra class for the button; when set the button is a plain toolbar
   * button instead of a menu item. */
  className?: string
  onImportFile: (file: File) => void
}

/** Lets the user pick a previously exported `.svg`/`.pdf` file and recover
 * its embedded `.poentaro` source (see `extract_embedded_source`). Rendered as
 * a menu item inside the file-actions ("⋯") dropdown. The dropdown must stay
 * open (not close-on-click like the other menu items) until a file is
 * actually chosen — closing it eagerly unmounts the file `<input>` before
 * the OS picker has resolved, discarding the selection. */
export function ImportButton({
  disabled = false,
  importing = false,
  label = 'Import',
  className,
  onImportFile,
}: ImportButtonProps) {
  const inputRef = useRef<HTMLInputElement>(null)

  return (
    <>
      <button
        type="button"
        role={className ? undefined : 'menuitem'}
        className={className ?? 'export-menu-item'}
        disabled={disabled || importing}
        onClick={() => inputRef.current?.click()}
      >
        <UploadIcon aria-hidden="true" />
        {importing ? 'Importing…' : label}
      </button>
      <input
        ref={inputRef}
        type="file"
        accept=".svg,.pdf"
        style={{ display: 'none' }}
        onChange={(event) => {
          const file = event.target.files?.[0]
          event.target.value = ''
          if (file) onImportFile(file)
        }}
      />
    </>
  )
}
