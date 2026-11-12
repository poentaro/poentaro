//! Reads and writes `.poentaro` documents, written in anyfix expressions (see
//! `syntax.md`).
//!
//! - The `poentaro_grammar` crate decodes the shapes a document is made of.
//! - [`decoded`] says what those shapes mean.
//! - [`read`] reads a decoded document into the document it describes.
//! - [`write`] writes the fields and part kinds the editor's forms change.
//!
//! Notes, lyrics and chords stay text: a score is one raw string.

pub(crate) mod decoded;
mod read;
mod words;
mod write;

pub(crate) use read::{read, read_metadata_field, source_part_declarations};
pub(crate) use words::{abbreviation, label, quoted};
pub(crate) use write::{key_value_field as field_text, part_kind_text, PartKindText};

#[cfg(test)]
mod read_tests;
