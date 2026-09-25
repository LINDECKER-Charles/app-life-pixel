//! Opening a library folder: its `library.json`, created when missing, says which layout it
//! follows.

use std::fmt::Display;
use std::fs;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};
use thiserror::Error;

use super::files::{is_not_found, locked, write_atomically};
use crate::error::{Coded, params};

/// The file that marks a library folder.
const MANIFEST_FILE: &str = "library.json";
/// The `format` of `library.json`.
const LIBRARY_FORMAT: &str = "life-pixel/library";
/// The layout this code reads and writes.
const LIBRARY_VERSION: u64 = 1;

/// Why a library folder cannot be opened.
#[derive(Clone, Debug, PartialEq, Eq, Error)]
pub enum LocalLibraryError {
    /// The library was written by a later version of Life Pixel.
    #[error("library version {version} is not supported")]
    UnsupportedVersion {
        /// The version its `library.json` declares.
        version: u64,
    },
    /// The folder is missing, unreadable or unwritable, or its `library.json` is not one; the
    /// detail is logged.
    #[error("library unavailable at {}", path.display())]
    Unavailable {
        /// The library folder.
        path: PathBuf,
    },
}

impl LocalLibraryError {
    /// The folder `root` as unavailable, logging why.
    fn unavailable(root: &Path, reason: impl Display) -> Self {
        tracing::warn!(path = %root.display(), %reason, "library unavailable");
        Self::Unavailable {
            path: root.to_path_buf(),
        }
    }
}

impl Coded for LocalLibraryError {
    fn code(&self) -> &'static str {
        match self {
            Self::UnsupportedVersion { .. } => "library.unsupported_version",
            Self::Unavailable { .. } => "library.unavailable",
        }
    }

    fn params(&self) -> Map<String, Value> {
        match self {
            Self::UnsupportedVersion { version } => params([("version", Value::from(*version))]),
            Self::Unavailable { path } => {
                params([("path", Value::from(path.display().to_string()))])
            }
        }
    }
}

/// What `library.json` holds.
#[derive(Serialize, Deserialize)]
struct Manifest {
    format: String,
    version: u64,
}

/// Checks that `root` is a writable library folder of this version, writing its `library.json`
/// when it has none.
pub(super) fn open_manifest(root: &Path) -> Result<(), LocalLibraryError> {
    if !root.is_dir() {
        return Err(LocalLibraryError::unavailable(root, "not a folder"));
    }
    let opened = locked(root, || read_or_create(root));
    opened.map_err(|error| LocalLibraryError::unavailable(root, error))?
}

/// Checks the `library.json` of `root`, or writes it.
fn read_or_create(root: &Path) -> Result<(), LocalLibraryError> {
    let path = root.join(MANIFEST_FILE);
    let unavailable = |reason: &dyn Display| LocalLibraryError::unavailable(root, reason);
    tempfile::tempfile_in(root).map_err(|error| unavailable(&error))?;
    let bytes = match fs::read(&path) {
        Ok(bytes) => bytes,
        Err(error) if is_not_found(&error) => return create(&path).map_err(|e| unavailable(&e)),
        Err(error) => return Err(unavailable(&error)),
    };
    let manifest: Manifest = serde_json::from_slice(&bytes).map_err(|error| unavailable(&error))?;
    if manifest.format != LIBRARY_FORMAT {
        return Err(unavailable(&"not a Life Pixel library"));
    }
    if manifest.version != LIBRARY_VERSION {
        let version = manifest.version;
        return Err(LocalLibraryError::UnsupportedVersion { version });
    }
    Ok(())
}

/// Writes a `library.json` of this version at `path`.
fn create(path: &Path) -> std::io::Result<()> {
    let manifest = Manifest {
        format: LIBRARY_FORMAT.to_owned(),
        version: LIBRARY_VERSION,
    };
    let mut text = serde_json::to_vec_pretty(&manifest)?;
    text.push(b'\n');
    write_atomically(path, &text, None).map(drop)
}
