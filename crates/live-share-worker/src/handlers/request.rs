use crate::{app_log::RequestMeta, crypto};
use worker::Request;
pub(crate) const TRACE_ID_HEADER: &str = "X-Trace-Id";
pub(crate) fn request_meta(req: &Request) -> RequestMeta {
    let header = |name: &str| req.headers().get(name).ok().flatten();
    RequestMeta {
        origin: header("Origin"),
        user_agent: header("User-Agent"),
        client_ip: header("CF-Connecting-IP"),
        trace_id: crypto::new_trace_id().ok(),
        worker_origin: req.url().ok().map(|url| url.origin().ascii_serialization()),
        resolved_user_id: Default::default(),
    }
}
