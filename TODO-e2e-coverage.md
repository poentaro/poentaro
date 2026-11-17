# TODO: remaining e2e coverage gaps

Ranked by real user-facing risk (a bug here would actually reach a user
undetected), not by ease of writing the test. See `web/e2e/` for existing
specs and `web/e2e/github-contents-mock.ts` for the GitHub Contents API mock
helper used by the `*-github.spec.ts` files.

- [x] **GitHub 409 conflict resolution** (`StorageSettingsModal.tsx:89`
      `resolveGithubConflict`, `handleResolveConflict` at line 277) — the
      highest-risk gap: a regression here could silently destroy a user's
      unsaved edits. Extend `mockGithubContentsApi` (or add a variant) so a
      `PUT` returns `409`, driving `githubBackend.ts:292`'s
      `{ kind: 'conflict', path }`. Then assert:
      - `data-testid="conflict-banner"` appears with the conflicting path.
      - Clicking "Overwrite mine" (`onClick={() =>
        handleResolveConflict('overwrite-mine')}`) re-PUTs the in-memory
        content and the banner clears.
      - Clicking "Discard mine" reloads the backend's listing and the
        editor's content is replaced with what the mock now serves for that
        path (assert the visible source text changed, not just that the
        banner cleared).

- [x] **GitHub OAuth connect/disconnect device flow**
      (`StorageSettingsModal.tsx:228` `handleConnect`, `:254`
      `handleDisconnect`). Every existing `*-github.spec.ts` test seeds
      `poentaro:github-auth:v1` via `page.addInitScript`, so `handleConnect`'s
      device-code flow (`connectWithDeviceFlow`, `GITHUB_OAUTH_PROXY_URL`)
      has never run through the UI. Mock the device-flow proxy + GitHub's
      `users.getAuthenticated`/`repos.get` endpoints via `page.route` and
      assert:
      - Clicking the "GitHub repository" radio with no stored auth shows
        `data-testid="github-connect"`, then clicking its button shows
        `data-testid="device-verification"` with the `user_code`.
      - Once the mocked proxy resolves, `data-testid="github-connected"`
        appears with the right `@username`.
      - Clicking "Disconnect" clears `poentaro:github-auth:v1` and reverts to
        the local backend (assert via `switchBackend`'s visible effect, e.g.
        the radio selection and that a subsequent edit doesn't hit
        `API_PREFIX`).

- [x] **PDF export** (`Preview.tsx:359` `canExportPdf`/`onExportPdf`,
      `usePoentaroWorker.ts:490` `exportPdf`, `:526` `exportSplitPdf`,
      `workerHelpers.ts:41` `downloadPdf`). Zero e2e coverage on a primary
      output button. Using Playwright's download event
      (`page.waitForEvent('download')`), assert:
      - Clicking "Export PDF" with a valid document produces a downloaded
        file with a non-trivial size (catches a silently-empty/corrupt PDF).
      - Clicking "Export parts" (`canExportSplitPdf`, requires
        `partsCount > 0`) on a multi-part score produces a download too —
        this path is separate from `exportPdf` and could regress
        independently.
      - Both buttons are disabled while `rendering`/`exporting` is true
        (`disabled={!canExportPdf}` / `!canExportSplitPdf`).

