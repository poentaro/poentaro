# TODO: Replace the REST stack with anyx protocols

## Goal

The worker stops serving REST and OpenAPI. Every client talks to it through
an anyx protocol, each generated from its own schema:

| Schema | Endpoint | Credential | Who calls it |
|---|---|---|---|
| `poentaro public` | `POST /public` | none | anyone: sign up, sign in, open a link, origin handoff |
| `poentaro app` | `POST /app` | session token | the web app, signed in |
| `poentaro` (agent) | `POST /agent` (exists) | API key | AI agents |
| `poentaro test` | `POST /test` | none; served only when `e2e_reset_enabled` | e2e setup |

Done means: `handlers/routes.rs`, `api_error.rs`, `protocol.rs`, utoipa,
`export_openapi`, `openapi-typescript` and `web/src/generated/live-share-worker/`
are deleted, and nothing in `web/` builds a URL other than these endpoints.

## Decisions

- **Separate schemas per audience.** The agent schema stays as it is. A
  leaked API key can only edit scores: it can't manage keys, change the
  password, start a public link or trash files. Agent `write` refuses a score
  that doesn't parse; app `write` saves it, because the web app autosaves
  half-typed scores.
- **Credentials go in `Authorization: Bearer`, never in a method's
  arguments.** The worker checks them before calling `handle`; a bad one is
  HTTP 401 for the whole request.
- **The app addresses files by id; agents by path.** The outbox queues
  offline edits by id, and a file moved elsewhere (another device, an agent)
  while a tab is offline must not break its queued writes. App methods take
  `(id: TEXT)` and return the path for display; the agent schema keeps paths.
- **The app loads content lazily.** `files/list` in `poentaro app` returns
  summaries (id, path, revision, trashed at); content comes from `files/read _`
  when a file is opened. This is a behavior change, not just a port: see
  step 5.
- **Server-side naming.** `files/restore` picks a free name and
  `folders/move` computes the moves, in Rust. `cloudBackendNaming.ts`'s
  `uniqueName` and the client-built rename batch go away.
- **The app compiles scores itself.** `poentaro app` has no export and no
  diagnostics; `write`/`create` return only the revision.
- **An unknown share id opens as `ended`**, so a link never reveals whether
  an id exists.
- **`links/open _` is a POST.** Safe: every worker response is already
  `Cache-Control: no-store` (`routes.rs`), so nothing relies on GET caching.
- **e2e reset gets its own schema**, `poentaro test`, with one method
  `accounts/reset (emails: LIST: TEXT)`. It never appears in a guide an agent
  reads.
- **anyx gets `RETURNS NOTHING`** for methods with nothing to return
  (`account/sign out`, `links/stop file _`, `keys/revoke _`, `files/move _ to _`
  …). It generates `()` in Rust and `void` in TS.
- **`folders/move` is safe to retry.** REST's batch rename succeeded when
  repeated after its answer was lost; moving a folder already at its
  destination succeeds the same way, so the outbox can resend it.
