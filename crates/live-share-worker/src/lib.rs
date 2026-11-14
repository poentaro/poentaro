//! Entry point for the Rust rewrite of the Synced Share worker (see
//! `TODO-synced-share-rust-d1-migration.md`, task 4, and the "Synced Share
//! worker" section of `ARCHITECTURE.md` for the full picture).
//!
//! Ported from the old TS `live-share-worker/src/*.ts` (Workers KV + an
//! anonymous `ownerToken`) to D1 + account-required ownership. A share is a
//! pointer to a cloud `files` row (`shares.file_id`), owned through
//! `files.owner_user_id` -- see `share`. Accounts are email + password
//! (`auth`, routes in `handlers::auth`); every write resolves its caller
//! through `identity::resolve_verified_user_id`, a session lookup.
//!
//! `agent`, `api_error`, `api_keys`, `app_log`, `auth`, `e2e_reset`, `e2e_worker_config`,
//! `file_history`, `files`, `folder_share`, `protocol`, `share` and `share_id` are `pub`
//! (and D1/JsValue-free) so `tests/*.rs` can unit-test them directly, per
//! this repo's convention of keeping tests in separate files rather than inline
//! `#[cfg(test)]` modules. Everything else here is D1- or
//! wasm-runtime-facing and stays crate-private -- it isn't exercised by
//! host-side `cargo test` (real D1 integration testing is left to the e2e
//! suite against `cf dev`).

mod crypto;
mod db;
mod handlers;
mod identity;

pub mod agent;
pub mod api_error;
pub mod api_keys;
pub mod app_log;
pub mod auth;
pub mod e2e_reset;
pub mod e2e_worker_config;
pub mod file_history;
pub mod files;
pub mod folder_share;
pub mod naming;
pub mod origin_migration;
pub mod protocol;
pub mod share;
pub mod share_id;

use worker::{
    console_error, event, Context, Cors, Env, Method, Request, Response, Result, ScheduleContext,
    ScheduledEvent,
};

use crate::api_error::ApiError;
use crate::app_log::{prune_cutoff_millis, APP_LOG_PRUNE_BATCH_SIZE, APP_LOG_PRUNE_MAX_BATCHES};

/// The web app calls this worker cross-origin, so every response --
/// including the `OPTIONS` preflight a `POST`'s JSON body triggers -- needs
/// these. `*` is fine: there's no cookie auth here, just an unguessable
/// `share_id` plus a session token (`identity_token`) in the request body,
/// neither of which `Access-Control-Allow-Origin` exposes to an origin that
/// doesn't already have them (same reasoning as the old TS `index.ts`).
fn cors_config() -> Cors {
    Cors::new()
        .with_origins(["*"])
        .with_methods([Method::Get, Method::Post, Method::Options])
        .with_allowed_headers(["Content-Type", "Authorization", handlers::SCHEMA_HEADER])
        .with_exposed_headers([handlers::TRACE_ID_HEADER])
}

#[event(fetch)]
async fn fetch(req: Request, env: Env, _ctx: Context) -> Result<Response> {
    let cors = cors_config();

    if req.method() == Method::Options {
        return Response::empty()?.with_status(204).with_cors(&cors);
    }

    if handlers::serves_agent(&req.path()) {
        return handlers::serve_agent(req, &env).await?.with_cors(&cors);
    }

    if let Some(endpoint) = handlers::protocol_endpoint(&req.path(), &env) {
        return handlers::serve_protocol(req, &env, endpoint)
            .await?
            .with_cors(&cors);
    }

    let e2e_reset_enabled = env.var(e2e_reset::EMAILS_VAR).is_ok();
    let route = format!("{} {}", req.method(), req.path());
    let request_meta = handlers::request_meta(&req);
    let outcome = handlers::routes(e2e_reset_enabled)?
        .into_router()
        .run(req, env.clone())
        .await;
    let response = match outcome {
        Ok(response) if response.headers().has(handlers::TRACE_ID_HEADER)? => response,
        // No route matched: the router's own plain-text 404/405.
        Ok(_) => handlers::fail(&env, &route, ApiError::NotFound, &request_meta).await?,
        // The router itself failed, outside any route's `fail`.
        Err(error) => handlers::fail(&env, &route, error.into(), &request_meta).await?,
    };
    response.with_cors(&cors)
}

/// The daily cron trigger (`cloudflare.config.ts`): prunes `app_log` rows
/// older than `app_log::APP_LOG_RETENTION_MILLIS`, one batch at a time until
/// a batch comes back short, capped at `APP_LOG_PRUNE_MAX_BATCHES` per run
/// (the next run picks up any rest), then expired origin-migration handoffs.
#[event(scheduled)]
async fn scheduled(_event: ScheduledEvent, env: Env, _ctx: ScheduleContext) {
    if let Err(error) = prune_app_log(&env).await {
        console_error!("app_log: prune failed: {error}");
    }
    if let Err(error) = prune_origin_migrations(&env).await {
        console_error!("origin_migrations: prune failed: {error}");
    }
}

/// Drops origin-migration handoffs nobody claimed in time (see
/// `origin_migration::ORIGIN_MIGRATION_TTL_MILLIS`).
async fn prune_origin_migrations(env: &Env) -> Result<()> {
    let d1 = env.d1(handlers::D1_BINDING)?;
    let cutoff_millis =
        origin_migration::prune_cutoff_millis(worker::Date::now().as_millis() as i64);
    db::prune_origin_migrations(&d1, cutoff_millis).await
}

async fn prune_app_log(env: &Env) -> Result<()> {
    let d1 = env.d1(handlers::D1_BINDING)?;
    let cutoff_millis = prune_cutoff_millis(worker::Date::now().as_millis() as i64);
    for _ in 0..APP_LOG_PRUNE_MAX_BATCHES {
        let deleted = db::prune_app_log(&d1, cutoff_millis, APP_LOG_PRUNE_BATCH_SIZE).await?;
        if deleted < APP_LOG_PRUNE_BATCH_SIZE as usize {
            break;
        }
    }
    Ok(())
}

/// The OpenAPI spec of every route this worker serves, built from the same
/// `handlers::routes()` list the `fetch` handler above serves from -- see
/// `handlers::routes`. `tests/export_openapi.rs` writes it out for `web/`'s
/// generated client.
pub fn openapi_json() -> Result<String> {
    handlers::routes(false)?
        .into_openapi()
        .to_pretty_json()
        .map_err(|error| worker::Error::RustError(error.to_string()))
}
