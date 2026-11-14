//! `POST /public`, `/app` and `/test`: the web app's anyx protocols, each
//! generated from its schema in `crates/poentaro-protocol/schemas`.
//!
//! - `/public` needs no credential: sign up, sign in, open a link, the
//!   origin handoff.
//! - `/app` needs a session token as `Authorization: Bearer <token>`. One
//!   that names no live session is a `401` before anything runs, which the
//!   web app turns into "session ended".
//! - `/test` sets up end-to-end tests, and exists only when
//!   `e2e_reset::EMAILS_VAR` is set.
//!
//! A generated client sends its schema's hash as `Poentaro-Schema`. A tab
//! built from another schema gets a `409`, and the web app asks to reload.
//! Responses are anyx text, and each request writes one `app_log` row.

mod app;
mod public;
mod test;

use worker::{console_error, D1Database, Env, Method, Request, Response, Result};

use super::{request_meta, D1_BINDING, TRACE_ID_HEADER};
use crate::api_error::ApiError;
use crate::app_log::{LogEvent, ProtocolCalled, ProtocolEvent, ProtocolRefused, RequestMeta};
use crate::db;
use crate::e2e_reset;
use crate::identity::resolve_verified_user_id;
use app::AppServer;
use public::PublicServer;
use test::TestServer;

/// The header a generated client sends its schema's hash in.
pub(crate) const SCHEMA_HEADER: &str = "Poentaro-Schema";

#[derive(Clone, Copy)]
pub(crate) enum Endpoint {
    Public,
    App,
    Test,
}

impl Endpoint {
    fn name(self) -> &'static str {
        match self {
            Endpoint::Public => "public",
            Endpoint::App => "app",
            Endpoint::Test => "test",
        }
    }

    fn schema(self) -> &'static poentaro_protocol::Schema {
        match self {
            Endpoint::Public => poentaro_protocol::public::schema(),
            Endpoint::App => poentaro_protocol::app::schema(),
            Endpoint::Test => poentaro_protocol::test::schema(),
        }
    }

    fn schema_hash(self) -> &'static str {
        match self {
            Endpoint::Public => poentaro_protocol::public::SCHEMA_HASH,
            Endpoint::App => poentaro_protocol::app::SCHEMA_HASH,
            Endpoint::Test => poentaro_protocol::test::SCHEMA_HASH,
        }
    }
}

/// The endpoint `path` names, if this worker serves it.
pub(crate) fn endpoint(path: &str, env: &Env) -> Option<Endpoint> {
    match path {
        "/public" => Some(Endpoint::Public),
        "/app" => Some(Endpoint::App),
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
    // `/app` acts for the session's account; a token that names none is a
    // `401` for the whole request (`resolve_verified_user_id` logs why).
    let session = match endpoint {
        Endpoint::App => {
            let authorization = req.headers().get("Authorization")?;
            let token = bearer_token(authorization.as_deref()).unwrap_or_default();
            match resolve_verified_user_id(&d1, token, &meta).await? {
                Ok(user_id) => Some(Session {
                    token: token.to_string(),
                    user_id,
                }),
                Err(_) => {
                    refuse(&d1, endpoint, 401, &meta).await;
                    return text("Your session ended. Sign in again.\n", 401);
                }
            }
        }
        Endpoint::Public | Endpoint::Test => None,
    };
    let request = req.text().await?;
    let mut failed = false;
    let on_internal_error = |error: worker::Error| {
        console_error!("{}: internal error: {error}", endpoint.name());
        failed = true;
    };
    let response = match endpoint {
        Endpoint::Public => {
            let server = PublicServer {
                d1: &d1,
                env,
                meta: &meta,
            };
            poentaro_protocol::public::handle(&server, &request, on_internal_error).await
        }
        Endpoint::App => {
            let Some(Session { token, user_id }) = session else {
                return Err(worker::Error::RustError("no session".to_string()));
            };
            let server = AppServer {
                d1: &d1,
                meta: &meta,
                token,
                user_id,
            };
            poentaro_protocol::app::handle(&server, &request, on_internal_error).await
        }
        Endpoint::Test => {
            let server = TestServer { d1: &d1, env };
            poentaro_protocol::test::handle(&server, &request, on_internal_error).await
        }
    };
    let event = ProtocolEvent::Called(ProtocolCalled {
        protocol: endpoint.name().to_string(),
        calls: poentaro_protocol::method_names(endpoint.schema(), &request),
        failed,
    });
    log(&d1, event, &meta).await;
    let mut response = text_response(response, 200)?;
    if let Some(trace_id) = &meta.trace_id {
        response.headers_mut().set(TRACE_ID_HEADER, trace_id)?;
    }
    Ok(response)
}

/// The signed-in caller of `/app`.
struct Session {
    token: String,
    user_id: String,
}

/// The token in an `Authorization: Bearer <token>` header.
fn bearer_token(authorization: Option<&str>) -> Option<&str> {
    authorization?.trim().strip_prefix("Bearer ").map(str::trim)
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
    Ok(response)
}
