# Synced Share: Rust + D1 migration, GitHub-required ownership

Context: `live-share-worker` is currently TypeScript + Workers KV, with
anonymous (no-login) share ownership via a device-secret-derived
`ownerToken`. Decided in discussion: rewrite the worker in Rust
(`workers-rs`), move storage from KV to D1, and make **GitHub sign-in a
hard requirement for the Synced Share feature** — there is no longer an
anonymous ownership path. Share ownership is always a verified GitHub user
id; the device-secret/`ownerToken` mechanism is retired entirely, not kept
as a fallback.

Diesel is **not** used anywhere in this plan — it cannot reach D1 (no driver
target exists for D1's binding-based access) without writing a full custom
Diesel backend, which is out of scope. SQLx is used only in the
raw-SQL + local-shadow-SQLite-check pattern discussed, not as a live-D1 driver.

No backward compatibility is required anywhere in this plan (confirmed): no
data migration from the current KV store, no protocol byte-compatibility
requirement. There is currently only one live share in production; it can
simply be recreated after cutover if lost. Share links will look different
(new format, see §1) and old links simply stop working — that's accepted.

## 0. Decisions (resolved)

- [x] **GitHub sign-in is required for Synced Share to work at all.** There
      is no anonymous/no-login path any more. Creating a share requires a
      verified GitHub identity; the device-secret `ownerToken` scheme
      (`getOrCreateDeviceSecret`, `deriveSyncedShareIdentity` in
      `web/src/syncedShareUrl.ts`) is deleted, not kept as a fallback.
- [x] **Ownership = verified GitHub user id, permanently, from creation.**
      A share is created already bound to the creator's verified GitHub id
      (via `GET /user`, using a token from the dedicated Synced Share sign-in
      connection — see below). There is no "unpinned" state to reason about,
      and no device-token/GitHub precedence question, since the device token
      no longer exists in this system.
- [x] **Credential conflicts are rejected outright, never guessed at.** A
      write whose verified GitHub id doesn't match the share's
      `owner_github_id`, or whose GitHub verification fails, is rejected.
      No fallback of any kind.
- [x] **Fail closed, with one retry first.** On a GitHub verification
      failure (expired/invalid token, non-2xx from GitHub, timeout), retry
      once; if the retry also fails, reject the write. The failure must be
      reported to the UI in a form the user can screenshot (explicit error
      message with failure reason and timestamp), not a silent failure or
      generic toast.
- [x] Cache **a hash of the GitHub OAuth token**, never the raw token, in
      the verification-cache table.
- [x] **Auth and storage are separate concerns — use separate GitHub
      connections, not the shared storage-backend token.** The existing
      `githubAuth.ts` connection is scoped for the storage backend's
      Contents API access (broad, e.g. `repo` write) and was opt-in for a
      niche feature; Synced Share is now mandatory for a core, everyone-uses-it
      feature, so forcing every Synced Share user through that broad-scope
      consent (and sending that high-privilege token on every sync write)
      would be a real over-ask and a larger leak blast-radius than
      necessary. Instead: a **dedicated, minimally-scoped "sign in with
      GitHub" connection** used only for identity verification (no more
      scope than `GET /user` needs), independent of whether the user ever
      connects the storage backend. Originally implemented by reusing the
      *same registered GitHub OAuth App* (`client_id`) as the storage
      backend, requesting a different, minimal `scope` and storing its token
      separately. **Revised**: this leaked the storage app's full `repo`
      grant onto the sign-in's consent/reauthorization screen, because
      GitHub grants OAuth App scope per app per user, not per request — a
      user who'd ever connected the storage backend saw "Full control of
      private repositories" as "existing access" on what was supposed to be
      an identity-only sign-in. Now uses a **second, separate registered
      GitHub OAuth App**, giving full isolation (independent client id,
      rate limits, revocation) as originally flagged as the fallback if this
      exact problem showed up.
- [x] **This new sign-in uses standard authorization-code + PKCE
      (redirect-based), not device flow** — decided over reusing the
      existing `@octokit/auth-oauth-device` pattern because sign-in is now
      mandatory/core rather than an opt-in power-user feature, and
      redirect-based "Sign in with GitHub" (one click, no manual code entry)
      is meaningfully lower-friction for something everyone must do. Use
      `oauth4webapi` for this — a small, spec-compliant, dependency-free
      OAuth2/OIDC client built on Fetch + Web Crypto (no Node-specific
      APIs), so the same library runs unmodified in the browser (to start
      the redirect + PKCE challenge) and in the Worker (to perform the
      token exchange, since Workers can hold the client secret as a secret
      binding — established earlier that a Worker is a real server for this
      purpose). Its docs explicitly cover GitHub as a plain-OAuth2
      (non-OIDC) example provider.
- [x] **No cleanup job for the verification-cache table.** Accept unbounded
      row growth at this project's scale; do not build TTL sweeping/cron
      cleanup.
- [x] **The create-share endpoint's abuse protection is GitHub verification
      itself** — since creating a share now requires a verified GitHub
      identity (not an open/anonymous call), no separate rate-limiting
      layer is needed for this reason alone.
- [x] **Testing**: mock the GitHub verification call in unit tests (inject a
      fake GitHub client); exercise the actual Worker/D1 integration via
      `wrangler dev`, not a from-scratch simulation of the Workers runtime.
- [x] **Reading a share stays fully anonymous.** GitHub is required only to
      create/own a share (write side). The unguessable `shareId` in the link
      remains the sole credential needed to view — no login for viewers,
      matching the feature's original point (a link you can send to anyone).
- [x] **No seamless OAuth-then-continue flow.** "Start sync" stays enabled
      even when GitHub isn't connected; clicking it while unconnected
      triggers the GitHub OAuth prompt, but does **not** automatically
      proceed to creating the share afterward — the user must click "start
      sync" again once connected. If the OAuth prompt is abandoned/expires,
      the UI just returns to the idle state (no share created, no dangling
      state).
