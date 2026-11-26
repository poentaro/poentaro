//! Poentaro's protocols, each generated at build time from `schemas/`:
//!
//! - `api`: what the web app calls (`POST /api`), from the anyx core schema
//!   `schemas/api/api.anyx`. An endpoint needs a bearer session unless the
//!   schema says `access (public)`, and the types enforce it: a handler
//!   that needs an account only gets one from a verified token.
//! - `agent`: what an account owner's AI agent calls with an API key
//!   (`POST /agent`).
//! - `test`: what end-to-end tests call to set up (`POST /test`).
//!
//! The worker (`crates/live-share-worker`) implements their server traits
//! over D1.

include!(concat!(env!("OUT_DIR"), "/api.rs"));
include!(concat!(env!("OUT_DIR"), "/agent.rs"));
include!(concat!(env!("OUT_DIR"), "/test.rs"));

/// What a generated type turns into and out of, for a server that needs to
/// name an error by its shape.
pub use anyx_protocol::{IntoValue, Schema, Value};

/// The names of the methods `request` calls (`read file _`), or none
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
