//! An animation as a JSON document, versioned and migrated on read, and a cel as a text grid.

mod document_v1;
pub mod grid;
mod json;
mod migrate;
mod rle;

pub use json::{DOCUMENT_FORMAT, DOCUMENT_VERSION, write_document};
pub use migrate::read_document;
