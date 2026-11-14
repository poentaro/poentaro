//! Files without their content, for `poentaro app`.

use serde::Deserialize;
use worker::wasm_bindgen::JsValue;
use worker::{D1Database, Result};

use crate::naming::FileSummaryRow;

const LIST_FILE_SUMMARIES: &str = include_str!("../../queries/list_file_summaries.sql");

#[derive(Deserialize)]
struct FileSummaryRowRaw {
    id: String,
    name: String,
    revision: i64,
    trashed_at: Option<i64>,
}

/// Every file of the owner, in the trash or not.
pub(crate) async fn list_file_summaries(
    db: &D1Database,
    owner_user_id: &str,
) -> Result<Vec<FileSummaryRow>> {
    let rows: Vec<FileSummaryRowRaw> = db
        .prepare(LIST_FILE_SUMMARIES)
        .bind(&[JsValue::from_str(owner_user_id)])?
        .all()
        .await?
        .results()?;
    Ok(rows
        .into_iter()
        .map(|row| FileSummaryRow {
            id: row.id,
            name: row.name,
            revision: row.revision,
            trashed_at: row.trashed_at,
        })
        .collect())
}
