# Plan: force-save on tab switch (GitHub backend)

## Problem

Switching the active file tab does not flush a pending debounced GitHub
save. `shouldScheduleAutosave` (`web/src/hooks/useStorageBackend.ts:102-110`)
intentionally does not schedule a *new* save when the active file changes —
it assumes the previously active file was already saved. But if a debounce
timer is already pending from edits made just before the switch, nothing
currently flushes it early on tab switch: it only resolves via the
remaining `AUTOSAVE_DEBOUNCE_MS` (20s) window, or via one of the existing
flush triggers (`blur`, `visibilitychange`, Ctrl/Cmd+S, or
`switchBackend`). Tab switching is not one of those triggers today.

This is confirmed by the existing unit test
`web/src/hooks/useStorageBackend.test.ts:24-32`, which asserts that
switching the active file does *not* schedule an autosave — it says
nothing about flushing an *already-pending* one.

## Fix

1. In `useStorageBackend.ts`, add a new returned callback,
   `flushPendingSave`, that mirrors the pattern already used in
   `switchBackend` (lines 241-244):
   ```ts
   const flushPendingSave = useCallback(() => {
     if (backend.kind === 'github' && debouncedSave.isPending()) {
       debouncedSave.flush()
     }
   }, [backend, debouncedSave])
   ```
   Add it to `UseStorageBackendResult` and the hook's return object.

2. In `App.tsx`'s `handleSelect` (~line 190-195), call `flushPendingSave()`
   before `setStore((prev) => selectFile(prev, name))`, so any pending edit
   to the file being left is persisted before the active file changes.

3. Check whether `App.tsx`'s `handleSelect` or `useStorageBackend`'s return
   shape is documented in `ARCHITECTURE.md`. If so, update the doc in the
   same commit (per `CLAUDE.md`'s rule on entry-point/key-type changes).

4. Update the doc comment on `shouldScheduleAutosave` if its "already
   saved" assumption needs qualifying now that a separate flush path
   exists.

## e2e test

New file: `web/e2e/tab-switch-force-save-github.spec.ts`, modeled on
`web/e2e/autosave-github.spec.ts` (same `mockGithubContentsApi` + fake
clock setup):

1. Seed two files via the mock, e.g. `a.poentaro` and `b.poentaro`.
2. Open `a.poentaro`, type an edit, **do not** advance the fake clock.
3. Assert no PUT has fired yet (debounce still pending) and no
   `save-status-badge` is showing "Saved".
4. Click the tab for `b.poentaro`.
5. Assert a PUT for `a.poentaro`'s path fires immediately (poll the mock's
   captured PUT bodies) — without any `page.clock.fastForward` call. This
   is what proves the tab switch itself forced the flush, rather than the
   debounce timer happening to elapse.
6. Switch back to `a.poentaro` and reload the page; assert the edit survived
   in the (mocked) remote content, not just in-memory React state.

## Out of scope

- Local backend behavior (see the companion local-backend plan).
- GitHub 409 conflict handling on flush — assume happy path, matching the
  existing autosave-github test's scope.
