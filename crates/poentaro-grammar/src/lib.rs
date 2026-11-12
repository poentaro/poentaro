//! The shapes a `.poentaro` document is made of: `poentaro.anyx` is the
//! grammar, and `generated.rs` is the decoder anyx-generator writes from it.
//! `just generate-parser` writes it again after the grammar changes.
//!
//! What the shapes mean, such as which metadata keys exist, is read in the
//! main crate (`src/parser/anyx/`).

#[rustfmt::skip]
mod generated;

pub use generated::*;
