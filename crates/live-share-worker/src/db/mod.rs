//! Raw D1 queries against the `shares` / `users` / `password_credentials` /
//! `sessions` / `sign_in_lockouts` / `files` / `file_versions` / `app_log` /
//! `origin_migrations` tables, split by table into `shares`, `accounts`,
//! `sessions`, `files`, `file_versions`, `app_log` and `origin_migrations`
//! submodules (re-exported here so every call site keeps using the flat
//! `db::function_name` form regardless of which table it hits).
//! Each query lives in its own file under `queries/`, loaded via
//! `include_str!` for the actual `D1Database::prepare()` calls -- these
//! submodules are the only place those files are read.
//!
//! Compile-time query checking via `sqlx::query_file!` (as originally
//! sketched in `TODO-synced-share-rust-d1-migration.md` §2) isn't possible
//! in this crate's own wasm-targeted compile -- see the long comment on the
//! `[build-dependencies] sqlx` entry in `Cargo.toml`. The pragmatic
//! substitute used instead: `tests/query_syntax.rs`, a native-only test
//! that loads these same query files (unmodified, via the identical
//! `include_str!` paths) and runs them through `sqlx`'s statement
//! preparation (not execution) against the shadow SQLite database built by
//! `build.rs`, to catch a broken/typo'd query before it would otherwise
//! only surface at real D1 runtime. This is weaker than `query_file!`'s
//! compile-time type-checking -- it doesn't check bound-parameter types or
//! result-column names against Rust types, and any D1-vs-SQLite dialect
//! drift is still uncaught -- but it's an honest, reasonable substitute
//! given the constraint above, not a silent downgrade.

mod accounts;
mod api_keys;
mod app_log;
mod e2e_reset;
mod file_summaries;
mod file_versions;
mod files;
mod origin_migrations;
mod sessions;
mod shares;

pub(crate) use accounts::*;
pub(crate) use api_keys::*;
pub(crate) use app_log::*;
pub(crate) use e2e_reset::*;
pub(crate) use file_summaries::*;
pub(crate) use file_versions::*;
pub(crate) use files::*;
pub(crate) use origin_migrations::*;
pub(crate) use sessions::*;
pub(crate) use shares::*;
