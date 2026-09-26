//! Why a local export could not write its files: `docs/v1/mcp-cli.md`'s "Local writes".

use std::path::{Path, PathBuf};

use life_pixel_service::CodedError;
use serde_json::{Map, Value, json};
use thiserror::Error;

/// Every code this crate owns, each with its key `errors.<code>` in every catalogue.
pub const CODES: &[&str] = &[
    "export.directory_not_allowed",
    "export.file_exists",
    "export.symlink",
    "export.write_failed",
];

/// A local export write refused by `docs/mcp.md`'s "Limits and safety".
#[derive(Clone, Debug, PartialEq, Eq, Error)]
pub enum LocalWriteError {
    /// The directory lies outside the working directory and every `--allow-dir`.
    #[error("directory not allowed: {}", .0.display())]
    DirectoryNotAllowed(PathBuf),
    /// The file exists and the call did not ask to replace it.
    #[error("file exists: {}", .0.display())]
    FileExists(PathBuf),
    /// The file is a symbolic link, or a path to it leaves through one.
    #[error("symbolic link: {}", .0.display())]
    Symlink(PathBuf),
    /// The file could not be written for another reason; the cause is logged, never shown.
    #[error("cannot write {}", .0.display())]
    WriteFailed(PathBuf),
}

impl LocalWriteError {
    /// The path this error is about.
    fn path(&self) -> &Path {
        match self {
            Self::DirectoryNotAllowed(path)
            | Self::FileExists(path)
            | Self::Symlink(path)
            | Self::WriteFailed(path) => path,
        }
    }

    /// The stable code: one of [`CODES`].
    #[must_use]
    pub fn code(&self) -> &'static str {
        match self {
            Self::DirectoryNotAllowed(_) => "export.directory_not_allowed",
            Self::FileExists(_) => "export.file_exists",
            Self::Symlink(_) => "export.symlink",
            Self::WriteFailed(_) => "export.write_failed",
        }
    }

    /// This error's parameters: `path`, in every case.
    #[must_use]
    pub fn params(&self) -> Map<String, Value> {
        match json!({ "path": self.path().display().to_string() }) {
            Value::Object(map) => map,
            _ => Map::new(),
        }
    }
}

impl From<LocalWriteError> for CodedError {
    fn from(error: LocalWriteError) -> Self {
        Self {
            code: error.code(),
            params: error.params(),
        }
    }
}

/// Logs `reason` at `path`, then [`LocalWriteError::WriteFailed`].
pub(crate) fn write_failed(path: &Path, reason: impl std::fmt::Display) -> LocalWriteError {
    tracing::warn!(path = %path.display(), %reason, "export file not written");
    LocalWriteError::WriteFailed(path.to_path_buf())
}
