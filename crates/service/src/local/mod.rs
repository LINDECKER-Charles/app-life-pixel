//! The local library: projects and animations as files in a library folder, for the CLI and the
//! desktop app, [`Owner::Local`](crate::Owner::Local) only.
//!
//! ```text
//! <library>/
//!   library.json                                 {"format": "life-pixel/library", "version": 1}
//!   .lock                                        advisory lock, held while writing
//!   projects/<project-id>/project.json           {"id", "name", "createdAt", "updatedAt"}
//!   projects/<project-id>/animations/<id>.json   the document, as core serializes it
//! ```
//!
//! Writes go to a temporary file beside their target, are flushed, then renamed over it, under
//! the `.lock` file lock: a reader never sees half a document, and the desktop app, the CLI and
//! an agent can share a library. An animation's version is the first 6 bytes of the SHA-256 of
//! its file; its dates are the file's. Files the library does not know are ignored, and a
//! document that does not parse is left out of lists with a warning in the log.

mod animations;
mod default_path;
mod documents;
mod files;
mod folder;
mod library;
mod opening;
mod projects;
mod writes;

pub use default_path::default_library_path;
pub use library::LocalLibrary;
pub use opening::LocalLibraryError;
