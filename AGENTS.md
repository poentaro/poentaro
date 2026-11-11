## Guiding principles

- Cut taglines, helper text and secondary labels. If a screen needs them to make sense, fix the layout.
- Make the best decision that will elevate all three: the user experience (UX), developer experience (DX), and agent experience (AX) always, without breaking anything.

## Branching and PRs

**Never commit directly to `master`.** Always work on a feature branch and open a pull request. The full e2e suite is too large to run in the pre-commit hook, so it runs in CI (`.github/workflows/ci.yml`) on the PR instead — the PR is the gate, not the local commit.

## Committing

Commit message titles must always include a scope: `<type>(<scope>): <description>`, e.g. `feat(sequence): allow entries to omit parts for a single occurrence`. The scope identifies **where** the change lives (the module/directory/area touched, e.g. `sequence`, `grid-layout`, `midi`, `editor`), never **what** the change does or which feature it adds — that belongs in the description. When a change spans multiple modules in service of one feature, pick the scope by where the change is centered (the module doing the most work, or the one a reader would look in first), not by naming the feature itself. Never omit the scope, even for small or single-file changes. Don't restate the scope word in the description; the scope already says it.

Do not manually run `cargo build`/`cargo test` as a pre-commit check — the pre-commit hook already runs them, acting like a CI gate. It's fine to run tests during development to verify your own fix, but don't re-run the full suite right before `git commit` just to double-check.

The pre-commit hook runs cargo checks and the web typecheck, which can take a few minutes — longer than the default 2-minute Bash tool timeout. A plain `git commit` will hit that timeout even though the hook itself is still running to completion in the background. Always pass an explicit `timeout` of at least 300000ms (5 minutes), and prefer 480000ms (8 minutes) to be safe, on the `git commit` Bash call to avoid wasting a turn on a false timeout.

## Running e2e tests

When you run Playwright e2e tests yourself locally, run only the specific tests you need, with plain Playwright: `cd web && pnpm test:e2e --grep "..."`. Do **not** use the auto-retry script `pnpm test:e2e:resolve` locally — it exists for CI (`web/scripts/resolve-e2e-flakes.ts`), where the full suite runs and its known flaky tests are re-run until the failing set settles. A scoped local run is small enough to read directly, and a failure there should be investigated, not retried away. Stop any dev server you started once the run finishes.

## Syntax documentation

The `.poentaro` input syntax is documented in `syntax.md`.

- When a commit introduces or changes user-facing `.poentaro` syntax, **MUST update `syntax.md`** in the same commit.
- When a commit introduces or changes user-facing `.poentaro` syntax, **MUST update the `demo/` folder** in the same commit (add/update the example measure demonstrating the feature in the relevant `demo/NN-*.poentaro` file). Each file in `demo/` is a complete, standalone-valid `.poentaro` document (its own `# metadata`/`# parts`/`# score`) — the web editor opens them individually as a folder of demo files, so a fragment missing its own header would fail to render there even though it's never concatenated with the others.
- Syntax-affecting code lives under `src/parser/` and `src/desugar.rs`.

## Architecture documentation

`ARCHITECTURE.md` explains how the system fits together and defines its domain terms. It isn't a code reference: `trail skeleton` and `trail callers` answer those questions.

Update it in the same commit only when:

- a layer is added, removed or reordered, or the data flow between layers changes
- a domain term is introduced or redefined (glossary entries are at most two sentences)

Implementation detail, and the reasoning behind it, goes in doc comments next to the code. Don't add signatures, field lists or call chains.

## Coding style

Prefer functional programming style, unless the iterative version is simpler, shorter and easier to understand:
- **TypeScript**: use the `remeda` library (`import * as R from 'remeda'`)
- **Rust**: use the `itertools` crate

Avoid using uncommon abbreviations when naming (e.g. `TimedRdParser` for "recursive descent" — spell it out as `TimedRecursiveDescentParser` instead). Widely understood abbreviations (e.g. `Ast`, `Id`, `Http`) are fine.

Test cases should not be inlined with the source code, they should live in separate files.

Never use tuple in new data structures, always use struct instead.

## Node scripts

