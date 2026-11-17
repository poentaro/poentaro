//! AnyX endpoint dispatch and shared service helpers.

mod agent;
mod api_keys;
mod auth;
mod file_store;
mod origin_migrations;
mod protocols;
mod request;
mod shares;

use worker::{console_error, Env, Response};

use crate::api_error::{ApiError, ApiErrorBody};
use crate::app_log::{LogEvent, RequestEvent, RequestFailed, RequestMeta};
use crate::db;
pub(crate) use agent::{serve as serve_agent, serves as serves_agent};
pub(crate) use protocols::{endpoint as protocol_endpoint, serve as serve_protocol, SCHEMA_HEADER};
pub(crate) use request::{request_meta, TRACE_ID_HEADER};
type HandlerResult<T> = Result<T, ApiError>;

/// D1 binding name this worker expects in `deploy.json`/`cloudflare.config.ts`. Wiring the
/// actual binding is task 5 (`TODO-synced-share-rust-d1-migration.md` §4's
/// "Update `deploy.json`/`cloudflare.config.ts`" bullet) -- out of scope here.
pub(crate) const D1_BINDING: &str = "DB";

/// Answers `error` to a request for `route`, and logs it to `app_log` under
/// the request's trace ID. The one place a failure becomes a response, so no
/// failure can go unlogged or reach the user without its trace ID.
pub(crate) async fn fail(
    env: &Env,
    route: &str,
    error: ApiError,
    request_meta: &RequestMeta,
) -> worker::Result<Response> {
    let status = error.status_code();
    let event = LogEvent::Request(RequestEvent::RequestFailed(RequestFailed {
        route: route.to_string(),
        status,
        code: error.code(),
        message: error.message().map(str::to_owned),
    }));
    match env.d1(D1_BINDING) {
        Ok(d1) => {
            let user_id = request_meta.resolved_user_id.borrow().clone();
            db::record_app_log(&d1, &event, user_id.as_deref(), request_meta).await;
        }
        Err(d1_error) => console_error!("app_log: no database to record {event:?}: {d1_error}"),
    }
    let body = ApiErrorBody {
        error,
        trace_id: request_meta.trace_id.clone().unwrap_or_default(),
    };
    let mut response = Response::from_json(&body)?.with_status(status);
    response
        .headers_mut()
        .set(TRACE_ID_HEADER, &body.trace_id)?;
    Ok(response)
}

/// D1 surfaces a SQLite `UNIQUE constraint failed` violation as a plain
/// stringly-typed `worker::Error`, not a structured error code -- so this
/// checks the message text.
fn is_unique_constraint_violation(error: &worker::Error) -> bool {
    error.to_string().contains("UNIQUE constraint failed")
}

fn now_millis() -> i64 {
    worker::Date::now().as_millis() as i64
}

/// Refuses the request once `binding`'s per-IP limit is spent. Requests
/// with no `CF-Connecting-IP` (only local runs) share one bucket.
async fn check_rate_limit(
    env: &Env,
    binding: &str,
    request_meta: &RequestMeta,
) -> HandlerResult<()> {
    let key = request_meta
        .client_ip
        .clone()
        .unwrap_or_else(|| "unknown".to_string());
    let outcome = env.rate_limiter(binding)?.limit(key).await?;
    if outcome.success {
        Ok(())
    } else {
        Err(ApiError::RateLimited)
    }
}
