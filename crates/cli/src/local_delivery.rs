//! Where the `mcp` transport's `export` tool writes its files: a directory inside the working
//! directory or an `--allow-dir` — `docs/v1/mcp-cli.md`'s "Local writes". The `export` command
//! writes through [`crate::local_write`] directly: it has no `--allow-dir` of its own.

use std::fs;
use std::path::{Path, PathBuf};

use async_trait::async_trait;
use life_pixel_mcp::{ExportCall, ExportDelivery, ExportFile};
use life_pixel_service::{CodedError, Owner};
use serde::Deserialize;
use serde_json::{Map, Value, json};

use crate::errors::LocalWriteError;
use crate::local_write::write_files;

/// The transport's own arguments of `export`, as the protocol sends them.
#[derive(Default, Deserialize)]
struct RawOptions {
    directory: Option<String>,
    #[serde(default)]
    overwrite: bool,
}

/// The transport's own arguments of `export`, ready to use.
pub(crate) struct LocalOptions {
    /// Relative to the working directory; the working directory itself when `None`.
    pub(crate) directory: Option<PathBuf>,
    /// Replaces an existing file instead of failing.
    pub(crate) overwrite: bool,
}

impl From<RawOptions> for LocalOptions {
    fn from(raw: RawOptions) -> Self {
        Self {
            directory: raw.directory.map(PathBuf::from),
            overwrite: raw.overwrite,
        }
    }
}

/// Delivers an export's files into the working directory or an `--allow-dir`.
#[derive(Clone)]
pub struct LocalExportDelivery {
    working_directory: PathBuf,
    allowed: Vec<PathBuf>,
}

impl LocalExportDelivery {
    /// Delivers into `working_directory` (always allowed) and each of `allow_dirs`; one that
    /// cannot be resolved grants no extra access, logged as a warning.
    #[must_use]
    pub fn new(working_directory: PathBuf, allow_dirs: &[PathBuf]) -> Self {
        let mut allowed = vec![canonical_or_self(&working_directory)];
        allowed.extend(allow_dirs.iter().filter_map(|dir| canonical_or_warn(dir)));
        Self {
            working_directory,
            allowed,
        }
    }

    /// Writes `files` under `options`, giving the path each one landed at.
    ///
    /// # Errors
    ///
    /// `export.directory_not_allowed`, `export.symlink`, `export.file_exists` or
    /// `export.write_failed`.
    pub(crate) fn write(
        &self,
        options: &LocalOptions,
        files: &[ExportFile],
    ) -> Result<Vec<PathBuf>, LocalWriteError> {
        let directory = self.resolve_directory(options.directory.as_deref())?;
        write_files(&directory, files, options.overwrite)
    }

    /// The requested directory, canonicalized and checked against [`Self::allowed`]; created
    /// with its parents when missing, once an existing ancestor is already allowed.
    fn resolve_directory(&self, requested: Option<&Path>) -> Result<PathBuf, LocalWriteError> {
        let target = requested.map_or_else(
            || self.working_directory.clone(),
            |relative| self.working_directory.join(relative),
        );
        let not_allowed = || LocalWriteError::DirectoryNotAllowed(target.clone());
        let ancestor = target.ancestors().find(|candidate| candidate.exists());
        let ancestor = ancestor.ok_or_else(not_allowed)?;
        let canonical_ancestor = fs::canonicalize(ancestor).map_err(|_| not_allowed())?;
        if !self.is_allowed(&canonical_ancestor) {
            return Err(not_allowed());
        }
        fs::create_dir_all(&target).map_err(|_| not_allowed())?;
        let canonical = fs::canonicalize(&target).map_err(|_| not_allowed())?;
        self.is_allowed(&canonical)
            .then_some(canonical)
            .ok_or_else(not_allowed)
    }

    fn is_allowed(&self, path: &Path) -> bool {
        self.allowed.iter().any(|root| path.starts_with(root))
    }
}

/// `path`, canonicalized, or itself when that fails: the working directory always grants at
/// least the path the caller gave.
fn canonical_or_self(path: &Path) -> PathBuf {
    fs::canonicalize(path).unwrap_or_else(|_| path.to_path_buf())
}

/// `path`, canonicalized, or `None` with a warning: an `--allow-dir` that does not exist grants
/// no access rather than failing the command.
fn canonical_or_warn(path: &Path) -> Option<PathBuf> {
    fs::canonicalize(path)
        .inspect_err(|error| tracing::warn!(path = %path.display(), %error, "allow-dir ignored"))
        .ok()
}

