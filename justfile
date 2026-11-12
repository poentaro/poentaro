# Local dev runner for the app against a local Synced Share worker (accounts,
# cloud storage, Synced Share). Starts the worker and the Vite dev server together, streaming
# plain [worker]/[web]-prefixed logs to stdout and ./dev.log (no TUI, so it is
# safe to run in the background and tail the log). Ctrl-C / kill stops both.
#
# `cf` and `wrangler` are dev dependencies of crates/live-share-worker; run
# `just worktree-setup` once first. Under `vite` dev the web side talks to the
# local worker on localhost:8787 (see `web/vite.config.ts`).
dev:
    #!/usr/bin/env bash
    set -uo pipefail
    trap 'kill 0' EXIT INT TERM
    : > dev.log
    just dev-worker 2>&1 | sed -u 's/^/[worker] /' | tee -a dev.log &
    just dev-web 2>&1 | sed -u 's/^/[web] /' | tee -a dev.log &
    wait

# `cf dev` for crates/live-share-worker, on a local D1 with the migrations
# applied. Accounts made here exist only locally. Deliberately plain HTTP (no `--local-protocol
# https`): cf dev's self-signed cert is silently rejected by browsers (fetch()
# surfaces an opaque "Failed to fetch"), and nothing locally needs HTTPS (see
# `web/src/syncedShare/workerUrl.ts`).
[private]
dev-worker:
    #!/usr/bin/env bash
    set -euo pipefail
    cd crates/live-share-worker
    scripts/apply-local-migrations.sh
    exec pnpm exec cf dev --port 8787

[private]
dev-web:
    cd web && pnpm dev

# Readies a fresh checkout or worktree for `just dev` and the e2e tests:
# installs both packages' dependencies, builds the wasm component and the
# worker's generated types, builds the worker (so `cf dev` starts in
# seconds rather than compiling first) and applies the local D1 migrations.
worktree-setup:
    #!/usr/bin/env bash
    set -euo pipefail
    (cd web && pnpm install && pnpm run predev)
    cd crates/live-share-worker
    pnpm install
    worker-build --release
    scripts/apply-local-migrations.sh

