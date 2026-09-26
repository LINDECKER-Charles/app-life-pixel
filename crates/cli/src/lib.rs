//! `life-pixel`: `mcp` serves the tools of `docs/mcp.md` over stdio on the local library,
//! `list` and `export` read and export it directly. One Rust core, one library, three doors.
//!
//! Every command opens `service::local::LocalLibrary` at [`library::resolve_path`] and reads or
//! writes it for [`Owner::Local`](life_pixel_service::Owner::Local) alone; `export` and `mcp`
//! share [`local_delivery::LocalExportDelivery`], the local write rules of `docs/mcp-cli.md`.
//! Output is `println!` only: the protocol on stdout in `mcp` mode, results elsewhere, logs
//! always on stderr; a failure prints its message, from [`messages::Messages`], and exits 1.

pub mod args;
pub mod caller_policy;
pub mod commands;
pub mod errors;
pub mod library;
pub mod local_delivery;
pub mod local_write;
pub mod messages;
pub mod ports;

use life_pixel_service::CodedError;

use crate::messages::Messages;

/// Why a command stopped: a coded failure the catalogues translate, or a plain line already in
/// the right language — an argument the catalogues do not cover, such as an unreadable path.
#[derive(Debug)]
pub enum AppError {
    /// A use case's or the local delivery's failure.
    Coded(CodedError),
    /// A line to print as it is.
    Plain(String),
}

impl From<CodedError> for AppError {
    fn from(error: CodedError) -> Self {
        Self::Coded(error)
    }
}

impl AppError {
    /// The line to print on stderr.
    #[must_use]
    pub fn message(&self, messages: &Messages) -> String {
        match self {
            Self::Coded(error) => messages.error(error),
            Self::Plain(text) => text.clone(),
        }
    }
}