/// `request.malformed`, without parameters: the transport's own arguments do not match the
/// schema this delivery advertises.
fn malformed() -> CodedError {
    CodedError {
        code: "request.malformed",
        params: Map::new(),
    }
}

/// `service.unavailable`: the blocking write task panicked, which valid input never causes.
fn unavailable() -> CodedError {
    CodedError {
        code: "service.unavailable",
        params: Map::new(),
    }
}

#[async_trait]
impl ExportDelivery for LocalExportDelivery {
    fn export_schema(&self) -> schemars::Schema {
        schemars::json_schema!({
            "type": "object",
            "properties": {
                "directory": {
                    "type": "string",
                    "description": "Where to write the files, relative to the working \
                                    directory; the working directory itself when omitted."
                },
                "overwrite": {
                    "type": "boolean",
                    "description": "Replaces an existing file instead of failing.",
                    "default": false
                }
            }
        })
    }

    async fn deliver(
        &self,
        _owner: &Owner,
        request: ExportCall,
        files: Vec<ExportFile>,
    ) -> Result<Value, CodedError> {
        let raw: RawOptions =
            serde_json::from_value(Value::Object(request.options)).map_err(|_| malformed())?;
        let options = LocalOptions::from(raw);
        let delivery = self.clone();
        let task = tokio::task::spawn_blocking(move || delivery.write(&options, &files));
        let paths = task
            .await
            .map_err(|_| unavailable())?
            .map_err(CodedError::from)?;
        let files: Vec<String> = paths
            .iter()
            .map(|path| path.display().to_string())
            .collect();
        Ok(json!({ "files": files }))
    }
}

#[cfg(test)]
mod tests {
    use tempfile::TempDir;

    use super::*;

    fn export_file(name: &str) -> ExportFile {
        ExportFile {
            name: name.to_owned(),
            media_type: "image/gif",
            bytes: b"content".to_vec(),
        }
    }

    #[test]
    fn a_file_is_written_inside_the_working_directory_by_default() {
        let root = TempDir::new().unwrap();
        let delivery = LocalExportDelivery::new(root.path().to_path_buf(), &[]);
        let options = LocalOptions {
            directory: None,
            overwrite: false,
        };

        let paths = delivery
            .write(&options, &[export_file("mascot.gif")])
            .unwrap();

        assert_eq!(fs::read(&paths[0]).unwrap(), b"content");
        assert!(paths[0].starts_with(fs::canonicalize(root.path()).unwrap()));
    }

    #[test]
    fn a_directory_outside_every_allowed_one_is_refused() {
        let root = TempDir::new().unwrap();
        let outside = TempDir::new().unwrap();
        let delivery = LocalExportDelivery::new(root.path().to_path_buf(), &[]);
        let options = LocalOptions {
            directory: Some(outside.path().to_path_buf()),
            overwrite: false,
        };

        let error = delivery
            .write(&options, &[export_file("a.gif")])
            .unwrap_err();

        assert_eq!(error.code(), "export.directory_not_allowed");
    }

    #[test]
    fn an_allow_dir_widens_where_an_export_may_land() {
        let root = TempDir::new().unwrap();
        let allowed = TempDir::new().unwrap();
        let delivery =
            LocalExportDelivery::new(root.path().to_path_buf(), &[allowed.path().to_path_buf()]);
        let options = LocalOptions {
            directory: Some(allowed.path().to_path_buf()),
            overwrite: false,
        };

        let paths = delivery.write(&options, &[export_file("a.gif")]).unwrap();

        assert!(paths[0].starts_with(fs::canonicalize(allowed.path()).unwrap()));
    }

    #[test]
    fn a_relative_directory_is_created_under_the_working_directory() {
        let root = TempDir::new().unwrap();
        let delivery = LocalExportDelivery::new(root.path().to_path_buf(), &[]);
        let options = LocalOptions {
            directory: Some(PathBuf::from("out/nested")),
            overwrite: false,
        };

        let paths = delivery.write(&options, &[export_file("a.gif")]).unwrap();

        let expected = fs::canonicalize(root.path()).unwrap().join("out/nested");
        assert_eq!(paths[0], expected.join("a.gif"));
    }
}
