# TODO: file tree (folders of unlimited depth)

Design drafts: file tree storage model and before/after UX (Artifact mockups, captured from the real app).
Model is tentative: file name is a slash-separated path (`Hymns/Advent/O Come.poentaro`); a folder is a shared prefix; `fileIds` stay stable.

## Decisions needed first

- [x] Delete folder: bin each file individually (with its path)
- [x] Folder share: read-only for viewers
- [x] Path length limit: same as S3 object keys, 1024 bytes of UTF-8 for the whole path (CJK names use 3 bytes per character, so count bytes, not characters)
- [ ] Copy `Demo/` into the user's own files? (not asked; default is a later follow-up, out of scope here)
- [x] Tree logic lives in Rust/wasm; TypeScript only renders and passes raw UI actions
- [x] Synced folder share: identified by a `folder_prefix` on the shares row (follows folder renames via the rename-batch route); viewers see changes on reload, no polling/push
- [x] Cloud stores the path as a plain slash-separated string in `name` (backward compatible); folders are a UI concept like S3; explicit empty folders persist as a hidden placeholder file

## Phase 1: model (pure functions, tests in separate files)

- [x] Path rules: no empty segments, no `.`/`..`, a path can't be both file and folder
- [x] Worker also rejects a path that is both file and folder (`ApiError::FileFolderConflict`) on create, rename, restore and batch rename
- [x] Derive folder tree from the flat file map; explicit empty folders (`folders` list)
- [x] Sorting: folders first, then files, alphabetical
- [x] Unique-name logic scoped to a folder
- [x] Rename/move file (keeps `fileId`)
- [x] Rename/move folder as one all-or-nothing batch (collision check)
- [x] Delete folder to bin, bin entries remember their path
- [x] Restore to original path, recreating the folder if gone
- [x] Duplicate stays in the same folder
- [x] Local persistence: optional `folders` field, no version bump
- [x] Update `ARCHITECTURE.md` (key types, glossary: folder, path)

## Phase 2: local UI (matches the real app's look)

- [x] File list becomes a tree in the existing monospace dropdown; widen it
- [x] Toolbar button shows the folder path as a muted prefix
- [x] Active file's ancestors always expanded; expanded state remembered
- [x] Demo becomes a read-only `Demo/` row, last and separated
- [x] `···` menu: add "New folder" and "Move to…"; New creates next to the open file
- [x] Hover `···` on folder rows: New file here, New folder here, Import here, Rename, Move to, Share folder, Delete folder (Share folder deferred to Phase 4)
- [x] Move to… dialog (folder picker, no moving into itself, clash message)
- [x] Rename accepts `/` to move
- [x] Delete folder confirmation with file count
- [x] Bin shows each file's original location
- [x] Empty folder row hint
- [x] Gherkin scenarios (write failing ones first): create, nest, move, rename folder, delete, restore
- [x] Check phone layout of the real app first, then design the tree there

## Phase 3: cloud

- [x] Paths as names in the cloud store; uniqueness on full path
- [x] Batch-rename message in the outbox (no half-delivered folder move)
- [x] Explicit empty folders: placeholder file `<folder>/.keep` (reserved leaf name with no `.poentaro` extension, so users can't collide); worker stays unaware of folders
- [x] Collision-retry naming for folders
- [x] Capture signed-in / Start Sync share states from the real app for mockups

## Phase 4: share a folder

- [x] Share dialog for folders: static link (snapshot of the subtree) and synced link
- [x] Static link size guard: warn and offer synced link when too big
- [x] Synced folder: new files, edits, renames and deletions appear for viewers on reload (no polling or push, same as single-file synced links)
- [x] Shared marker on the folder row, "via" tag on files inside
- [x] Shared folder menu: Copy share link, Sharing settings; nested folder shows "Shared via"
- [x] Warn on delete of a shared folder; moving keeps the link working
- [x] Warn when a file inside already has its own synced link
- [x] Stop sync confirmation
- [x] Viewer: same toolbar as today, file picker rooted at the shared folder, read-only
- [x] Viewer: "Import this folder" (name clash becomes `Hymns 2`) and Discard
- [x] Viewer: "no longer shared" screen (updates on reload, no live push)
- [x] Update `syntax.md` only if `.poentaro` syntax changes (not expected) — unchanged, no syntax change
