//! HTTP routing for the Synced Share worker, ported from the old TS
//! `live-share-worker/src/index.ts`. CORS and the `OPTIONS` preflight are
//! handled once in `lib.rs` around whatever this router returns; this
//! module owns the Synced Share routes (`GET /shares/{share_id}` and
//! `POST /files/{id}/share{,/stop,/status}`, `POST /folders/share{,/stop,/status}`
//! and `POST /shares/overview`; `shares` submodule), the
//! `/files` routes (the cloud storage backend, see `crate::files`; `files`
//! submodule), plus the account routes `/auth/sign-up`, `/auth/sign-in`,
//! `/auth/sign-out` and `/auth/change-password` (`auth` submodule), and the
//! one-time origin-migration handoff `POST /origin-migrations{,/claim}`
//! (`origin_migrations` submodule, see `crate::origin_migration`).
//!
//! Every write (including starting/stopping a share) resolves its caller's
//! identity via `identity::resolve_verified_user_id` -- a session lookup,
//! never the client's own claim. A token that names no live session fails
//! the request closed with `ApiError::Unauthorized` (401), carrying a
//! `auth::session::VerificationFailure`: reason, timestamp, attempt count,
//! and deliberately nothing else -- the token/hash must never appear in
//! this response. Every `/files/*` route requires a resolved identity too.
//! The only anonymous read of a file's content is `GET /shares/{share_id}`,
//! and only while that file's share is live and the file isn't in the bin.
//!
//! `routes()` is the one list of routes: each entry registers into both the
//! worker `Router` and the OpenAPI spec (see `routes`), so the web client
//! generated from that spec can't drift from what's actually served. Every
//! failure goes through `fail`: an `ApiErrorBody` carrying the request's
//! trace ID, and one `request_failed` row in `app_log`.
//!
//! Helpers below (`is_unique_constraint_violation`, `resolve_files_caller`)
//! are private to this module but visible to every submodule per Rust's
//! ancestor-visibility rule -- no `pub(crate)` needed. `D1_BINDING` is
//! `pub(crate)` since `lib.rs`'s scheduled handler writes through it too.

mod agent;
mod api_keys;
mod auth;
mod e2e_reset;
mod files;
mod origin_migrations;
pub(crate) mod routes;
mod shares;

use worker::{console_error, D1Database, Env, Response, RouteContext};

use crate::api_error::{ApiError, ApiErrorBody};
use crate::app_log::{LogEvent, RequestEvent, RequestFailed, RequestMeta};
use crate::db;
use crate::identity::resolve_verified_user_id;
pub(crate) use agent::{serve as serve_agent, serves as serves_agent};
pub(crate) use routes::{request_meta, TRACE_ID_HEADER};
use routes::{HandlerResult, Routes};

/// D1 binding name this worker expects in `deploy.json`/`cloudflare.config.ts`. Wiring the
/// actual binding is task 5 (`TODO-synced-share-rust-d1-migration.md` §4's
/// "Update `deploy.json`/`cloudflare.config.ts`" bullet) -- out of scope here.
pub(crate) const D1_BINDING: &str = "DB";

/// Every route, for both serving and the spec. `e2e_reset_enabled` (see
/// `crate::e2e_reset`) decides only whether `POST /e2e/reset` is served;
/// the spec always documents it, so the e2e setup calls it through the
/// generated client.
pub(crate) fn routes(e2e_reset_enabled: bool) -> worker::Result<Routes> {
    Routes::new()
        .get("/shares/{share_id}", shares::get_share)?
        .post("/auth/sign-up", auth::sign_up)?
        .post("/auth/sign-in", auth::sign_in)?
        .post("/auth/sign-out", auth::sign_out)?
        .post("/auth/change-password", auth::change_password)?
        .post("/files/list", files::list_files)?
        .post("/files", files::create_file)?
        .post("/files/{id}/content", files::update_file_content)?
        .post("/files/{id}/versions", files::list_file_versions)?
        .post("/files/{id}/versions/get", files::get_file_version)?
        .post("/files/rename-batch", files::rename_files_batch)?
        .post("/files/{id}/rename", files::rename_file)?
        .post("/files/{id}/delete", files::delete_file)?
        .post("/files/{id}/restore", files::restore_file)?
        .post("/files/{id}/share", shares::start_share)?
        .post("/files/{id}/share/stop", shares::stop_share)?
        .post("/files/{id}/share/status", shares::share_status)?
        .post("/folders/share", shares::start_folder_share)?
        .post("/folders/share/stop", shares::stop_folder_share)?
        .post("/folders/share/status", shares::folder_share_status)?
        .post("/shares/overview", shares::share_overview)?
        .post("/api-keys/list", api_keys::list_api_keys)?
        .post("/api-keys", api_keys::create_api_key)?
        .post("/api-keys/{id}/revoke", api_keys::revoke_api_key)?
        .post("/origin-migrations", origin_migrations::upload)?
        .post("/origin-migrations/claim", origin_migrations::claim)?
        .post_if(e2e_reset_enabled, "/e2e/reset", e2e_reset::reset)
}

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

/// Resolves an authenticated request's caller identity -- see this
/// module's doc comment. A token that names no live session becomes
/// `ApiError::Unauthorized`, whose body is exactly `VerificationFailure`'s
/// fields, which never include the token or its hash.
async fn resolve_files_caller(
    d1: &D1Database,
    identity_token: &str,
    request_meta: &RequestMeta,
) -> HandlerResult<String> {
    resolve_verified_user_id(d1, identity_token, request_meta)
        .await?
        .map_err(ApiError::Unauthorized)
}

fn now_millis() -> i64 {
    worker::Date::now().as_millis() as i64
}

/// Refuses the request once `binding`'s per-IP limit is spent. Requests
/// with no `CF-Connecting-IP` (only local runs) share one bucket.
async fn check_rate_limit(
    ctx: &RouteContext<()>,
    binding: &str,
    request_meta: &RequestMeta,
) -> HandlerResult<()> {
    let key = request_meta
        .client_ip
        .clone()
        .unwrap_or_else(|| "unknown".to_string());
    let outcome = ctx.rate_limiter(binding)?.limit(key).await?;
    if outcome.success {
        Ok(())
    } else {
        Err(ApiError::RateLimited)
    }
}
