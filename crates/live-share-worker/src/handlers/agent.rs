//! `/agent`: where an account owner's AI agent reads and edits the owner's
//! scores, through the anyx protocol `crates/poentaro-protocol/schema` describes.
//!
//! `POST /agent` runs a request, sent with `Authorization: Bearer poe_…`.
//! Everything an agent reads comes through it: `SCHEMA` answers the guide
//! (how to write a request, then the schema) and `score/syntax` answers
//! `syntax.md`.
//!
//! It answers plain text, not `ApiErrorBody` JSON, since an agent reads
//! them, so `lib.rs` serves them before the JSON router. Each `POST` writes
//! one `app_log` row: the calls it ran, or the key it refused.

mod files;

use std::cell::RefCell;

use worker::{console_error, D1Database, Env, Method, Request, Response, Result};

use super::{now_millis, request_meta, D1_BINDING, TRACE_ID_HEADER};
use crate::api_keys::{bearer_key, is_expired, should_touch, shown_prefix, ApiKeyOwner};
use crate::app_log::{AgentEvent, Called, KeyEvent, KeyRejected, LogEvent, RequestMeta};
use crate::auth::session::hash_token;
use crate::db;
use files::AgentFiles;

/// The per-key limit on `POST /agent`. Must match `cloudflare.config.ts`.
const AGENT_RATE_LIMITER: &str = "AGENT_RATE_LIMITER";

/// Whether `lib.rs` should hand a request for `path` to `serve`.
pub(crate) fn serves(path: &str) -> bool {
    path == "/agent"
}

pub(crate) async fn serve(mut req: Request, env: &Env) -> Result<Response> {
    if req.method() != Method::Post {
        return text(
            "Send requests as the body of `POST /agent`; send `SCHEMA` for how to write one.\n",
            405,
        );
    }
    let meta = request_meta(&req);
    let key = req.headers().get("Authorization")?;
    let request = req.text().await?;
    let d1 = env.d1(D1_BINDING)?;
    run(&d1, env, key.as_deref(), &request, &meta).await
}

async fn run(
    d1: &D1Database,
    env: &Env,
    authorization: Option<&str>,
    request: &str,
    meta: &RequestMeta,
) -> Result<Response> {
    let Some(owner) = key_owner(d1, authorization).await? else {
        let prefix = authorization
            .and_then(|header| bearer_key(Some(header)))
            .map(shown_prefix);
        log(
            d1,
            AgentEvent::KeyRejected(KeyRejected { prefix }),
            None,
            meta,
        )
        .await;
        return text(
            "Found no valid API key in the `Authorization` header; expected \
             `Bearer poe_…`. The key may have been revoked: ask the account \
             owner for a new one from the account menu in Poentaro.\n",
            401,
        );
    };
    *meta.resolved_user_id.borrow_mut() = Some(owner.user_id.clone());
    let now = now_millis();
    if is_expired(owner.created_at, now) {
        let event = AgentEvent::KeyExpired(KeyEvent {
            prefix: owner.prefix,
        });
        log(d1, event, Some(&owner.user_id), meta).await;
        return text(
            "This API key has expired: keys work for 30 days. Ask the account \
             owner for a new one from the account menu in Poentaro.\n",
            401,
        );
    }
    if !env
        .rate_limiter(AGENT_RATE_LIMITER)?
        .limit(owner.id.clone())
        .await?
        .success
    {
        return text(
            "Too many requests with this key. Wait a minute, then send again.\n",
            429,
        );
    }
    if should_touch(owner.last_used_at, now) {
        if let Err(error) = db::touch_api_key(d1, &owner.id, now).await {
            console_error!("api_keys: failed to touch: {error}");
        }
    }
    let server = AgentFiles {
        d1,
        user_id: owner.user_id.clone(),
        calls: RefCell::new(Vec::new()),
    };
    let mut failed = false;
    let response = poentaro_protocol::handle(&server, request, |error| {
        console_error!("agent: internal error: {error}");
        failed = true;
    })
    .await;
    let event = AgentEvent::Called(Called {
        prefix: owner.prefix,
        calls: server.calls.into_inner(),
        failed,
    });
    log(d1, event, Some(&owner.user_id), meta).await;
    let mut response = text_response(response, 200)?;
    if let Some(trace_id) = &meta.trace_id {
        response.headers_mut().set(TRACE_ID_HEADER, trace_id)?;
    }
    Ok(response)
}

/// The key `authorization` names and whose it is, if it names one.
async fn key_owner(d1: &D1Database, authorization: Option<&str>) -> Result<Option<ApiKeyOwner>> {
    match bearer_key(authorization) {
        Some(key) => db::get_api_key(d1, &hash_token(key)).await,
        None => Ok(None),
    }
}

async fn log(d1: &D1Database, event: AgentEvent, user_id: Option<&str>, meta: &RequestMeta) {
    db::record_app_log(d1, &LogEvent::Agent(event), user_id, meta).await;
}

fn text(body: &str, status: u16) -> Result<Response> {
    text_response(body.to_string(), status)
}

fn text_response(body: String, status: u16) -> Result<Response> {
    let mut response = Response::ok(body)?.with_status(status);
    response
        .headers_mut()
        .set("Content-Type", "text/plain; charset=utf-8")?;
    Ok(response)
}
