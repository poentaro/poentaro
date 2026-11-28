//! `POST /api` and `/test`: the web app's protocols, each generated from
//! its schema in `crates/poentaro-protocol/schemas`.
//!
//! - `/api` is the anyx core schema `api`: a JSON list of calls. Each call
//!   carries its credential, and the schema decides which endpoints need a
//!   bearer session (see `api.rs`). A token that names no live session
//!   answers that call `unauthenticated`, which the web app turns into
//!   "session ended".
//! - `/test` sets up end-to-end tests, and exists only when
//!   `e2e_reset::EMAILS_VAR` is set.
//!
//! A generated client sends its schema's hash as `Poentaro-Schema`. A tab
//! built from another schema gets a `409`, and the web app asks to reload.
//! Each request writes one `app_log` row.

mod api;
mod app;
mod public;
mod test;

use std::cell::Cell;

use worker::{console_error, D1Database, Env, Method, Request, Response, Result};

use super::{request_meta, D1_BINDING, TRACE_ID_HEADER};
use crate::api_error::ApiError;
use crate::app_log::{LogEvent, ProtocolCalled, ProtocolEvent, ProtocolRefused, RequestMeta};
use crate::db;
use crate::e2e_reset;
use api::ApiServer;
use test::TestServer;

/// The header a generated client sends its schema's hash in.
pub(crate) const SCHEMA_HEADER: &str = "Poentaro-Schema";

#[derive(Clone, Copy)]
pub(crate) enum Endpoint {
    Api,
    Test,
}

impl Endpoint {
    fn name(self) -> &'static str {
        match self {
            Endpoint::Api => "api",
            Endpoint::Test => "test",
        }
    }

    fn schema_hash(self) -> &'static str {
        match self {
            Endpoint::Api => poentaro_protocol::api::SCHEMA_HASH,
            Endpoint::Test => poentaro_protocol::test::SCHEMA_HASH,
        }
    }
}

/// The endpoint `path` names, if this worker serves it.
pub(crate) fn endpoint(path: &str, env: &Env) -> Option<Endpoint> {
    match path {
        "/api" => Some(Endpoint::Api),
        "/test" if env.var(e2e_reset::EMAILS_VAR).is_ok() => Some(Endpoint::Test),
        _ => None,
    }
}

pub(crate) async fn serve(mut req: Request, env: &Env, endpoint: Endpoint) -> Result<Response> {
    if req.method() != Method::Post {
        return text(
            "Send requests as the body of a `POST`; send `SCHEMA` for how to write one.\n",
            405,
        );
    }
    let meta = request_meta(&req);
    let d1 = env.d1(D1_BINDING)?;
    let sent_hash = req.headers().get(SCHEMA_HEADER)?;
    if sent_hash.is_some_and(|hash| hash != endpoint.schema_hash()) {
        refuse(&d1, endpoint, 409, &meta).await;
        return text("Poentaro was updated. Reload to keep going.\n", 409);
    }
    let request = req.text().await?;
    let (response, calls, failed) = match endpoint {
        Endpoint::Api => {
            let server = ApiServer {
                d1: &d1,
                env,
                meta: &meta,
                failed: Cell::new(false),
            };
            let response = poentaro_protocol::api::handle_request(&server, &request).await;
            let calls = poentaro_protocol::api::called(&request);
            (response, calls, server.failed.get())
        }
        Endpoint::Test => {
            let mut failed = false;
            let on_internal_error = |error: worker::Error| {
                console_error!("test: internal error: {error}");
                failed = true;
            };
            let server = TestServer { d1: &d1, env };
            let response =
                poentaro_protocol::test::handle(&server, &request, on_internal_error).await;
            let calls =
                poentaro_protocol::method_names(poentaro_protocol::test::schema(), &request);
            (response, calls, failed)
        }
    };
    let event = ProtocolEvent::Called(ProtocolCalled {
        protocol: endpoint.name().to_string(),
        calls,
        failed,
    });
    log(&d1, event, &meta).await;
    let mut response = text_response(response, 200)?;
    if let Some(trace_id) = &meta.trace_id {
        response.headers_mut().set(TRACE_ID_HEADER, trace_id)?;
    }
    Ok(response)
}

/// An `ApiError` no method answers as one of its errors. By value, so it
/// reads as `.map_err(internal)`.
#[allow(clippy::needless_pass_by_value)]
fn internal(error: ApiError) -> worker::Error {
    let message = error.message().map(str::to_owned).unwrap_or_default();
    worker::Error::RustError(format!("{}: {message}", error.code()))
}

async fn refuse(d1: &D1Database, endpoint: Endpoint, status: u16, meta: &RequestMeta) {
    let event = ProtocolEvent::Refused(ProtocolRefused {
        protocol: endpoint.name().to_string(),
        status,
    });
    log(d1, event, meta).await;
}

async fn log(d1: &D1Database, event: ProtocolEvent, meta: &RequestMeta) {
    let user_id = meta.resolved_user_id.borrow().clone();
    db::record_app_log(d1, &LogEvent::Protocol(event), user_id.as_deref(), meta).await;
}

fn text(body: &str, status: u16) -> Result<Response> {
    text_response(body.to_string(), status)
}

fn text_response(body: String, status: u16) -> Result<Response> {
    let mut response = Response::ok(body)?.with_status(status);
    response
        .headers_mut()
        .set("Content-Type", "text/plain; charset=utf-8")?;
    // Protocol replies include live documents and account data. Keep the
    // same no-store policy as the REST routes, including refused requests.
    response.headers_mut().set("Cache-Control", "no-store")?;
    Ok(response)
}

/// A database timestamp at the generated protocol boundary. D1 stores
/// signed milliseconds; AnyX uses `SystemTime` and writes RFC 3339.
fn instant(milliseconds: i64) -> std::time::SystemTime {
    let offset = std::time::Duration::from_millis(milliseconds.unsigned_abs());
    if milliseconds >= 0 {
        std::time::UNIX_EPOCH + offset
    } else {
        std::time::UNIX_EPOCH - offset
    }
}

#[cfg(test)]
#[path = "protocols/tests.rs"]
mod tests;
