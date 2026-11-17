//! Writes to the `app_log` table (see `crate::app_log` for the typed events
//! and the table's purpose).

use worker::wasm_bindgen::JsValue;
use worker::{console_error, D1Database, Result};

use crate::app_log::{LogEvent, LogRow, RequestMeta, WORKER_VERSION};

const INSERT_APP_LOG: &str = include_str!("../../queries/insert_app_log.sql");
const PRUNE_APP_LOG: &str = include_str!("../../queries/prune_app_log.sql");

/// Appends `event` to `app_log`, stamped with the current time and
/// `WORKER_VERSION`. Best-effort: a failure is reported with
/// `console_error!` and never fails (or changes) the request the event
/// belongs to. Awaited rather than deferred (handlers have no `Context` for
/// `wait_until`), which is fine because every logged path is rare (a
/// failed request, or an auth event).
pub(crate) async fn record_app_log(
    db: &D1Database,
    event: &LogEvent,
    user_id: Option<&str>,
    request_meta: &RequestMeta,
) {
    let result = match event.to_row() {
        Ok(row) => insert_app_log(db, &row, user_id, request_meta).await,
        Err(error) => Err(worker::Error::RustError(error.to_string())),
    };
    if let Err(error) = result {
        console_error!("app_log: failed to record {event:?}: {error}");
    }
}

async fn insert_app_log(
    db: &D1Database,
    row: &LogRow,
    user_id: Option<&str>,
    request_meta: &RequestMeta,
) -> Result<()> {
    let at = worker::Date::now().as_millis() as f64;
    db.prepare(INSERT_APP_LOG)
        .bind(&[
            JsValue::from_f64(at),
            JsValue::from_str(&row.flow),
            JsValue::from_str(&row.kind),
            JsValue::from_str(row.outcome.as_str()),
            optional_str(user_id),
            optional_str(row.subject.as_deref()),
            JsValue::from_str(WORKER_VERSION),
            optional_str(request_meta.origin.as_deref()),
            optional_str(request_meta.user_agent.as_deref()),
            JsValue::from_str(&row.detail_json),
            optional_str(request_meta.trace_id.as_deref()),
        ])?
        .run()
        .await?;
    Ok(())
}

/// Deletes up to `batch_size` rows older than `cutoff_millis`, returning how
/// many were deleted.
pub(crate) async fn prune_app_log(
    db: &D1Database,
    cutoff_millis: i64,
    batch_size: u32,
) -> Result<usize> {
    let result = db
        .prepare(PRUNE_APP_LOG)
        .bind(&[
            JsValue::from_f64(cutoff_millis as f64),
            JsValue::from_f64(f64::from(batch_size)),
        ])?
        .run()
        .await?;
    Ok(result
        .meta()?
        .and_then(|meta| meta.changes)
        .unwrap_or_default())
}

fn optional_str(value: Option<&str>) -> JsValue {
    value.map(JsValue::from_str).unwrap_or(JsValue::NULL)
}
