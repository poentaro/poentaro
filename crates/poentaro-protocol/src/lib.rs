//! The methods an account owner's AI agent calls on Poentaro:
//! `poentaro.protocol.anyx` is the schema, and `generated.rs` is the server
//! trait, codec and client anyx-protocol-generate writes from it.
//! `just generate-protocol` writes it again after the schema changes.
//!
//! The worker implements `Server` over D1 (`crates/live-share-worker`,
//! `handlers::agent`) and serves `POST /agent` through `handle`.

#[rustfmt::skip]
mod generated;

pub use generated::*;

/// What a generated type turns into and out of, for a server that needs to
/// name an error by its shape.
pub use anyx_protocol::{IntoValue, Value};

/// The guide `GET /agent` answers: how to write a request and read a
/// response, then the schema as written. `how_to_send` says where to send it.
pub fn guide(how_to_send: &str) -> String {
    anyx_protocol::guide(schema(), how_to_send)
}
