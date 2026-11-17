# TODO: e2e coverage for the GitHub storage backend

`web/e2e/rename-file-github.spec.ts` covers rename. The same pattern (mock
`https://api.github.com/repos/{owner}/jianpu-generator-storage/contents/**`
via `page.route`, seed `poentaro:storage-backend:v1` + `poentaro:github-auth:v1`
in `page.addInitScript`, no real OAuth/network) still needs to be written for:

- [x] **new** (`FileTabBar`'s "New" button → `createFile`/`pureCreateFile`,
      `githubBackend.ts`'s `createFile`) — expect a `PUT contents/scores/<name>.poentaro`
      (create-only, no prior `sha` lookup) and the new tab to appear active.
      See `web/e2e/new-file-github.spec.ts`.
- [x] **duplicate** (`FileTabBar`'s "Duplicate" button → `duplicateFile`) —
      same create-only `PUT` as new, but seeded from an existing file's
      content; assert the duplicate's preview matches the source's.
      See `web/e2e/duplicate-file-github.spec.ts`.
- [x] **delete** (tab's `×` / `file-tab-close`, `aria-label="Move <name> to bin"`
      → `deleteFile`) — expect `PUT contents/trash/<name>.poentaro` then
      `DELETE contents/scores/<name>.poentaro` (mirrors rename's create+delete
      pair, just to `trash/` instead of a renamed path); assert the tab
      disappears and the name shows up under the "Bin (n)" `<details>`.
      See `web/e2e/delete-file-github.spec.ts`.
- [x] **restore** (bin item's `↩` / `file-tab-bar-restore`, `aria-label="Restore
      <name>"` → `restoreFile`) — expect `PUT contents/scores/<name>.poentaro`
      then `DELETE contents/trash/<name>.poentaro`; assert the tab reappears
      outside the bin.
      See `web/e2e/restore-file-github.spec.ts`.
- [x] **import** (shared-link flow, see `web/e2e/share.spec.ts` for the local-
      backend version) — open `/#share=...` while the GitHub backend is
      already active/seeded, click "Import to my scores", and expect the same
      create-only `PUT contents/scores/<name>.poentaro` as new/duplicate.
      Required adding `StorageBackend.importFile` (mirroring `createFile`)
      and wiring `App.tsx`'s `handleImportShared` through it — previously it
      called `setStore` directly, which never actually persisted an import
      on the GitHub backend since `useStorageBackend`'s autosave only fires
      for edits to the *same* active file, not a newly-switched-to one.
      See `web/e2e/import-shared-file-github.spec.ts`.

For each: also add a post-reload assertion (like the rename test's) that
re-fetches through the mock, to prove the operation actually landed in the
fake remote rather than just updating in-memory React state.

Common mock gaps to watch for beyond what `rename-file-github.spec.ts`
already handles:
- `deleteFile`/`restoreFile` touch **both** `scores/` and `trash/` prefixes —
  the existing mock's path-based `Map` already supports this, no changes
  needed there.
- `restoreFile` can target a name that collides with something already in
  `scores/` (see `fileStore.ts`'s `restoreFile`/`uniqueName`), renaming the
  restored file rather than overwriting the existing one. Covered by
  `web/e2e/restore-file-collision-github.spec.ts`.