- [x] **Errors are maximally verbose, with one hard exception.** On any
      failure (verification failure, retries exhausted, etc.) show the full
      error context — reason, timestamps, GitHub's raw status/error body,
      internal state — even if it looks like a stack trace, so it's directly
      useful when screenshotted for debugging. The one thing that must
      always be redacted regardless: the GitHub OAuth token and its stored
      hash never appear in any error surfaced to the UI.
- [x] **Error presentation**: a full-screen dialog/modal (not a toast or
      inline banner), containing the full verbose error context (per above)
      plus a message directing the user to screenshot it and file a GitHub
      issue, with a direct link:
      `https://github.com/jianpu-generator/jianpu-generator/issues/new`.
- [x] **Retry/backoff**: use an executor-agnostic backoff crate (e.g. the
      `backoff` crate's `future` module, which awaits a supplied future
      rather than owning its own tokio-based timer — required since Workers
      run on `wasm-bindgen-futures`, not tokio); 2 retries, exponential
      delay, capped around ~2s total wait before failing closed.

## Task breakdown (commit-sized, in dependency order)

Each row is meant to be one commit/PR. Scope follows this repo's commit
convention (`<type>(<scope>): <description>`); the scope shown is a
suggestion, pick the module doing the most work if a task ends up touching
more than expected. Items are cross-referenced back to the numbered
sections below (`§1`, `§4`, etc.) — do the detailed work there, check items
off in both places.

- [x] **1. Schema migrations** — `.sql` files for `users` /
      `user_identities` / `oauth_sessions` / `docs` (§1). Scope: `schema`.
      Depends on: nothing.
- [x] **2. Rust crate scaffold** — new `live-share-worker` crate,
      `wasm32-unknown-unknown` target, `worker` + `sqlx` deps, shadow-SQLite
      build step (§2). Scope: `live-share-worker`. Depends on: 1.
- [x] **3. `sqruff` pre-commit** — lint/format gate scoped to `queries/` and
      migration `.sql` files (§3). Scope: `lint`. Depends on: 1.
- [x] **4. Port worker logic to Rust** — `resolveRole`/`doc`/`index`/
      `protocol` against the new schema, with identity resolution stubbed
      (no real GitHub verification yet); update `ARCHITECTURE.md` in the
      **same commit** per this repo's docs rule (§4, §5). Scope:
      `live-share-worker`. Depends on: 2.
- [x] **5. Wrangler + CI wiring** — `wrangler.toml` D1 binding and
      `worker-build`, CI/pre-commit build of the new crate for
      `wasm32-unknown-unknown` (§4). Scope: `live-share-worker`. Depends
      on: 4. (`crates/live-share-worker/wrangler.toml`, `[[d1_databases]]`
      binding `DB` matching `D1_BINDING` in `src/handlers.rs`, placeholder
      `database_id = "REPLACE_AT_ROLLOUT"` filled in at task 12; `worker`
      name `jianpu-live-share-worker-rs`, distinct from the live TS
      worker's `jianpu-live`, not deployed yet. `worker-build` documented in
      `AGENTS.md`'s "Local tooling" section. Pre-commit:
      `live-share-worker-wasm-check` job in `lefthook.yml`'s fast parallel
      group, glob-scoped to `crates/live-share-worker/**`, running `cargo
      check --target wasm32-unknown-unknown -p live-share-worker` --
      full `worker-build` bundle verification left as future work for a CI
      workflow once deployment is wired, task 11/12.)
- [x] **6. Dedicated GitHub sign-in plumbing** — `IdentityProvider` trait +
      `GithubIdentityProvider`, PKCE module (`oauth4webapi`), Worker
      callback route + client-secret binding (§6). Scope:
      `synced-share-auth`. Depends on: 2. (`identity::github::GithubIdentityProvider`
      in `crates/live-share-worker/src/identity/github.rs`, not yet wired
      into any write path -- that's task 7. `POST /auth/github/callback` in
      `crates/live-share-worker/src/oauth.rs`, reading
      `SYNCED_SHARE_GITHUB_CLIENT_ID` (plain `[vars]`) and
      `SYNCED_SHARE_GITHUB_CLIENT_SECRET` (Worker secret binding, not
      committed) from `crates/live-share-worker/wrangler.toml`. Client-side
      PKCE-initiation module `web/src/storage/syncedShareGithubAuth.ts`,
      using `oauth4webapi`, distinct from `githubAuth.ts` -- starts the
      redirect only; completing the flow is task 8.)
