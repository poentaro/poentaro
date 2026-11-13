//! Native-only SQL syntax check for the `.sql` query files under
//! `queries/`, run against the shadow SQLite database `build.rs` builds
//! from `live-share-worker/migrations/`.
//!
//! This is the pragmatic substitute for `sqlx::query_file!`'s compile-time
//! checking described in `src/db/mod.rs`'s doc comment and the
//! `[build-dependencies] sqlx` comment in `Cargo.toml`: those macros can't
//! run inside this crate's own wasm-targeted compile (the
//! `libsqlite3-sys`-on-`wasm32-unknown-unknown` problem), so instead this
//! test loads the exact same `include_str!`-embedded query text used by
//! `src/db/`'s submodules at runtime and asks `sqlx` (here a dev-dependency, so it
//! never touches the wasm build) to *prepare* -- not execute -- each one
//! against the shadow schema. Preparing (rather than executing) means no
//! dummy bound values are needed and `NOT NULL`/foreign-key constraints
//! never come into play; this only proves each query parses and its
//! tables/columns exist, which is weaker than `query_file!`'s full
//! bound-parameter and result-column type checking, and does not catch any
//! D1-vs-SQLite dialect drift -- an honest, reasonable substitute given the
//! constraint above, not a silent downgrade of it.
//!
//! Not run under the `wasm32-unknown-unknown` target (this repo's `cargo
//! check --target wasm32-unknown-unknown` only checks, never tests, that
//! target, but the `cfg` below is a defensive belt-and-suspenders in case
//! that ever changes).

#![cfg(not(target_arch = "wasm32"))]

use sqlx::sqlite::SqliteConnectOptions;
use sqlx::{ConnectOptions, Executor};