- **The browser makes a new file's id, as today.** `files/create (id) at
  (path) content (content)`, so offline creates and the outbox work
  unchanged. An id clash is an error.
- **Offline, a file never opened on this device shows as unavailable.**
  Files opened before stay cached and editable offline. No background
  prefetch.
- **Old tabs reload.** Every generated client sends `Poentaro-Schema:
  <hash>`; on a mismatch the worker answers 409 and the app asks to reload.
  Schema changes don't need to stay backward compatible.
- **Generated code is built, not checked in**, for all four schemas,
  including the agent one: a build script runs the generator, and
  `src/generated.rs`, `tests/generated.rs` and `just generate-protocol` go.
- **One PR for the whole migration, REST deleted in a follow-up.** Tabs open
  on today's bundle call REST without a schema hash, so REST stays served
  until `app_log` shows no REST calls for a while; then a small PR deletes it.
- **anyx changes are committed straight to anyx's upstream `master`**, not
  through a PR; then poentaro repins.

## Steps

### 0. anyx (`~/repos/anyx`, commit and push straight to upstream `master`)

- [x] `RETURNS NOTHING` in the schema grammar (anyx `adf3324`) (`protocol.anyx`), the checker,
      the codec (a result of `RETURNS` with no value), the Rust/wasm/TS
      generators, `PROTOCOL.md` and the guide. Round-trip tests.
- [x] Repin anyx in poentaro.

### 1. Schemas and codegen (`crates/poentaro-protocol`)

- [x] Move `schema/` to `schemas/agent/` unchanged; add `schemas/public/`,
      `schemas/app/` and `schemas/test/`.
- [x] Revise the app draft: ids instead of paths in every method's slot,
      `files/list` returns summaries with id, `RETURNS NOTHING` where the
      draft returned filler.
- [x] A `build.rs` in `poentaro-protocol` generates one Rust module per
      schema into `OUT_DIR` (`agent`, `app`, `public`, `test`), included from
      `lib.rs`. Delete the checked-in `src/generated.rs`,
      `tests/generated.rs` and the `generate-protocol` recipe.
- [x] The wasm codec: `poentaro-wasm` exports `protocol-encode-request` and
      `protocol-decode-response`, taking the protocol (`app`, `public`,
      `test`) and calling `anyx_protocol::json` with the generated
      `schema()`. Not anyx's `wasm` output: that is `#[wasm_bindgen]`, and
      the web app's wasm is a WIT component.
- [x] `build:protocol` in `web/package.json` writes the TS clients into
      `web/src/generated/protocol/` (git-ignored) with the
      `poentaro-protocol-typescript` binary. `build:worker-types` runs it, so
      CI, lefthook and `predev`/`prebuild` get it unchanged; it becomes the
      only step when REST goes.
- [x] `SCHEMA_HASH`: `build.rs` hashes each schema's files (FNV-1a) into
      both the Rust module and the TS client, so server and client agree by
      construction.

### 2. Worker endpoints (`crates/live-share-worker`)

- [x] `POST /public`: no credential, rate limit by IP as sign-in does today,
      call `public::handle`.
- [x] `POST /app`: verify the session token (`auth::session`), 401 on
      failure with the same `VerificationFailure` logging as today, call
      `app::handle`.
- [x] `POST /test`: only when `e2e_reset_enabled`, otherwise no such route.
- [x] Keep `POST /agent` as it is.
- [x] Schema hash check: each generated client sends `Poentaro-Schema:
      <hash>`; on a mismatch the worker answers 409 and the web app shows
      "Poentaro was updated, reload". Missing header (agents, curl) is fine.
- [x] Log each call to `app_log` by its full method name, as `/agent` does.

### 3. Implement `poentaro public` and `poentaro test`

Each item: implement the `Server` trait method by calling the code the REST
handler calls today, add a Rust test, then delete the REST route.

- [x] `account/sign up _ password _` (was `/auth/sign-up`)
- [x] `account/sign in _ password _` (was `/auth/sign-in`), keeping the
      lockout and IP rate limit
- [x] `links/open _` (was `GET /shares/{id}`), including folder links and
      `too large`
- [x] `migrations/upload _` and `migrations/claim _` (were
      `/origin-migrations*`)
- [x] `accounts/reset _` in `poentaro test` (was `/e2e/reset`)

### 4. Implement `poentaro app`

- [x] `files/list` (summaries), `files/read _`
- [x] `files/write _ at revision _ content _`, with `conflict at revision`
      returning the current content
- [x] `files/create _ at _ content _`, with the browser's id (today's
      `generateFileId()`); an id clash is an error
- [x] `files/move _ to _` (was `/files/{id}/rename`)
- [x] `folders/move _ to _` (was `/files/rename-batch`), computing the moves
      in Rust; property-test it against the TS it replaces