- [x] **7. Verification + caching in the worker** — hashed-token lookup
      against `oauth_sessions`, backoff-wrapped GitHub call on miss/stale,
      create-on-first-sight `user_identities` row, wired into
      `resolveRole`-equivalent ownership check (§6). Scope:
      `synced-share-auth`. Depends on: 4, 6. (`identity::resolve_verified_user_id`
      in `crates/live-share-worker/src/identity.rs`: SHA-256 hashed-token
      lookup against `oauth_sessions` with a ~1hr TTL
      (`verification::session_is_fresh`), falling through to
      `identity::github::GithubIdentityProvider` (now the only
      `IdentityProvider` -- the insecure `StubIdentityProvider` from task 4
      is deleted) wrapped in `verification::retry_with_backoff` (2 retries,
      `backoff` crate's exponential delay, capped ~2s, D1-/worker-free and
      unit-tested with a fake operation + instant sleep in
      `tests/verification.rs`, per §0's testing decision). `handlers.rs`'s
      `create_share`/`post_share` now call this instead of the stub, and
      turn a `VerificationFailure` into a `401` JSON response carrying only
      `reason`/`failedAt`/`attempts` -- never the token or its hash.
      `ARCHITECTURE.md` updated in the same commit.)
- [x] **8. Client sign-in + owner UI** — popup OAuth flow, "Sync" popover
      when disconnected, "Synced as @username" identity chip, send the
      token on every write (mockup Screens 1–3). Scope: `synced-share-ui`.
      Depends on: 6. (`syncedShareGithubAuth.ts`'s `openSyncedShareGithubSignInPopup`
      drives the whole popup round trip; `SyncedShareGithubCallbackPage.tsx`
      -- rendered by `main.tsx` at `SYNCED_SHARE_GITHUB_REDIRECT_PATH` --
      completes the exchange and relays the result to the opener via
      `postMessage`. `oauth.rs`'s `POST /auth/github/callback` response now
      also carries a best-effort `login`. `useSyncedShareOwner.ts` wires
      this in: `startSync` stays a no-op returning `null` when disconnected
      rather than itself opening the popup; `SyncedShareButton.tsx` shows a
      "Sign in with GitHub" popover in that case and a "Synced as
      @username" chip once connected + synced. Every write
      (`SyncedUpdateRequest`/`SyncedStopRequest`) now carries `identityToken`
      alongside the still-live `ownerToken`, per `SyncedIdentityFields` in
      `syncedShare/protocol.ts`.)
- [x] **9. Client error dialog** — full-screen verbose failure dialog with
      the one hard redaction (token/hash), "file a GitHub issue" link
      (mockup Screen 4). Scope: `synced-share-ui`. Depends on: 7, 8.
      (`SyncedShareErrorDialog.tsx`: a full-screen Radix `Dialog` showing the
      worker's `401` `VerificationFailure` fields (reason/failedAt/attempts)
      verbosely, plus a "file a GitHub issue" link to
      `https://github.com/jianpu-generator/jianpu-generator/issues/new`;
      dismissing it returns to idle with no auto-retry.
      `web/src/syncedShare/errors.ts` builds the structured failure from a
      non-ok write response or a thrown network error in
      `useSyncedShareOwner.ts` (`syncFailure`/`dismissSyncFailure`), and
      `redactSecrets` there is the defensive backstop redacting any
      token-/hash-shaped text even though `VerificationFailure` has no such
      field by construction. No worker change was needed -- its existing
      `401` body already carried enough for this.)
- [x] **10. Viewer attribution + response scrubbing** — "Shared by
      @username" on `SyncedShareBanner` (mockup Screen 5), confirm the
      public doc response never leaks token/hash/internal ids (§6). Scope:
      `synced-share-ui`. Depends on: 7. (Audited `doc::to_public_doc` /
      `handlers::get_share` first: the response already never carried
      `owner_user_id`, `share_id`, timestamps, or any token/hash — this was
      already satisfied by task 7's work, not a leak to fix. What was
      missing was `login` itself not being threaded through at all: added
      `queries/get_owner_login.sql` + `db::get_owner_login` (best-effort
      `user_identities.login` lookup by `owner_user_id`, D1-free from
      `to_public_doc`'s own signature — the login is passed in as a plain
      `Option<String>` parameter so that function stays unit-testable),
      `protocol::SyncedDoc::owner_login` (new field, `None`/`null` when
      uncached — never any other internal id), and `get_share` now fetches
      it alongside the doc row. Threaded through the client:
      `web/src/syncedShare/protocol.ts`'s `SyncedDoc.ownerLogin` ->
      `useSyncedShareViewer.ts`'s `syncedShareViewerOwnerLogin` ->
      `useScoreSource.ts` -> `AppHeader.tsx` -> `SyncedShareBanner.tsx`,
      which renders "Shared by @{ownerLogin}" next to the synced filename
      when present. `ARCHITECTURE.md` updated (Synced Share worker section
      + glossary).)
- [x] **11. Retire the old path** — delete
      `getOrCreateDeviceSecret`/`deriveSyncedShareIdentity`/
      `SyncedShareIdentity` and the old TypeScript worker source; update
      Playwright scenarios for the new create-share flow, run via
      `pnpm test:e2e:resolve` ([[feedback_use_e2e_resolve_script]]) (§4,
      §6). Scope: `synced-share`. Depends on: 8, 9, 10 verified working.
      (Deleted `getOrCreateDeviceSecret`/`deriveSyncedShareIdentity`/
      `SyncedShareIdentity` from `web/src/syncedShareUrl.ts`, the `ownerToken`
      field from `web/src/syncedShare/protocol.ts`
      (`SyncedUpdateRequest`/`SyncedStopRequest` now carry only
      `identityToken`, required), and every remaining reference in
      `useSyncedShareOwner.ts`/its tests. `useSyncedShareOwner.ts` rewired
      around the new server-side create-share flow: `startSync` is now
      async, calling the worker's `POST /shares` (new `CreateShareRequest`/
      `CreateShareResponse` types) the first time a file syncs and
      persisting the returned `shareId` locally per file
      (`poentaro:synced-share-id:v1:<fileId>`, §1's "client persists the
      returned shareId" bullet), reusing it on later "Sync" clicks with no
      network call. `SyncedShareButton`/`AppHeader`'s `onStartSync` type
      updated to `Promise<string | null>` to match. Deleted the old
      top-level `live-share-worker/` TypeScript/KV source
      (`src/{doc,index,protocol,resolveRole}.ts`, `test/`, `package.json`,
      `wrangler.jsonc`, `tsconfig.json`) and its deploy workflow
      (`.github/workflows/live-share-worker.yml`) -- kept
      `live-share-worker/migrations/` in place, since that's the *new* D1
      schema's shared source of truth (`crates/live-share-worker/build.rs`
      depends on that exact path), not old-worker code. `ARCHITECTURE.md`
      updated in the same commit (old-worker/KV language removed from the
      Synced Share worker section, `ownerToken` references updated).
      Rust worker's GitHub token-exchange/`GET /user` endpoints made
      env-var-overridable (`SYNCED_SHARE_GITHUB_TOKEN_URL`/
      `SYNCED_SHARE_GITHUB_USER_URL`, unset in production) so e2e never
      makes a real GitHub call -- `web/e2e/mock-github-oauth-server.ts` is
      a tiny local mock of both endpoints, wired into
      `web/playwright.config.ts`'s webServer list alongside a `wrangler dev`
      run of `crates/live-share-worker` (replacing the old worker's
      webServer entry; migrations applied to local D1 first). The popup's
      navigation to GitHub's real authorization endpoint is still mocked
      via `context.route` (browser-level), per this repo's existing OAuth
      e2e pattern. Updated `synced-share-button.feature`
      (every scenario now signs the owner in via a pre-seeded Synced Share
      GitHub connection, exercising the real create-share flow) and added
      `synced-share-github-signin.feature` (the sign-in-prompt/popup
      OAuth/identity-chip flow and the full-screen verification-failure
      dialog). Also switched the viewer/late-viewer "opens the copied sync
      link" steps to isolated `browser.newContext()`s instead of
      `context.newPage()` -- sharing the owner's localStorage let the
      viewer page's own `useSyncedShareOwner` instance (rendered
      regardless of role) see the same "actively synced" flag and
      re-push the local file store's latest content on its own
      mount/reload, defeating the autosave-debounce scenario. All 15
      Synced Share scenarios pass via
      `pnpm test:e2e:resolve -- --grep Synced`.)