Node scripts (`web/scripts/`, e2e mock/static servers, Playwright reporters, anything run with `node`) are TypeScript (`.ts`) run directly with `node` (native type stripping — no `tsx`/`ts-node`, so stick to erasable syntax and explicit `.ts` import extensions), never untyped `.mjs`/`.js`, and must be covered by `tsc -b` (the pre-commit `web-typecheck` job) — add any new script's path to the matching `web/tsconfig.*.json` `include`. Parse CLI arguments with `commander`, not hand-rolled `process.argv` slicing. The only exception is config a tool requires in a fixed JS format (e.g. `web/.pnpmfile.cjs`).

## Web frontend must stay dumb

Keep the TypeScript web frontend as dumb as possible; put logic in Rust (compiled to wasm for the worker). The web layer passes raw UI state (e.g. hidden/soloed sets) to Rust and renders what comes back. Don't derive effective state (solo resolution, track filtering) in TS hooks.

## UI components

Prefer Radix UI primitives over DIY implementations for interactive controls (sliders, selects, dialogs, checkboxes, tooltips, etc.). Available packages: `@radix-ui/react-dialog`, `@radix-ui/react-select`, `@radix-ui/react-slider`, `@radix-ui/react-tooltip`, `@radix-ui/react-progress`. Install additional Radix packages as needed rather than rolling custom components.

Reuse the shared components in `web/src/components/` before building a new control. That folder is flat (shared primitives sit next to feature components), so these are the ones to know:

- **`ResponsiveMenu`**: every dropdown or menu, including pickers like the notation view. It renders a Radix `DropdownMenu` on desktop and a full-screen `vaul` drawer on mobile, so mobile is handled for you. Style the trigger with `.preview-export-btn` and the content with `.export-menu-list` / `.export-menu-item` (`web/src/preview-toolbar.css`). See `ExportMenuButton` and `NotationToggle` for examples. Don't use a native `<select>` or a bare Radix `DropdownMenu` for these.
- **`ExportMenuButton`**: a button that opens a sectioned menu of actions (built on `ResponsiveMenu`).
- **`RadixSelectItem`** (`RadixTableSelect.tsx`): items for a Radix `Select` inside a table cell.
- **`PoentaroDiffEditor`** / **`DiffLayoutToggle`**: a Monaco diff of two `.poentaro` texts (read-only, or with an editable right side) and its inline / side-by-side switch, whose choice `useDiffLayout` remembers. Used by the History modal.
- **`EditorToolbarButton`**: an icon button with a tooltip for the editor toolbar.
- **`SpinnerLabel`**: a button label with the shared pending spinner.
- **`accountDialogParts.tsx`**: `AccountDialogFrame`, `Field`, `ErrorBanner` and friends for account modals.
- **`TraceId`**: the worker's trace ID for a failed request, with a copy button. Show it beside every request error (`ErrorBanner` takes a `traceId`).
- **Icons**: `@radix-ui/react-icons` first; app-specific icons live in `components/icons/`.
- **Mobile branching**: in TypeScript, `useMediaQuery(MOBILE_BREAKPOINT_QUERY)` (`web/src/hooks/useMediaQuery.ts`) rather than a hard-coded width; in CSS, `@media (max-width: 768px)`.

When you add a new reusable primitive, add it to this list.

## Browser storage

