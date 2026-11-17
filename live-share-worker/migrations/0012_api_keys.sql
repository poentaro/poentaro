-- API keys: what an account owner's AI agent sends as `Authorization: Bearer
-- poe_…` to `POST /agent` (see `crates/live-share-worker/src/agent.rs`).
-- Like `sessions`, only the SHA-256 hex digest of the key is stored; the key
-- itself is shown once, when it is created. `prefix` is the start of the key,
-- for telling keys apart in the account menu and in `app_log`. Revoking a
-- key deletes its row.
CREATE TABLE api_keys (
    id TEXT PRIMARY KEY,
    user_id TEXT NOT NULL REFERENCES users(id),
    token_hash TEXT NOT NULL UNIQUE,
    prefix TEXT NOT NULL,
    created_at INTEGER NOT NULL,
    last_used_at INTEGER
);

CREATE INDEX idx_api_keys_user_id ON api_keys(user_id);
