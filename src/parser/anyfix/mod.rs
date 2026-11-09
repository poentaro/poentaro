//! Reads and writes `.poentaro` documents, written in anyfix expressions (see
//! `syntax.md`).
//!
//! - [`read`] reads a document's anyfix tree into the document it describes.
//! - [`write`] writes the fields and part kinds the editor's forms change.
//!
//! Notes, lyrics and chords stay text: a score is one raw string.

mod read;
pub(crate) mod tree;
mod words;
mod write;

pub(crate) use read::{read, read_metadata_field, source_part_declarations};
pub(crate) use words::{abbreviation, label, quoted};
pub(crate) use write::{key_value_field as field_text, part_kind_text, PartKindText};

#[cfg(test)]
mod read_tests;