- [x] **12. Rollout** — `wrangler d1 create --location apac`, decide
      staging-vs-direct-to-prod cutover (§7). Scope: `live-share-worker`.
      Depends on: 11. (Real D1 database `jianpu-live-share` created in
      region APAC — `database_id = "2761bae0-e553-47e8-af04-6e4a12aab1fa"`,
      filled into `crates/live-share-worker/wrangler.toml`, migrations
      applied via `wrangler d1 migrations apply jianpu-live-share --remote`
      and verified (`users`/`user_identities`/`oauth_sessions`/`docs` tables
      all present). Cutover decision: direct-to-prod, no staging, per §0/§7
      and this task's explicit instruction. **Not done**: actually deploying
      the worker (`wrangler deploy`) — `SYNCED_SHARE_GITHUB_CLIENT_ID` in
      `wrangler.toml` is still the `REPLACE_AT_ROLLOUT` placeholder and the
      `SYNCED_SHARE_GITHUB_CLIENT_SECRET` Worker secret has never been set,
      so deploying now would ship a Worker whose GitHub sign-in is broken.
      This is a manual step left for the user — see §7 below for the exact
      remaining commands. **Update**: a CI/CD deploy pipeline now exists at
      `.github/workflows/live-share-worker.yml` (builds with `worker-build`,
      applies D1 migrations via `wrangler d1 migrations apply
      jianpu-live-share --remote`, then `wrangler deploy`, triggered on
      every push to `master` touching `crates/live-share-worker/**` or
      `live-share-worker/migrations/**`), so the manual one-off `wrangler
      deploy` in §7 below is no longer the way this ships — once the GitHub
      OAuth App secrets are set (the one remaining manual prerequisite), a
      normal push to `master` deploys automatically.)

Mockup reference for 8–10: `synced-share-screens` artifact (5 screens:
sign-in prompt, OAuth popup, synced/owned state, error dialog, viewer
banner).

## 1. Schema design and shareId generation

- [ ] **`shareId` keeps its current format/length/charset** (whatever
      `deriveSyncedShareIdentity` currently produces), but is now generated
      server-side at share-creation time (after GitHub verification)
      instead of derived offline from a device secret.
- [ ] **Collision handling: append exactly one extra character** from the
      same charset if the generated id collides with an existing row (checked
      against D1), rather than regenerating from scratch or looping through
      many attempts. (Collision is already astronomically unlikely; this is
      belt-and-suspenders, not a retry loop.)
- [ ] Client persists the returned `shareId` locally (keyed by file) so
      stopping/resuming sync on the same file reuses the same link.
- [x] **Schema is provider-agnostic (future-proof for more than GitHub)**,
      per discussion — ownership points at an internal user, not a raw
      GitHub id:
  ```sql
  CREATE TABLE users (
      id TEXT PRIMARY KEY,              -- unguessable random id, same style
                                         -- as share_id — not sequential
      created_at INTEGER NOT NULL
  );

  CREATE TABLE user_identities (
      provider TEXT NOT NULL,           -- 'github' today; future: others
      provider_user_id TEXT NOT NULL,   -- TEXT: not every provider's id is numeric
      user_id TEXT NOT NULL REFERENCES users(id),
      login TEXT,                       -- best-effort cached display name
      linked_at INTEGER NOT NULL,
      PRIMARY KEY (provider, provider_user_id)
  );
  -- index on user_id: reverse lookup, "this user's linked identities"

  CREATE TABLE oauth_sessions (
      token_hash TEXT PRIMARY KEY,      -- hash of the *Synced Share sign-in*
                                         -- token specifically (never the
                                         -- storage-backend token — see §0)
      provider TEXT NOT NULL,
      provider_user_id TEXT NOT NULL,
      verified_at INTEGER NOT NULL      -- ~1hr TTL checked in app code
  );

  CREATE TABLE docs (
      share_id TEXT PRIMARY KEY,
      owner_user_id TEXT NOT NULL REFERENCES users(id),
      filename TEXT NOT NULL,
      content TEXT NOT NULL,
      revision INTEGER NOT NULL DEFAULT 0,
      ended INTEGER NOT NULL DEFAULT 0,
      created_at INTEGER NOT NULL,
      updated_at INTEGER NOT NULL
  );
  ```
  Scope note: this is schema/verification future-proofing only — no
  account-linking UI, no merge-accounts tooling, no app-wide session concept
  is being built now. A `user_identities` row is created automatically the
  first time a given `(provider, provider_user_id)` is verified; the same
  person authenticating via a second provider later would get a second,
  separate `users` row until an actual linking feature is built (out of
  scope here).
