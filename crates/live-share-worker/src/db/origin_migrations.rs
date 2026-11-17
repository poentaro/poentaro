//! Queries against the `origin_migrations` table (see
//! `crate::origin_migration`).

use serde::Deserialize;
use worker::wasm_bindgen::JsValue;
use worker::{D1Database, Result};

const INSERT_ORIGIN_MIGRATION: &str = include_str!("../../queries/insert_origin_migration.sql");
const CLAIM_ORIGIN_MIGRATION: &str = include_str!("../../queries/claim_origin_migration.sql");
const PRUNE_ORIGIN_MIGRATIONS: &str = include_str!("../../queries/prune_origin_migrations.sql");

/// A claimed handoff, already deleted from the table.
#[derive(Deserialize)]
pub(crate) struct ClaimedOriginMigration {
    pub(crate) payload: String,
    pub(crate) created_at: i64,
}

pub(crate) async fn insert_origin_migration(
    db: &D1Database,
    id: &str,
    payload: &str,
    created_at_millis: i64,
) -> Result<()> {
    db.prepare(INSERT_ORIGIN_MIGRATION)
        .bind(&[
            JsValue::from_str(id),
            JsValue::from_str(payload),
            JsValue::from_f64(created_at_millis as f64),
        ])?
        .run()
        .await?;
    Ok(())
}

/// Deletes handoff `id` and returns it, or `None` if there's no such row.
pub(crate) async fn claim_origin_migration(
    db: &D1Database,
    id: &str,
) -> Result<Option<ClaimedOriginMigration>> {
    db.prepare(CLAIM_ORIGIN_MIGRATION)
        .bind(&[JsValue::from_str(id)])?
        .first::<ClaimedOriginMigration>(None)
        .await
}

/// Deletes every handoff created before `cutoff_millis`.
pub(crate) async fn prune_origin_migrations(db: &D1Database, cutoff_millis: i64) -> Result<()> {
    db.prepare(PRUNE_ORIGIN_MIGRATIONS)
        .bind(&[JsValue::from_f64(cutoff_millis as f64)])?
        .run()
        .await?;
    Ok(())
}
