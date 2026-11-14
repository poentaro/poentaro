//! Poentaro's anyx protocols, one per audience, each generated at build
//! time from its folder in `schemas/` (a file per scope: `files.anyx` is the
//! scope `files`):
//!
//! - `agent`: what an account owner's AI agent calls with an API key
//!   (`POST /agent`).
//! - `app`: what the web app calls for a signed-in account (`POST /app`).
//! - `public`: what anyone calls without signing in (`POST /public`).
//! - `test`: what end-to-end tests call to set up (`POST /test`).
//!
//! Each module holds the scopes' server traits, `handle` and `Client`. The
//! worker (`crates/live-share-worker`) implements the server traits over D1.

include!(concat!(env!("OUT_DIR"), "/agent.rs"));
include!(concat!(env!("OUT_DIR"), "/app.rs"));
include!(concat!(env!("OUT_DIR"), "/public.rs"));
include!(concat!(env!("OUT_DIR"), "/test.rs"));

/// What a generated type turns into and out of, for a server that needs to
/// name an error by its shape.
pub use anyx_protocol::{IntoValue, Schema, Value};

/// The full names of the methods `request` calls (`files/read _`), or none
/// if it doesn't read. Never their arguments, which can hold passwords.
pub fn method_names(schema: &Schema, request: &str) -> Vec<String> {
    anyx_protocol::server::read_request(schema, request)
        .map(|calls| {
            calls
                .iter()
                .map(|call| schema.method_at(call.method).full_name())
                .collect()
        })
        .unwrap_or_default()
}