`localStorage`/`sessionStorage` in the web app is only for:
- **transient relays** (e.g. the GitHub sign-in popup's PKCE verifier and auth-result handoff),
- **local files** (the `local` storage backend's file store),
- **per-device preferences** (e.g. storage backend choice, part toggles, the sign-in token),
- **the cloud outbox**: unsynced edits awaiting delivery to the database, held in IndexedDB on this device only until the server acknowledges them. It is never read as a source of truth for state that must agree across devices, and entries are deleted on acknowledgement.

Anything else, especially state that must agree across devices (e.g. whether a file is shared, which share belongs to which file), lives in the database and is derived from it, never mirrored into browser storage.

## Cross-boundary invariants (Rust ↔ TS)

A cross-boundary invariant is any value, shape or rule that two sides must agree on. The two sides can be Rust and TS, or either of them and config/scripts. Examples:

- a string tag
- a field name
- an index space
- an id format
- a numeric limit
- a URL path or status code
- a SQL column
- an env var name
- a label format like `"N: Name"`

It applies on every channel:

- wasm/WIT exports and their jco `.d.ts`
- serde JSON and worker `postMessage` messages
- the live-share worker's HTTP API
- the D1 schema, including test scripts that query it
- deploy/build config (`cloudflare.config.ts`, workflows, `.env`)
- Pages Functions
- e2e selectors and mocks

**Never keep such an agreement by hand.** Drift must be a compile, typecheck or build error, or impossible by construction. To get there, use one of these:

- **Generated types:** the WIT/jco `.d.ts`, or the OpenAPI `schema.ts` from `handlers::routes()`. Use a WIT `variant`/`enum`/`record` rather than a bare `string`/`u32`/`list<u8>` whose meaning TS must know.
- **One shared data file** that both sides read at build time, e.g. `fonts/fonts.json`.
- **Logic on one side only.** If TS needs something Rust already computes (a part's name after hidden parts are filtered, a formatted label, which font measures which text, a section kind), make Rust return it. Don't re-derive it in TS.

These do **not** count as a fix. They narrow the hand-matching instead of removing it:

- "must match X" / "mirrors Y" comments
- a TS constant or `satisfies Record<…>` restating a Rust value
- a hand-typed interface or cast on a raw `fetch`/`.json()`
- a regex re-implementing a Rust lexer rule
- positional parameters whose meaning the caller must know
- a test asserting that two hand-written copies agree

Rules:

- Never add a new instance.
- Remove any existing instance you touch rather than adding another occurrence.
- If none of the options above fits (e.g. values fixed by an external deploy platform), stop and ask the user before settling for less.
- `TODO-cross-boundary-invariants.md` tracks the known instances and how each was fixed.

## Local tooling

The whole toolchain is pinned in `mise.toml` and shared with CI
(`jdx/mise-action`). After cloning, run `mise install` and then
`lefthook install` (activate mise in your shell, or prefix commands with
`mise exec --`). To change a tool version, edit `mise.toml` — CI picks it up.

It provides Rust 1.89.0 (with `rustfmt`, `clippy` and the
`wasm32-unknown-unknown` target), Node, pnpm, `cargo-component`,
`worker-build` (compiles `crates/live-share-worker` to the wasm bundle `cf`
expects), `sqruff` (SQL linter used by the pre-commit `sqruff` job — see
`.sqruff`), `lefthook`, `prek`, `just` and `jq`. `sqruff` is pinned to 0.29.3
because 0.34.1 fails to compile from crates.io (a `PathBuf == str` type error
in `sqruff-cli-lib`) and newer releases require a rustc ahead of this repo's.

## Generating an SVG

```sh
cargo run -- generate svg simple.poentaro
```

## Running the app locally

Start the dev servers with `just dev` from the repo root: it runs the web app (Vite, `localhost:5173`) together with the live-share worker (`cf dev`, `localhost:8787`, with the local D1 migrations applied), and logs both to `dev.log`. Don't run `pnpm dev` in `web/` alone — without the worker there is no backend, so cloud files (the local D1 database) won't load.

In a fresh worktree, run `just worktree-setup` once first: it installs the web and worker dependencies, builds the wasm and the worker, and migrates the local D1 database, so `just dev` and `cd web && pnpm test:e2e` (which starts its own servers) work without further setup.

Once the worker is up, **always run `just copy-prod-db`** to copy production's rows into the local D1 database, so local dev runs against real accounts and files. It does nothing if this checkout already has a copy (it leaves `crates/live-share-worker/.wrangler/state/prod-data-copied-at`), so run it every time you start the dev servers. Each worktree has its own local database, so a fresh worktree needs its own copy. `just copy-prod-db force` copies again; `app_log` is never copied. It needs wrangler's stored login (`pnpm exec wrangler whoami`), and a running worker picks up the rows without a restart.

### Set up the test fixture for the user

When you start a local server so the user can try a feature or bug fix, **set up everything they need to test it before giving them the URL**. They should not have to create data or sign in.

- **Data:** put the state that reaches the change into the local database, e.g. a file that reproduces the bug, or the production copy migrated to a new format. Never write to production.
- **Sign-in:** if the change needs an account, sign the user in automatically. Insert a `sessions` row into the local D1 database with the SHA-256 hex of a new token, then serve the web app with a scratch Vite config that wraps `web/vite.config.ts` and uses `transformIndexHtml` to set `poentaro:account-auth:v2` (`{ token, email }`) in `localStorage`.
- **Keep it out of the repo:** fixture scripts and configs go in the scratchpad, not the commit.
- **Check it:** open the URL in a fresh browser and confirm the change can be reached. Then tell the user the URL and where to look.

## Debugging production

When asked why something happened in production (e.g. a user got signed out, or can't sign in), **query the `app_log` D1 table before guessing**. It holds 30 days of what the Synced Share worker did (see "App log (observability)" in `ARCHITECTURE.md`). Saved queries live in `crates/live-share-worker/queries/observability/` (`recent.sql`, `auth_rejections.sql`, `auth_by_version.sql`, `sign_ins_per_email.sql`, `request_failures.sql`, `by_trace_id.sql`). Run one against the production database from `crates/live-share-worker`:

```sh
pnpm exec wrangler d1 execute jianpu-live-share --remote --command="$(cat queries/observability/auth_rejections.sql)"
```

- When a user reports an error with a trace ID, edit the ID in `by_trace_id.sql` and run it: it returns every row written for that request, including the `request_failed` row (route, status, code).
- Keep the `--command=` form: the saved queries start with a `-- ` comment, which a space-separated `--command` would read as a flag. `--file` returns no rows for a remote `SELECT`.
- This uses wrangler's stored login (`pnpm exec wrangler whoami`). `cf d1 query <database-id> --sql="..."` also works, but needs `CLOUDFLARE_API_TOKEN` or `cf auth login`.
- Against a local `cf dev` database, use `sqlite3 .wrangler/state/v3/d1/miniflare-D1DatabaseObject/<hash>.sqlite ".read queries/observability/recent.sql"`. `wrangler d1 execute --local` can't find the database, because the D1 binding lives in `cloudflare.config.ts`.

## Staging D1

**Purpose:** prod D1 (`jianpu-live-share`) is the only other database, and the local e2e worker runs plain SQLite, not D1. Staging is where a migration or a risky SQL statement gets run against real D1, on a copy of prod data, before it reaches prod. Use it for anything D1 could handle differently from SQLite (table-rebuild migrations, multi-row atomicity, constraint behaviour). It is not a place to deploy the worker or run e2e. Prod is deployed straight from `master` with no staging gate, so this check is manual: do it before merging a PR that adds a migration or changes `crates/live-share-worker/queries/`.

Database: `jianpu-live-share-staging` (id `e388837b-aed7-495c-96d3-fca824639cfc`, same Cloudflare account as prod). Only the database exists: no staging worker is deployed, and nothing in the repo or CI points at it.

- **Contents:** a copy of prod data, last taken 2026-10-16 at migration `0004`, with `0005` applied on top (first use: the file-tree PR). It includes the `oauth_sessions` rows, so treat it as sensitive as prod. Earlier tests leave stray data (e.g. files renamed to `Zed/...`), so refresh before relying on it.
- **Refresh from prod** (read-only on prod): `npx wrangler d1 export jianpu-live-share --remote --output <file>.sql`, then `npx wrangler d1 execute jianpu-live-share-staging --remote --file <file>.sql`. The export uses plain `CREATE TABLE`, so importing into a database that already has tables should fail (not tried): drop the tables first or create a fresh database. Keep the export outside the repo.
- **Apply migrations:** `d1 migrations apply` needs a config naming the database, and there is none in the repo. Write a throwaway `wrangler.staging.jsonc` outside the repo: `{"name":"staging-tmp","compatibility_date":"2024-09-23","d1_databases":[{"binding":"DB","database_name":"jianpu-live-share-staging","database_id":"e388837b-aed7-495c-96d3-fca824639cfc","migrations_dir":"<absolute path to live-share-worker/migrations>"}]}`, then `npx wrangler d1 migrations apply DB --remote -c <that file>`. Wrangler asks no confirmation in a non-interactive shell, and `-y` is rejected.
- **Run SQL:** `npx wrangler d1 execute DB --remote -c <that file> --command "..."`. It can't bind parameters, so inline literals when replaying a query from `crates/live-share-worker/queries/`.
- **Never** point `deploy.json`, `cloudflare.config.ts` or the deploy workflow at the staging id, or the next push to `master` would deploy against it.

## Naming: the app is Poentaro, the notation is numbered

The app was renamed from jianpu-generator to Poentaro (see "Rename to Poentaro" in `ARCHITECTURE.md`). Anything that means **the app** is named `poentaro`: crates, modules, identifiers, storage keys, the `.poentaro` file extension, and every new cloud resource. Anything that means **the notation** is called numbered notation: the numbered view, `Notation::Numbered` (`notation = numbered`), `NumberedPitch`. Don't use the word jianpu for it in code, docs or UI.

Some cloud resources still say `jianpu` because they can't be renamed, only recreated and migrated, which nobody chose to do: the worker `jianpu-live-share-worker-rs` (its workers.dev host is what every deployed client calls), the D1 databases `jianpu-live-share` and `jianpu-live-share-staging`, the B2 bucket `jianpu-live-share-backups`, the Keychain service `jianpu-backup-reader`, and the old origins `jianpu-generator.pages.dev` and the `jianpu-generator` GitHub org. Use them by their real names. Don't "fix" them in config: a renamed value points at a resource that doesn't exist.

- **Never create anything new with `jianpu` in its name** that means the app: no new worker, database, bucket, Pages project, secret, Keychain entry, storage key or crate. Use `poentaro` (e.g. `poentaro-<purpose>`), even next to an old `jianpu-…` resource.
- Code that reads a legacy name (`.jianpu` files, `jianpu:` keys, `jianpu-outbox:` databases, the `jianpu-source` embed tag, `notation = jianpu` in a preset) exists only so old data keeps working. Keep it next to the current name, as in `file-extension.json` and `web/src/legacyStorage/`, and never write a legacy name.

## Cloudflare stays on the free plan

The Cloudflare account has no payment method, and the owner won't add one. Don't propose or build on anything that needs one, such as R2. That's why production backups go to Backblaze B2 (see "Backups" in `ARCHITECTURE.md`).

## Recovering production data

- Within the last 7 days, D1 Time Travel restores production to any minute: `pnpm exec wrangler d1 time-travel restore jianpu-live-share --timestamp=<ISO time>`. It overwrites production, so confirm with the owner first.
- Older than that, or if the database is gone, use the daily backups in B2. Run `just restore-backup <YYYY-MM-DD>` to load one into the local D1 database and check it before touching production. "Backups" in `ARCHITECTURE.md` covers restoring production itself.

## Handling secrets

- Never ask the owner to paste a key or token into the chat; the chat is stored. When they copy one to the clipboard, pipe `pbpaste` straight into `gh secret set <NAME>` (repo secrets) or `security add-generic-password -U -s <service> -a <account> -w "$(pbpaste)"` (macOS Keychain). Before storing, check its length and prefix with the characters masked, and check again right before storing, because the clipboard can change between commands. Verify by comparing hashes, with trailing newlines stripped on both sides.
- When a step needs a key only the owner should type (e.g. the Backblaze master key), and you're running inside herdr (`HERDR_ENV=1`), don't hand the command back. Open a pane (`herdr pane split` + `herdr pane run`) running a script that reads the key with `read -s`, then wait for its result with `herdr pane wait-output`.
- Create Backblaze keys with `b2 key create` and an explicit capability list. The web UI's "Write Only" option also grants `deleteFiles` and lifecycle-rule changes.
- The B2 restore key lives in each laptop's Keychain (service `jianpu-backup-reader`, accounts `keyID` and `applicationKey`). On a laptop without it, create one with the master key in a herdr pane: `uvx b2==5.0.0 key create --bucket jianpu-live-share-backups jianpu-backup-reader-<machine> listBuckets,listFiles,readFiles`. Then store both values there.
