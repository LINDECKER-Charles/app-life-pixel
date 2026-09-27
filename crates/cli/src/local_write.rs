//! Writing exported files to disk, once their directory is resolved: a symbolic link is refused
//! outright, an existing file needs `overwrite`, and a file is created with `create_new` unless
//! `overwrite` — `docs/v1/mcp-cli.md`'s "Local writes". Shared by `export` and the `mcp`
//! transport's `export` tool.

use std::fs::{self, OpenOptions};
use std::io::Write as _;
use std::path::{Path, PathBuf};

use life_pixel_mcp::ExportFile;

use crate::errors::{LocalWriteError, write_failed};

/// Writes `files` into `directory`, already resolved and existing, giving the path each one
/// landed at. Every file is checked before any is written, so a later refusal never leaves a
/// partial export.
pub(crate) fn write_files(
    directory: &Path,
    files: &[ExportFile],
    overwrite: bool,
) -> Result<Vec<PathBuf>, LocalWriteError> {
    let targets: Vec<PathBuf> = files
        .iter()
        .map(|file| directory.join(&file.name))
        .collect();
    for target in &targets {
        check_target(target, overwrite)?;
    }
    for (target, file) in targets.iter().zip(files) {
        write_file(target, &file.bytes, overwrite)?;
    }
    Ok(targets)
}

/// A symbolic link at `path` is refused outright; an existing file needs `overwrite`.
fn check_target(path: &Path, overwrite: bool) -> Result<(), LocalWriteError> {
    match fs::symlink_metadata(path) {
        Ok(metadata) if metadata.file_type().is_symlink() => {
            Err(LocalWriteError::Symlink(path.to_path_buf()))
        }
        Ok(_) if !overwrite => Err(LocalWriteError::FileExists(path.to_path_buf())),
        _ => Ok(()),
    }
}

/// Writes `bytes` to `path`: `create_new` unless `overwrite`.
fn write_file(path: &Path, bytes: &[u8], overwrite: bool) -> Result<(), LocalWriteError> {
    let mut open = OpenOptions::new();
    open.write(true);
    if overwrite {
        open.create(true).truncate(true);
    } else {
        open.create_new(true);
    }
    let written = open.open(path).and_then(|mut file| file.write_all(bytes));
    written.map_err(|error| write_failed(path, error))
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
    fn files_are_written_with_create_new_by_default() {
        let root = TempDir::new().unwrap();

        let paths = write_files(root.path(), &[export_file("a.gif")], false).unwrap();

        assert_eq!(fs::read(&paths[0]).unwrap(), b"content");
    }

    #[test]
    fn an_existing_file_needs_overwrite_and_nothing_is_written_when_one_of_several_refuses() {
        let root = TempDir::new().unwrap();
        write_files(root.path(), &[export_file("a.gif")], false).unwrap();

        let files = [export_file("b.gif"), export_file("a.gif")];
        let refused = write_files(root.path(), &files, false).unwrap_err();

        assert_eq!(refused.code(), "export.file_exists");
        assert!(
            !root.path().join("b.gif").exists(),
            "b.gif must not be written"
        );
        let replaced = write_files(root.path(), &[export_file("a.gif")], true);
        assert!(replaced.is_ok());
    }

    #[cfg(unix)]
    #[test]
    fn a_symbolic_link_is_refused_even_with_overwrite() {
        let root = TempDir::new().unwrap();
        let elsewhere_dir = TempDir::new().unwrap();
        let elsewhere = elsewhere_dir.path().join("target.gif");
        fs::write(&elsewhere, b"outside").unwrap();
        std::os::unix::fs::symlink(&elsewhere, root.path().join("a.gif")).unwrap();

        let error = write_files(root.path(), &[export_file("a.gif")], true).unwrap_err();

        assert_eq!(error.code(), "export.symlink");
        assert_eq!(fs::read(&elsewhere).unwrap(), b"outside");
    }
}
