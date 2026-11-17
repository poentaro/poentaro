-- One-time handoffs of a browser's saved data from an old web origin to the
-- app's current one (see `crates/live-share-worker/src/origin_migration.rs`).
-- The old origin's bridge page uploads everything it kept in localStorage as
-- an opaque JSON `payload`, then redirects to the new origin with the row's
-- unguessable `id`; the new origin claims the row, which deletes it. Rows
-- never claimed expire after `ORIGIN_MIGRATION_TTL_MILLIS` and are pruned by
-- the worker's daily scheduled handler. Holds no account identity: only
-- whatever the browser held (which may include its own session token).

CREATE TABLE origin_migrations (
    id         TEXT PRIMARY KEY,
    payload    TEXT NOT NULL,
    created_at INTEGER NOT NULL  -- ms since epoch
);

CREATE INDEX idx_origin_migrations_created_at ON origin_migrations(created_at);