const QUERIES: &[(&str, &str)] = &[
    (
        "prune_file_versions.sql",
        include_str!("../queries/prune_file_versions.sql"),
    ),
    (
        "list_file_versions.sql",
        include_str!("../queries/list_file_versions.sql"),
    ),
    (
        "insert_file_version.sql",
        include_str!("../queries/insert_file_version.sql"),
    ),
    (
        "advance_file_version_link.sql",
        include_str!("../queries/advance_file_version_link.sql"),
    ),
    (
        "prune_origin_migrations.sql",
        include_str!("../queries/prune_origin_migrations.sql"),
    ),
    (
        "claim_origin_migration.sql",
        include_str!("../queries/claim_origin_migration.sql"),
    ),
    (
        "insert_origin_migration.sql",
        include_str!("../queries/insert_origin_migration.sql"),
    ),
    (
        "delete_files_of_users.sql",
        include_str!("../queries/delete_files_of_users.sql"),
    ),
    (
        "delete_other_sessions.sql",
        include_str!("../queries/delete_other_sessions.sql"),
    ),
    (
        "delete_password_credentials_of_users.sql",
        include_str!("../queries/delete_password_credentials_of_users.sql"),
    ),
    (
        "delete_session.sql",
        include_str!("../queries/delete_session.sql"),
    ),
    (
        "delete_sessions_of_users.sql",
        include_str!("../queries/delete_sessions_of_users.sql"),
    ),
    (
        "delete_shares_of_users.sql",
        include_str!("../queries/delete_shares_of_users.sql"),
    ),
    (
        "delete_sign_in_lockout.sql",
        include_str!("../queries/delete_sign_in_lockout.sql"),
    ),
    (
        "delete_sign_in_lockouts_of_emails.sql",
        include_str!("../queries/delete_sign_in_lockouts_of_emails.sql"),
    ),
    (
        "delete_users.sql",
        include_str!("../queries/delete_users.sql"),
    ),
    (
        "end_folder_share.sql",
        include_str!("../queries/end_folder_share.sql"),
    ),
    ("end_share.sql", include_str!("../queries/end_share.sql")),
    (
        "extend_session.sql",
        include_str!("../queries/extend_session.sql"),
    ),
    (
        "get_file_by_id.sql",
        include_str!("../queries/get_file_by_id.sql"),
    ),
    (
        "get_file_names_by_ids_or_names.sql",
        include_str!("../queries/get_file_names_by_ids_or_names.sql"),
    ),
    (
        "get_folder_files_stats.sql",
        include_str!("../queries/get_folder_files_stats.sql"),
    ),
    (
        "get_folder_share_header.sql",
        include_str!("../queries/get_folder_share_header.sql"),
    ),
    (
        "get_folder_share_status.sql",
        include_str!("../queries/get_folder_share_status.sql"),
    ),
    (
        "get_password_credentials_by_email.sql",
        include_str!("../queries/get_password_credentials_by_email.sql"),
    ),
    (
        "get_password_credentials_by_user.sql",
        include_str!("../queries/get_password_credentials_by_user.sql"),
    ),
    (
        "get_session.sql",
        include_str!("../queries/get_session.sql"),
    ),
    (
        "get_share_status_by_file.sql",
        include_str!("../queries/get_share_status_by_file.sql"),
    ),
    (
        "get_share_view.sql",
        include_str!("../queries/get_share_view.sql"),
    ),
    (
        "get_sign_in_lockout.sql",
        include_str!("../queries/get_sign_in_lockout.sql"),
    ),
    (
        "insert_app_log.sql",
        include_str!("../queries/insert_app_log.sql"),
    ),
    (
        "insert_file.sql",
        include_str!("../queries/insert_file.sql"),
    ),
    (
        "insert_password_credentials.sql",
        include_str!("../queries/insert_password_credentials.sql"),
    ),
    (
        "insert_session.sql",
        include_str!("../queries/insert_session.sql"),
    ),
    (
        "insert_user.sql",
        include_str!("../queries/insert_user.sql"),
    ),
    (
        "list_active_files_under_prefixes.sql",
        include_str!("../queries/list_active_files_under_prefixes.sql"),
    ),
    (
        "list_files_by_owner.sql",
        include_str!("../queries/list_files_by_owner.sql"),
    ),
    (
        "list_folder_files.sql",
        include_str!("../queries/list_folder_files.sql"),
    ),
    (
        "list_folder_share_prefixes.sql",
        include_str!("../queries/list_folder_share_prefixes.sql"),
    ),
    (
        "list_live_file_names.sql",
        include_str!("../queries/list_live_file_names.sql"),
    ),
    (
        "list_live_file_shares_by_owner.sql",
        include_str!("../queries/list_live_file_shares_by_owner.sql"),
    ),
    (
        "list_live_folder_shares_by_owner.sql",
        include_str!("../queries/list_live_folder_shares_by_owner.sql"),
    ),
    (
        "list_user_ids_by_emails.sql",
        include_str!("../queries/list_user_ids_by_emails.sql"),
    ),
    (
        "observability/auth_by_version.sql",
        include_str!("../queries/observability/auth_by_version.sql"),
    ),
    (
        "observability/auth_rejections.sql",
        include_str!("../queries/observability/auth_rejections.sql"),
    ),
    (
        "observability/by_trace_id.sql",
        include_str!("../queries/observability/by_trace_id.sql"),
    ),
    (
        "observability/request_failures.sql",
        include_str!("../queries/observability/request_failures.sql"),
    ),
    (
        "observability/recent.sql",
        include_str!("../queries/observability/recent.sql"),
    ),
    (
        "observability/sign_ins_per_email.sql",
        include_str!("../queries/observability/sign_ins_per_email.sql"),
    ),
    (
        "prune_app_log.sql",
        include_str!("../queries/prune_app_log.sql"),
    ),
    (
        "rename_file.sql",
        include_str!("../queries/rename_file.sql"),
    ),
    (
        "rename_files_batch.sql",
        include_str!("../queries/rename_files_batch.sql"),
    ),
    (
        "restore_file.sql",
        include_str!("../queries/restore_file.sql"),
    ),
    (
        "rewrite_folder_share_prefix.sql",
        include_str!("../queries/rewrite_folder_share_prefix.sql"),
    ),
    (
        "share_id_exists.sql",
        include_str!("../queries/share_id_exists.sql"),
    ),
    ("trash_file.sql", include_str!("../queries/trash_file.sql")),
    (
        "update_file_content.sql",
        include_str!("../queries/update_file_content.sql"),
    ),
    (
        "update_password_hash.sql",
        include_str!("../queries/update_password_hash.sql"),
    ),
    (
        "upsert_folder_share.sql",
        include_str!("../queries/upsert_folder_share.sql"),
    ),
    (
        "upsert_share.sql",
        include_str!("../queries/upsert_share.sql"),
    ),
    (
        "upsert_sign_in_lockout.sql",
        include_str!("../queries/upsert_sign_in_lockout.sql"),
    ),
    (
        "insert_api_key.sql",
        include_str!("../queries/insert_api_key.sql"),
    ),
    (
        "list_api_keys.sql",
        include_str!("../queries/list_api_keys.sql"),
    ),
    (
        "delete_api_key.sql",
        include_str!("../queries/delete_api_key.sql"),
    ),
    (
        "get_api_key.sql",
        include_str!("../queries/get_api_key.sql"),
    ),
    (
        "touch_api_key.sql",
        include_str!("../queries/touch_api_key.sql"),
    ),
    (
        "delete_api_keys_of_users.sql",
        include_str!("../queries/delete_api_keys_of_users.sql"),
    ),
    (
        "get_file_by_name.sql",
        include_str!("../queries/get_file_by_name.sql"),
    ),
];

#[tokio::test]
async fn every_query_file_parses_against_the_shadow_schema() -> Result<(), sqlx::Error> {
    let shadow_db_path = concat!(env!("CARGO_MANIFEST_DIR"), "/shadow.sqlite");
    let mut connection = SqliteConnectOptions::new()
        .filename(shadow_db_path)
        .connect()
        .await?;

    for (name, query) in QUERIES {
        connection
            .prepare(query)
            .await
            .map_err(|error| sqlx::Error::Protocol(format!("{name} failed to parse: {error}")))?;
    }

    Ok(())
}