- [ ] Open items carried over, still to decide during implementation:
      ~~whether `revision` still acts as an optimistic-concurrency guard on
      writes (it did in the old `StoredDoc`)~~ — **Resolved: no.** Decided
      in §8's "`revision` is not enforced as an optimistic-concurrency
      guard on writes" item: kept as last-write-wins, matching the old TS
      worker, since the design already assumes exactly one writer (the
      owner) per share; revisit only if a future feature needs real
      conflict detection. Still open: whether `login` is worth caching for
      "shared by @username" UI, and whether `owner_user_id` needs an index
      for a possible future "my shares" list.
- [x] Write the schema as plain `.sql` migration files (shared source for
      both real D1 via `wrangler d1 migrations apply` and the local shadow
      SQLite used for SQLx compile-time checks).

## 2. Set up Rust + SQLx + shadow SQLite (no Diesel)

- [x] Add `live-share-worker` as a new crate in the Rust workspace, target
      `wasm32-unknown-unknown`, depending on `worker` (Cloudflare's crate)
      and `sqlx` (SQLite feature only, for compile-time query checking).
      Lives at `crates/live-share-worker/` (see task 2's naming note: the
      top-level `live-share-worker/` dir still holds the old TS worker).
      `sqlx` ended up scoped to `[build-dependencies]` rather than a normal
      dependency — see the comment on that entry in
      `crates/live-share-worker/Cargo.toml` for why (its "sqlite" feature
      pulls in `libsqlite3-sys`, whose C build doesn't compile for
      `wasm32-unknown-unknown`, confirmed by trying it directly).
- [x] Set up local shadow `.sqlite` file, populated from the same migration
      files as §1, for `sqlx::query_file!` to check against at build time.
      (`crates/live-share-worker/shadow.sqlite`, gitignored; no
      `query_file!` calls exist yet since there are no real queries until
      task 4 — this only proves the DB gets created/migrated correctly.)
- [x] Write one `.sql` file per query under a `queries/` dir; each query
      read once via `query_file!` (compile-time check) and again via
      `include_str!` for the actual `D1Database::prepare()` call at
      runtime, so both share one source of truth. (`query_file!` itself
      isn't usable in this crate's wasm-targeted compile -- see the
      scaffold's own comment on this; landed instead as: `src/db.rs` reads
      each `queries/*.sql` file via `include_str!` for real
      `D1Database::prepare()` calls, and `tests/query_syntax.rs` --
      native-only, `sqlx` as a dev-dependency -- reads the same files to
      *prepare* (not execute) them against the shadow SQLite database as a
      substitute syntax check. See `ARCHITECTURE.md`'s "D1 query checking"
      section for what this does and doesn't catch.)
- [x] Wire up whatever build-order step (build script or documented dev
      command) ensures the shadow DB is migrated before `cargo build`/`cargo
      check` runs, so the compile-time query check has something to check
      against. (`crates/live-share-worker/build.rs`, using `sqlx` +
      `tokio` as build-dependencies, applies
      `live-share-worker/migrations/*.sql` in order via `sqlx::migrate`.)
- [ ] Add integration tests run against `wrangler dev` (per §0 testing
      decision) as a backstop for D1-vs-SQLite dialect differences the
      shadow-SQLite check can't catch.

## 3. Set up `sqruff` in pre-commit

- [x] Add `sqruff` (Rust-native SQL linter/formatter) as a pre-commit step
      in the lefthook config, scoped to the new `queries/`/migrations `.sql`
      files. Installed via `cargo install sqruff --locked --version 0.29.3`
      (documented in `AGENTS.md`) — pinned because 0.34.1 fails to compile
      from crates.io (a `PathBuf == str` type error in `sqruff-cli-lib`)
      and newer releases need a rustc ahead of this repo's; the lefthook
      job (`sqruff` in the parallel validation group) skips with a message
      instead of failing the commit if the binary isn't on `PATH`. Config
      lives in `.sqruff` at repo root, dialect `sqlite`, with `LT01`/`LT02`
      excluded and a comment explaining why (they fight this repo's
      intentional no-space-before-paren and column-aligned trailing
      comment conventions, not SQLite dialect issues). Runs `sqruff lint`
      only, not `fix` — `fix` reflows the aligned trailing comments in
      `0001_init.sql` in a way not worth auto-applying on every commit.
- [x] Spot-check `sqruff`'s SQLite dialect handling against the actual query
      files before relying on it as a gate (flagged earlier as newer/less
      battle-tested than `sqlfluff`). Ran `sqruff lint`/`fix` directly
      against `live-share-worker/migrations/0001_init.sql`: it parses
      standard SQLite `CREATE TABLE`/`CREATE INDEX` syntax with no
      dialect-related false positives; all findings were generic style
      rules (indent, spacing, line length), not dialect bugs. Shortened one
      over-80-char comment line in that file to satisfy `LT05`.
- [x] Confirm ordering: per project convention, keep this fast enough to
      run before/alongside the fast checks, not mixed into the slow e2e
      stage ([[feedback_lefthook_e2e_ordering]]). Added as `sqruff` in the
      existing `parallel: true` fast-validation group in `lefthook.yml`
      (alongside `dead-pub-check` etc.), scoped by `glob` to just the SQL
      paths so it's a no-op on unrelated commits — not mixed into the
      `e2e-tests` stage.

## 4. Migrate TS worker logic to Rust

- [x] Port `resolveRole.ts`, `doc.ts`, `index.ts`, `protocol.ts` to Rust,
      targeting the new `docs` table from §1 directly, and rewritten around
      GitHub-required ownership (no device-token code path to port at all).
      (`crates/live-share-worker/src/{resolve_role,doc,handlers,protocol}.rs`;
      identity resolution stubbed per §0/§6 -- `src/identity/stub.rs`, not
      real GitHub verification, which is task 6/7.)
- [x] Add the new "create share" endpoint from §1, gated on GitHub
      verification; update `web/src/syncedShareUrl.ts` (device-secret logic
      deleted per §0), `useSyncedShareOwner.ts`, `useSyncedShareViewer.ts`,
      and `web/src/syncedShare/protocol.ts` for the new flow. No
      byte-compatibility constraint with the old protocol (per §0).
      (`POST /shares` added in `crates/live-share-worker/src/handlers.rs`,
      gated on the *stub* identity resolver for now -- real GitHub
      verification is task 6/7. The `web/` changes in this bullet --
      `syncedShareUrl.ts`, `useSyncedShareOwner.ts`,
      `useSyncedShareViewer.ts`, `web/src/syncedShare/protocol.ts` -- are
      explicitly out of scope for task 4, tasks 8-11.)
- [x] Update `wrangler.toml`: build command pointing at `worker-build`
      (or equivalent), D1 binding instead of the KV namespace binding.
      (`crates/live-share-worker/wrangler.toml`, not the old
      `live-share-worker/wrangler.jsonc` — this is a new file for the Rust
      crate, not deployed yet.)
- [x] Update CI/pre-commit to build the new crate for `wasm32-unknown-unknown`
      as part of the existing `cargo build`/`cargo test` gate. (Landed as a
      dedicated `live-share-worker-wasm-check` job in `lefthook.yml` rather
      than folded into `cargo-checks`, since that job's workspace-wide
      `cargo clippy`/`cargo test` build for the host target, not
      `wasm32-unknown-unknown` — see task 5 above.)
- [x] Run the existing Playwright e2e suite against the new worker via
      `pnpm test:e2e:resolve` ([[feedback_use_e2e_resolve_script]]), updating
      scenarios for the new GitHub-required create-share flow. (Task 11.)
- [x] Remove the old TypeScript worker source once parity is verified.
      (Task 11 — `live-share-worker/migrations/` kept, as the new D1
      schema's shared source of truth.)

## 5. Update project docs (required by this repo's rules, same commit as §4)

- [x] Update `ARCHITECTURE.md`: new entry point/module paths for the
      Rust worker, any key types added/removed/renamed (e.g. `StoredDoc`
      → whatever the Rust struct is named), the KV→D1 terminology change,
      the GitHub-required create-share flow, and the retirement of the
      device-secret/anonymous-ownership concept, in the domain glossary as
      needed.

## 6. Implement GitHub-required ownership

- [x] Build the new dedicated, minimally-scoped "sign in with GitHub"
      connection for Synced Share identity (per §0), using `oauth4webapi`
      for authorization-code + PKCE (redirect-based) — a new module
      distinct from `githubAuth.ts`'s device-flow code, storing its own
      token independently from the storage-backend token. Add a Worker
      route for the redirect callback + token exchange (holding the client
      secret as a Worker secret binding). Define the `IdentityProvider`
      trait discussed, with one `GithubIdentityProvider` implementation for
      now. (Client-side redirect+PKCE start only, in
      `web/src/storage/syncedShareGithubAuth.ts` -- `oauth4webapi` is a
      browser/Fetch-API JS library with no wasm32 Rust equivalent, so the
      Worker-side token exchange in `crates/live-share-worker/src/oauth.rs`
      makes the equivalent plain HTTP call itself instead, per that file's
      doc comment. `IdentityProvider` trait already existed from task 4;
      this task adds `identity::github::GithubIdentityProvider` as its real
      implementation, not yet wired into any write path.)
- [x] Client (`useSyncedShareOwner.ts`): "start sync" stays enabled always;
      if this Synced Share GitHub connection isn't present, clicking it
      starts the redirect + PKCE sign-in flow but does not auto-continue
      afterward (per §0 — user clicks "start sync" again once connected).
      Send this connection's token with every write. No device-token code
      path exists any more.
- [x] Worker: on every write, check `oauth_sessions` for a fresh verified
      identity (hashed-token lookup); on miss/stale, call the
      `IdentityProvider` (GitHub's `GET /user` today) to resolve
      `(provider, provider_user_id)`, refresh the cache, and look up (or
      create-on-first-sight) the matching `user_identities` row to get
      `user_id`. On failure, retry via the backoff policy (§0), then reject
      (fail closed) with a specific, UI-surfaced error. Never trust a
      client-asserted identity directly.
- [x] `resolveRole`-equivalent logic: owner = resolved `user_id` matches
      `docs.owner_user_id` exactly. Any mismatch or verification failure
      (after retries) is rejected outright. No other cases exist.
- [x] Ensure the public doc response (`toPublicDoc`-equivalent) never leaks
      the raw token, its hash, or internal `user_id`/`user_identities` rows
      beyond what the UI actually needs (e.g. a cached `login` for display).
      (Already satisfied as of task 7 for token/hash/`owner_user_id`/
      `share_id`/timestamps — `doc::to_public_doc` never took or emitted
      any of those. Task 10 added the one deliberate exception this bullet
      calls out: `owner_login`, sourced from `db::get_owner_login`
      (`user_identities.login` only, never `user_id` or any other column)
      and passed into `to_public_doc` as a plain value, not a row/struct —
      see task 10's note above.)
- [x] Delete `getOrCreateDeviceSecret`/`deriveSyncedShareIdentity` and the
      `SyncedShareIdentity` type from `web/src/syncedShareUrl.ts` — no
      longer used anywhere. (Task 11.)

## 7. Rollout

- [x] Create the D1 database with `wrangler d1 create --location apac` —
      the nearest available location hint to Kuala Lumpur (D1 location
      hints are broad regions — `wnam`/`enam`/`sam`/`weur`/`eeur`/`apac`/`oc`
      — not city-level, so `apac` is the closest available choice, not a
      guarantee of KL-adjacency). Note the latency tradeoff for
      geographically distant users in the rollout notes, not as a launch
      blocker. (Ran `npx wrangler d1 create jianpu-live-share --location
      apac` — confirmed authenticated against a real Cloudflare account
      first via `npx wrangler whoami`, and `npx wrangler d1 list --json`
      showed no pre-existing databases, so this was a genuinely new
      resource, not a duplicate. Created `jianpu-live-share`,
      `database_id = "2761bae0-e553-47e8-af04-6e4a12aab1fa"`, region APAC
      — `wrangler d1 execute ... "SELECT name FROM sqlite_master..."`
      afterward reported `served_by_region: "APAC"`, `served_by_colo:
      "HKG"` (Hong Kong), i.e. genuinely APAC-local, not KL-local — the
      latency tradeoff this bullet calls out is real for e.g. European/
      American users but accepted, not a blocker, given this project's
      actual user base. `database_id` filled into
      `crates/live-share-worker/wrangler.toml`, replacing the
      `REPLACE_AT_ROLLOUT` placeholder. Migrations applied with `wrangler
      d1 migrations apply jianpu-live-share --remote` from
      `crates/live-share-worker/` and verified present:
      `users`/`user_identities`/`oauth_sessions`/`docs`.)
- [x] Decide staging vs. direct-to-prod cutover for the worker rewrite.
      (Direct-to-prod, no staging environment — decided by the user
      explicitly for this task.)
- [x] No rollback/compat plan needed (per §0) — direct cutover, old worker
      retired once the new one is verified. Existing share link(s) will stop
      working and can be recreated. (Old worker source already deleted in
      task 11; its live Cloudflare deployment, if still running under the
      name `jianpu-live`, was deliberately left untouched by this task —
      decommissioning it is a separate, explicitly-destructive step not
      taken unprompted, see the note below.)
- [ ] **Deploy the new worker to production** — NOT done in this task; left
      as a manual step, though now largely automated (see below). Blocked
      on one thing this task deliberately did not do, since it involves
      creating a real GitHub OAuth App:
      1. Create (or reuse, per §0) the dedicated Synced Share GitHub OAuth
         App, then fill its client id into
         `crates/live-share-worker/wrangler.toml`'s
         `SYNCED_SHARE_GITHUB_CLIENT_ID` (currently still
         `REPLACE_AT_ROLLOUT`) and set the secret:
         ```sh
         cd crates/live-share-worker
         npx wrangler secret put SYNCED_SHARE_GITHUB_CLIENT_SECRET
         ```
      2. **Update**: deploying the worker itself no longer needs a manual
         `wrangler deploy` — `.github/workflows/live-share-worker.yml` now
         does this automatically on every push to `master` that touches
         `crates/live-share-worker/**` or `live-share-worker/migrations/**`
         (installs Rust + `wasm32-unknown-unknown`, `cargo install
         worker-build`, builds, runs `wrangler d1 migrations apply
         jianpu-live-share --remote` (idempotent), then `wrangler deploy`,
         authenticating with the same `CLOUDFLARE_API_TOKEN`/
         `CLOUDFLARE_ACCOUNT_ID` repo secrets `oauth-proxy.yml` already
         uses). It can also be run on demand via `workflow_dispatch`. This
         deploys under the name in `wrangler.toml`
         (`jianpu-live-share-worker-rs`), a **distinct** Cloudflare Worker
         from the old, already-retired-in-source `jianpu-live` — deploying
         it will not touch/overwrite/replace the old worker's still-running
         Cloudflare-side deployment (if any). Confirm the deployed name in
         the Cloudflare dashboard or `npx wrangler deployments list` after
         it runs. The pipeline will still ship a Worker whose GitHub
         sign-in is broken until step 1 above is done once, by hand — that
         remains the one manual prerequisite.
      3. Point the web app at the new worker's URL (wherever its base URL
         is currently configured for Synced Share) and verify a real
         create-share round trip end-to-end before calling the cutover
         complete.
      4. **Follow-up, not done here**: once the new worker is verified
         live, the OLD worker's Cloudflare-side deployment (`jianpu-live`)
         is still running (only its source was deleted, task 11) and should
         eventually be decommissioned (`npx wrangler delete jianpu-live` or
         via the dashboard) — flagged here as a follow-up, deliberately
         **not** done as part of this task since deleting a real, currently
         serving production Worker needs explicit user confirmation.

## 8. Gaps found in post-implementation review (not yet done)

Found by re-reading the finished implementation end-to-end after task 12,
looking specifically for what would stop this from actually working once
deployed. None of these were done as part of any numbered task above —
listed here as their own follow-up items.

- [x] **`VITE_SYNCED_SHARE_HOST` still points at the OLD worker — nothing
      routes traffic to the new Rust worker yet.** This is the sharper,
      concrete version of bullet 3 above. Confirmed by tracing every call
      site: `web/src/hooks/useSyncedShareOwner.ts`,
      `web/src/hooks/useSyncedShareViewer.ts`, and
      `web/src/storage/syncedShareGithubAuth.ts` (via
      `SyncedShareGithubCallbackPage.tsx`) all build their request URLs from
      `import.meta.env.VITE_SYNCED_SHARE_HOST`. That var is currently set
      to `jianpu-live.hou32hou.workers.dev` — the old TS/KV worker's host —
      in three places: `web/.env` (default), `web/.env.local` (gitignored
      local override, currently duplicating the same value), and
      `.github/workflows/pages.yml` (lines 67 and 126, the CI/prod Pages
      build). The new Rust worker deploys under a deliberately different
      name, `jianpu-live-share-worker-rs` (see its own `wrangler.toml`
      comment — chosen on purpose so deploying it wouldn't silently replace
      the old worker). **This naming question was never actually decided**:
      either (a) rename/repoint the Rust worker's Cloudflare deployment to
      reuse the `jianpu-live` name so the existing env var keeps working, or
      (b) keep `jianpu-live-share-worker-rs` and update `VITE_SYNCED_SHARE_HOST`
      in all three locations above to its real `*.workers.dev` hostname (or
      a custom domain, if one gets set up) once it's deployed. Whichever is
      chosen, do it together with the OAuth App step above and verify a
      real create-share round trip against the *new* worker before calling
      cutover done — right now a prod deploy of the new worker would go
      live with zero web traffic ever reaching it.
      (**Resolved: (b)**, decided explicitly by the user — chosen because it
      keeps the old worker's live deployment completely untouched, matching
      the already-stated intent in §7 bullet 4 that decommissioning it stay
      a separate, explicit step rather than an incidental side effect of a
      routine CI deploy. `VITE_SYNCED_SHARE_HOST` updated in all three
      locations to `jianpu-live-share-worker-rs.hou32hou.workers.dev`
      (`.hou32hou` account subdomain matched from the old worker's host,
      `*.workers.dev` default domain — no custom domain set up).
      `crates/live-share-worker/wrangler.toml`'s comment updated to reflect
      the resolved decision instead of describing it as still-undecided.
      This is config/wiring only — a real create-share round trip against
      the new worker is still blocked on the open OAuth App prerequisite in
      §7 (`SYNCED_SHARE_GITHUB_CLIENT_ID`/`SECRET` are still
      `REPLACE_AT_ROLLOUT`/unset), so end-to-end verification was
      deliberately not attempted here and remains open.)
- [ ] **Consequence of the above, worth naming explicitly**: the Synced
      Share feature currently *live* in production is still the old TS/KV
      worker, whose source was already deleted in task 11 and whose old
      deploy pipeline no longer exists (`.github/workflows/live-share-worker.yml`
      now builds/deploys the *new* Rust worker instead). So the currently-
      serving production worker is source-orphaned — if it broke or needed
      a fix before cutover, there is no way to rebuild/redeploy it. Not a
      regression introduced by any single task, just worth knowing: the
      cutover (cache the new worker's URL + decommission the old one, per
      bullet 4 above) should happen promptly once started, not left
      half-done.
- [x] **`shareId` collision fallback can mint an id the client can't parse
      back out of a link.** Fixed by dropping the length-extension
      fallback: `share_id.rs`'s collision handling (now the generic,
      D1-free `generate_unique_id`, unit-tested in `tests/share_id.rs`)
      re-rolls a brand new candidate at the same `SHARE_ID_LENGTH` (11) on
      each collision instead of appending a character, so every `share_id`
      this crate mints stays exactly 11 characters and always matches
      `web/src/syncedShareUrl.ts`'s fixed-length `SHARE_ID_PATTERN`. Kept a
      true loop (not a fixed attempt count) rather than looping/regenerating
      with a cap, since a second collision is still vanishingly unlikely.
- [x] **`revision` is not enforced as an optimistic-concurrency guard on
      writes** — a client-sent `revision` is stored as-is, never compared
      against the existing row's `revision` before being overwritten.
      Confirmed this matches the OLD TS worker's behavior too (checked
      `live-share-worker/src/doc.ts` in git history as of commit
      `860e756`), so this is not a regression from the migration — carried
      over from the original design, which assumes exactly one writer (the
      owner) per share.
      (**Resolved: leave unenforced**, decided explicitly by the user —
      no feature currently needs real conflict detection, and enforcing it
      now would add rejection-handling complexity to `doc::apply_write`
      and `handlers.rs`'s write dispatch with no client depending on it.
      §1's "open items" bullet updated to point back here instead of
      flagging this as still-undecided. Revisit if a future feature, e.g.
      multi-device sync from the same owner, needs real conflict
      detection.)
- [x] **Redirect URI dropped the GitHub Pages base path.** Found while
      preparing to register `crates/live-share-worker`'s OAuth App callback
      URL: GitHub Pages (the currently-primary deployment, at
      `https://jianpu-generator.github.io/jianpu-generator/`) serves this
      app under a `/jianpu-generator/` subpath (`VITE_BASE_PATH` in
      `.github/workflows/pages.yml`'s `build` job), but
      `syncedShareGithubAuth.ts`'s `buildSyncedShareGithubAuthorizationUrl`
      built the redirect URI from `window.location.origin` alone, dropping
      that subpath -- and `main.tsx`'s pathname check had the same blind
      spot. Cloudflare Pages (served from the domain root) was unaffected,
      which is why this wasn't caught by the e2e suite (`wrangler
      dev`/Playwright serve the app from `/`, matching neither production
      GitHub Pages base path nor exposing the bug). There is no
      SPA-fallback `404.html` in this repo, so on GitHub Pages this was a
      genuine dead-end 404, not a client-side-recoverable routing quirk.
      Fixed by adding `syncedShareGithubCallbackPathname()` (joins
      `import.meta.env.BASE_URL` with `SYNCED_SHARE_GITHUB_REDIRECT_PATH`),
      used by both the redirect-URI builder and `main.tsx`'s check; unit
      tests in `syncedShareGithubAuth.test.ts` cover both base-path shapes.
- [ ] **§2's "integration tests against `wrangler dev`" item is still
      undone.** Confirmed by searching `crates/live-share-worker` for any
      such test file — none exists. This was called out in §2 as the
      backstop for D1-vs-SQLite dialect differences the shadow-SQLite
      compile-time check can't catch (real D1's SQLite dialect vs. the
      shadow SQLite used by `sqlx::migrate`/`query_file!`-adjacent checks in
      `build.rs`/`tests/query_syntax.rs`). Not exercised anywhere else
      either — task 11's Playwright e2e suite runs against `wrangler dev`
      too, but only through the app's HTTP surface, not as a dedicated
      integration-test target for this specific dialect-drift concern.
