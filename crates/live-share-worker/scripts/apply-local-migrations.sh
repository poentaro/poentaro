#!/usr/bin/env bash
# Applies the D1 migrations to the local database `cf dev` serves
# (`.wrangler/state`; `cf dev` ignores `--persist-to`, so this is the only
# local database there is). Used by `just dev`, `just worktree-setup`, the
# e2e worker (`web/playwright.config.ts`) and CI.
#
# `cf d1 migrations apply` can hang mid-apply, or never exit after applying
# everything (even printing `[]` with nothing to apply), so it's skipped
# when every migration is already recorded, and otherwise run in the
# background, stopped as soon as every migration is recorded, and retried
# if that doesn't happen within 60s.
set -euo pipefail
cd "$(dirname "$0")/.."

database_id=$(node -p "require('./deploy.json').d1DatabaseId")
expected=$(ls ../../live-share-worker/migrations/*.sql | wc -l)

applied() {
  for db in .wrangler/state/v3/d1/miniflare-D1DatabaseObject/*.sqlite; do
    [ -e "$db" ] || continue
    case "$db" in */metadata.sqlite) continue ;; esac
    n=$(sqlite3 "$db" "select count(*) from d1_migrations" 2>/dev/null || echo 0)
    [ "$n" -ge "$expected" ] && return 0
  done
  return 1
}

if applied; then exit 0; fi

# Job control gives the background job its own process group, so stopping
# it also stops the `cf` process `pnpm exec` spawns.
set -m
for attempt in 1 2 3 4 5; do
  pnpm exec cf d1 migrations apply "$database_id" --local --dir ../../live-share-worker/migrations --persist-to .wrangler/state < /dev/null &
  pid=$!
  for _ in $(seq 60); do
    applied && break
    kill -0 "$pid" 2>/dev/null || break
    sleep 1
  done
  kill -- -"$pid" 2>/dev/null || true
  wait "$pid" 2>/dev/null || true
  if applied; then
    echo "migrations applied (attempt $attempt)"
    exit 0
  fi
  echo "schema missing after attempt $attempt" >&2
done
exit 1
