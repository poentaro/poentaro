//! The methods an account owner's AI agent calls on Poentaro: `schema/` is
//! the schema, a file per scope (`files.anyx` is the scope `files`), and
//! `generated.rs` is the server traits, codec and client
//! anyx-protocol-generate writes from it, a module per scope.
//! `just generate-protocol` writes it again after the schema changes.
//!
//! The worker implements `Server`, `files::Server`,
//! `files::export::Server` and `score::Server` over D1 (`crates/live-share-worker`,
//! `handlers::agent`) and serves `POST /agent` through `handle`.

#[rustfmt::skip]
mod generated;

pub use generated::*;

/// What a generated type turns into and out of, for a server that needs to
/// name an error by its shape.
pub use anyx_protocol::{IntoValue, Value};