- [x] `files/trash _`, `files/restore _` (server picks a free name)
- [x] `files/versions/list _`, `files/versions/read _ at revision _`
- [x] `links/start|stop|status of file _`, `links/start|stop|status of
      folder _`, `links/list` (were `/files/{id}/share*`, `/folders/share*`,
      `/shares/overview`)
- [x] `keys/list`, `keys/create`, `keys/revoke _` (were `/api-keys*`)
- [x] `account/sign out`, `account/change password from _ to _`

### 5. Move the web app over

One caller at a time; each stays green on its e2e features.

- [x] A `send` per endpoint in `web/src/protocol/`: sets the URL, bearer token
      and schema hash, turns 401 into the existing "session ended" flow and
      409 into the reload prompt.
- [ ] **Lazy loading.** Find everything that reads a file's content without
      opening it (search, duplicate, folder export, offline open, the
      revision check every 5 seconds) and make each fetch with `files/read _`
      or cache what was read. Offline, a file never opened here shows as
      unavailable.
- [ ] Purpose-built methods for what needs many files' content, each with a
      result size the server controls: a server-side search method with a
      limit, and `folders/export _`. Needs lazy loading.
- [ ] Drop the "read every file" batch (`cloudBackendListing.ts`) once lazy
      loading is in: loading is one `files/list`, then `files/read _` per
      opened file. The batch was a stopgap for that N+1.
- [ ] Repin anyx once it has call values and typed tuple batches (anyx
      `TODO.md`, "Protocol clients and batching"); move any mixed batch to
      the tuple form and drop `AppFileCalls.batch`'s callback form.
- [x] `storage/accountRequests.ts`: sign up, sign in, sign out, change
      password
- [ ] `hooks/useCloudStoreLoader.ts`, `storage/cloudBackend.ts`: list, read,
      create, write
- [x] `storage/cloudBackendOutbox.ts`, `storage/outbox/sendMessage.ts`: same
      ids, new transport
- [ ] `storage/cloudBackendFolders.ts`, `cloudBackendNaming.ts`: folder
      move, restore; delete the naming logic that moved to Rust
- [ ] `storage/cloudBackendVersions.ts`,
      `components/fileHistory/FileHistoryModal.tsx`: versions
- [ ] `hooks/useSyncedShareOwner.ts`, `hooks/useFolderShares.ts`: links
- [ ] `hooks/useSyncedShareViewer.ts`, `syncedShare/workerClient.ts`,
      `syncedShareUrl.ts`: `links/open _`
- [x] API keys dialog: `keys/*`
- [ ] `bridge/main.ts`, `originMigration/importOriginMigration.ts`:
      `migrations/*`
- [ ] `syncedShare/errors.ts`: map each method's error `kind` instead of
      `ApiError.code`

### 6. Tests

- [ ] e2e helpers (`web/e2e/cloudFileHelpers.ts`, `e2eAccounts.ts`,
      `global-setup.ts`) call the protocols instead of the typed REST
      client; reset through `poentaro test`.
- [ ] Step files that hit the worker directly: `account-auth`,
      `cloud-folder-move-batch`, `cloud-outbox-conflicts`,
      `cloud-outbox-structural`, `file-op-error-cloud`, `session-ended`.
- [ ] New feature: an old tab whose schema hash is stale is asked to reload.
- [ ] New feature: a file's content loads when it is opened, not before.
- [ ] `agent-protocol` steps unchanged and passing.

### 7. Delete REST (follow-up PR)

Steps 1–6 are one PR and keep REST served. This step is its own small PR,
once `app_log` shows no REST calls for a while.

- [ ] `handlers/routes.rs`, the REST handlers in `handlers/`, `api_error.rs`,
      `protocol.rs` (and `protocol/origin_migration.rs`), `share_id.rs`'s
      OpenAPI schema, `tests/export_openapi.rs`.
- [ ] `utoipa` from `Cargo.toml`; `openapi-typescript` and
      `web/src/generated/live-share-worker/` from `web/`.
- [ ] Update `ARCHITECTURE.md` (wire types, endpoints, glossary: "app
      protocol", "public protocol", "test protocol").