# Replaces the local D1 database's rows (what `just dev`'s worker serves)
# with production's, so local dev runs against real accounts and files.
# Skips if this checkout already has a copy; `just copy-prod-db force`
# copies again. Every table except `app_log` and the migration/internal
# bookkeeping is copied. Reads production through wrangler's stored login
# (`pnpm exec wrangler whoami`); a running `just dev` picks the rows up
# without a restart.
copy-prod-db force="":
    #!/usr/bin/env bash
    set -euo pipefail
    cd crates/live-share-worker
    marker=.wrangler/state/prod-data-copied-at
    if [[ -f "$marker" && "{{force}}" != "force" ]]; then
      echo "Production data already copied at $(cat "$marker"); run \`just copy-prod-db force\` to copy again."
      exit 0
    fi
    # `just dev` (or `just worktree-setup`) already applied the migrations.
    db=$(ls .wrangler/state/v3/d1/miniflare-D1DatabaseObject/*.sqlite 2>/dev/null | grep -v /metadata.sqlite || true)
    if [[ -z "$db" ]]; then
      echo "No local D1 database yet; start \`just dev\` once so it applies the migrations, then rerun." >&2
      exit 1
    fi
    tables=$(sqlite3 "$db" "SELECT name FROM sqlite_master WHERE type = 'table' AND name NOT IN ('app_log', 'd1_migrations', 'sqlite_sequence') AND name NOT LIKE '\_cf\_%' ESCAPE '\' ORDER BY name")
    dump=$(mktemp)
    trap 'rm -f "$dump"' EXIT
    pnpm exec wrangler d1 export jianpu-live-share --remote --no-schema $(printf -- '--table %s ' $tables) --output "$dump"
    cp "$db" "$db.before-prod-copy"
    {
      echo "PRAGMA foreign_keys = OFF; BEGIN;"
      printf 'DELETE FROM "%s";\n' $tables
      grep -v '^PRAGMA' "$dump"
      echo "COMMIT;"
    } | sqlite3 -bail "$db"
    date -u +%Y-%m-%dT%H:%M:%SZ > "$marker"
    for table in $tables; do echo "$table: $(sqlite3 "$db" "SELECT count(*) FROM \"$table\"")"; done

# Creates the private Backblaze B2 bucket the daily production backup
# (`.github/workflows/d1-backup.yml`) uploads to, and sets its retention from
# `backup-lifecycle.json`. Safe to rerun: an existing bucket is left alone and
# the rules are replaced wholesale. Needs a B2 key with `writeBuckets` in
# `B2_APPLICATION_KEY_ID` / `B2_APPLICATION_KEY`; delete it afterwards.
setup-backup-bucket:
    #!/usr/bin/env bash
    set -euo pipefail
    cd crates/live-share-worker
    bucket=$(jq -r .backupBucket deploy.json)
    b2() { uvx b2==5.0.0 "$@"; }
    b2 bucket get "$bucket" >/dev/null 2>&1 || b2 bucket create --default-server-side-encryption SSE-B2 "$bucket" allPrivate
    rules=()
    while read -r rule; do rules+=(--lifecycle-rule "$rule"); done < <(jq -c '.[]' backup-lifecycle.json)
    b2 bucket update "${rules[@]}" "$bucket" allPrivate

# Replaces the local D1 database (what `just dev`'s worker serves) with a
# production backup: `just restore-backup 2026-10-28` for a `daily/` dump,
# `just restore-backup monthly/2026-10-01` for a monthly one. The previous
# local database is kept next to it as `*.sqlite.before-restore`. Downloads
# with the read-only B2 key from `B2_APPLICATION_KEY_ID` / `B2_APPLICATION_KEY`,
# or else from the macOS Keychain (service `jianpu-backup-reader`, accounts
# `keyID` and `applicationKey`).
restore-backup key:
    #!/usr/bin/env bash
    set -euo pipefail
    cd crates/live-share-worker
    db=$(ls .wrangler/state/v3/d1/miniflare-D1DatabaseObject/*.sqlite 2>/dev/null | grep -v /metadata.sqlite || true)
    if [[ -z "$db" ]]; then
      echo "No local D1 database yet; start \`just dev\` once so it creates one, then rerun." >&2
      exit 1
    fi
    key="{{key}}"
    [[ "$key" == */* ]] || key="daily/$key"
    dir=$(mktemp -d)
    trap 'rm -rf "$dir"' EXIT
    export B2_APPLICATION_KEY_ID="${B2_APPLICATION_KEY_ID:-$(security find-generic-password -s jianpu-backup-reader -a keyID -w)}"
    export B2_APPLICATION_KEY="${B2_APPLICATION_KEY:-$(security find-generic-password -s jianpu-backup-reader -a applicationKey -w)}"
    uvx b2==5.0.0 file download --no-progress "b2://$(jq -r .backupBucket deploy.json)/$key.sql.gz" "$dir/backup.sql.gz"
    gunzip "$dir/backup.sql.gz"
    tables=$(sqlite3 "$db" "SELECT name FROM sqlite_master WHERE type = 'table' AND name != 'sqlite_sequence' AND name NOT LIKE '\_cf\_%' ESCAPE '\'")
    cp "$db" "$db.before-restore"
    {
      echo "PRAGMA foreign_keys = OFF; BEGIN;"
      printf 'DROP TABLE "%s";\n' $tables
      grep -v '^PRAGMA' "$dir/backup.sql"
      echo "COMMIT;"
    } | sqlite3 -bail "$db"
    for table in $tables; do echo "$table: $(sqlite3 "$db" "SELECT count(*) FROM \"$table\"")"; done

# Mutation-tests how lyric parts follow their target's notes (lyric slots,
# the lyric cursor, and lyric spans) with cargo-mutants
# (`cargo install cargo-mutants`). Reports go to ./mutants.out.
mutants-lyrics:
    cargo mutants --file src/lyric_slots.rs --file src/compiler/mod.rs --file src/lyric_spans.rs --re "LyricCursor|lyric_target_visible|advance_lyric_cursors|measure_lyric_slots|list_lyric_spans_from_source" -- --lib

# Writes crates/poentaro-grammar/src/generated.rs again from the grammar,
# crates/poentaro-grammar/poentaro.anyfix, with the anyfix-generator its
# Cargo.toml pins.
generate-parser:
    UPDATE_GENERATED=1 cargo test -p poentaro-grammar --test generated
