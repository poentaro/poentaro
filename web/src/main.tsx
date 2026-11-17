import { StrictMode } from 'react'
import { createRoot } from 'react-dom/client'
import './index.css'
import { AppEntry } from './appEntry.tsx'
import { hasLegacyLocalStorage } from './legacyStorage/legacyStorageNames.ts'
import { MIGRATE_QUERY_PARAM } from './originMigration/originMigrationPayload.ts'

const root = document.getElementById('root')
if (root == null) {
  throw new Error('missing #root element')
}

function render(container: HTMLElement) {
  createRoot(container).render(
    <StrictMode>
      <AppEntry />
    </StrictMode>,
  )
}

// Data handed over from the app's old origin, and data saved before the
// rename to Poentaro, must be moved before the app first reads storage (see
// "Origin migration" and "Rename to Poentaro" in ARCHITECTURE.md). Only
// such a visit waits and loads that code; every other one renders straight
// away.
if (
  new URLSearchParams(window.location.search).has(MIGRATE_QUERY_PARAM) ||
  hasLegacyLocalStorage()
) {
  void import('./legacyStorage/prepareStorage.ts')
    .then(({ prepareStorage }) => prepareStorage())
    .catch((error: unknown) => console.warn('could not prepare storage', error))
    .then(() => render(root))
} else {
  render(root)
}
