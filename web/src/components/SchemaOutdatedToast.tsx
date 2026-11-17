import * as Toast from '@radix-ui/react-toast'
import { useSyncExternalStore } from 'react'
import {
  isSchemaOutdated,
  subscribeSchemaOutdated,
} from '../protocol/schemaOutdated'
import './ExportAudioToast.css'

/** Asks to reload once the worker answers that this tab was built from an
 * older schema: nothing this tab sends will be understood until then. */
export function SchemaOutdatedToast() {
  const outdated = useSyncExternalStore(
    subscribeSchemaOutdated,
    isSchemaOutdated,
  )
  return (
    <Toast.Provider swipeDirection="right" duration={Infinity}>
      <Toast.Root
        className="export-audio-toast"
        data-testid="schema-outdated-toast"
        open={outdated}
      >
        <Toast.Description>Poentaro was updated.</Toast.Description>
        <Toast.Action altText="Reload" asChild>
          <button
            type="button"
            className="preview-export-btn"
            onClick={() => window.location.reload()}
          >
            Reload
          </button>
        </Toast.Action>
      </Toast.Root>
      <Toast.Viewport className="export-audio-toast-viewport" />
    </Toast.Provider>
  )
}