- [x] **Cmd/Ctrl+S force-save shortcut** (`App.tsx:212`
      `modifier && event.key.toLowerCase() === 's'` → `forceSaveRef.current()`
      / `forceSave`). Shares the same `onKeyDown` handler as the already-
      tested play shortcut (`web/e2e/cmd-enter-play.spec.ts`), but only the
      play branch is covered — a regression in the modifier/key check could
      break saving without any test noticing. Assert that pressing
      Meta/Ctrl+S triggers a save (e.g. via the GitHub mock, assert a `PUT`
      fires immediately rather than waiting for autosave's debounce).

- [x] **Playback actually plays** (`Preview.tsx`'s `playMeasureRef` /
      `playSelectedMeasures` in `usePoentaroWorker.ts`). Existing
      `cmd-enter-play.spec.ts` only asserts the no-op case (no measure
      selected). Add a case with a valid `selectedMeasureRange` and
      `soundfontReady`, and assert the "playing" state actually engages
      (e.g. the play/stop control's visible state flips, or
      `measureAudioPlaying` becomes true) rather than only checking the
      shortcut didn't crash.

- [x] **Undo/redo across file switches with unsaved edits** (`fileStore.ts`
      — only unit-tested today; `restore-file-github.spec.ts` etc. cover
      single-file backend round-trips but not this). Assert that editing
      file A, switching to file B without saving, switching back to A,
      still shows A's edit (or the correct discard/keep behavior per
      `fileStore.ts`'s actual contract) — this is where autosave-per-active-
      file bugs like the one fixed for shared-import (see
      `TODO-e2e-github-storage.md`'s import entry) tend to hide.

- [x] **Generate/Regenerate full-score audio** (`Preview.tsx:404`
      `onGenerateAudio` button, `usePoentaroWorker.ts:282` `generateFullAudio`,
      `:125` `setNextWavUrl`). Full synthesis pipeline parallel in risk to
      PDF export and per-measure playback but previously uncovered. Added
      `web/e2e/export-audio.spec.ts`, asserting:
      - Clicking "Generate audio" produces a `.preview-audio-player` with a
        `blob:` `src` and a decodable, non-zero `duration` (catches a
        silently empty/corrupt WAV).
      - The button's label flips to "Regenerate audio" once a `wavUrl`
        exists, and clicking it again actually replaces the blob `src`
        rather than reusing the old object URL (`setNextWavUrl`'s
        revoke-then-set logic).
      - Editing the source in place (no file switch) does not clear the
        existing player or reset the button back to "Generate audio",
        matching `usePoentaroWorker.ts`'s `[activeFile]`-only clear effect.

- [x] **SoundfontSearchModal fuzzy search, tag filters, and instrument
      preview** (`SoundfontSearchModal.tsx:24` `fuzzyScore`/
      `instrumentFuzzyScore`, `:246` `toggleTag`, `:259` the `filtered`
      AND-filter, `:274` `handlePlay`). Previously zero automated coverage
      on three independent pieces of business logic. Added
      `web/e2e/soundfont-search-modal.spec.ts`, asserting:
      - Typing a fuzzy query (`vln`) surfaces "40: Violin" via subsequence
        matching while excluding unrelated instruments.
      - Clicking an instrument's `#strings` tag narrows the list to the
        strings-category instruments (AND-filter) and un-clicking restores
        the full list.
      - Clicking the preview button flips its title to "Pause preview" and
        clicking again reverts it to "Preview instrument", guarding against
        a stuck-playing preview toggle.

- [x] **File-operation error path** (`useFileOperations.ts:27` `runFileOp`,
      wired to `ErrorModal.tsx`). The single shared error-handling wrapper
      behind all six structural file operations (create, duplicate, rename,
      delete, restore, import) had zero coverage of any kind — every
      `*-github.spec.ts` test only mocked successful `PUT`/`DELETE`
      responses. Added `web/e2e/file-op-error-github.spec.ts`, asserting for
      a failed create:
      - The "New" button's spinner appears while the create is in flight,
        then the error modal appears with the right title and a message
        containing the underlying failure text.
      - The spinner clears and the button reverts to "New" afterward
        (`finally`'s `setPending(false)` ran on the error path, not just
        success — otherwise the button would spin forever).
      - No phantom `untitled.poentaro` tab appears and the active tab is
        unchanged (`setStore` is never called on failure).
      - Closing the modal and retrying "New" succeeds normally, proving the
        user can actually recover from the failure.

## Not worth adding right now

- Individual per-operation GitHub error-path tests beyond the generic
  create-failure case above (500s, rate-limit banner) for
  rename/delete/duplicate — real gap, but lower priority since it's the same
  shared `runFileOp`/banner logic already exercised once
  (`errorBannerMessage`) already partially exercised; add only if a bug
  surfaces there.

## Cleanup

- [ ] Remove `web/e2e/diag.spec.ts` — it's a console-log debugging scratch
      file ("what does localStorage contain when share URL loads?"), not an
      assertion-driven test, and it overlaps `share.spec.ts` which already
      covers the real behavior.
