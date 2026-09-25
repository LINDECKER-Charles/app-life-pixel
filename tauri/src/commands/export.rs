//! Saving exports: the files the app compiled, written where the person chooses.

use std::fs;
use std::path::{Path, PathBuf};

use serde::Deserialize;
use tauri::State;

use super::arguments;
use crate::errors::CommandError;
use crate::state::DesktopState;

/// A file of an export, as the app sends it.
#[derive(Clone, Debug, Deserialize)]
pub struct ExportFile {
    /// Its file name, without a folder.
    pub name: String,
    /// Its content, in base64.
    pub bytes: String,
}

/// A decoded file of an export.
struct Decoded {
    name: String,
    bytes: Vec<u8>,
}

/// Saves the files of an export: one through the system's save dialog, several into a folder
/// chosen in the folder picker — or, in a debug build with `LP_EXPORT_DIR`, into that folder
/// without a dialog. Answers the paths written, or `null` when the person cancels.
#[tauri::command]
pub async fn export_save_files(
    state: State<'_, DesktopState>,
    files: Vec<ExportFile>,
) -> Result<Option<Vec<String>>, CommandError> {
    let files = decode(files)?;
    let Some(paths) = destinations(&state, &files).await else {
        return Ok(None);
    };
    let written = paths
        .iter()
        .map(|path| path.display().to_string())
        .collect();
    let task = tokio::task::spawn_blocking(move || write_all(&paths, &files));
    task.await.map_err(CommandError::unavailable)??;
    Ok(Some(written))
}

/// The files, their names checked and their bytes decoded.
fn decode(files: Vec<ExportFile>) -> Result<Vec<Decoded>, CommandError> {
    if files.is_empty() {
        return Err(CommandError::malformed("files", "no file"));
    }
    files
        .into_iter()
        .map(|file| {
            if !is_plain_file_name(&file.name) {
                return Err(CommandError::malformed("files.name", &file.name));
            }
            let bytes = arguments::bytes("files.bytes", &file.bytes)?;
            Ok(Decoded {
                name: file.name,
                bytes,
            })
        })
        .collect()
}

/// Whether `name` names a file without reaching into another folder.
fn is_plain_file_name(name: &str) -> bool {
    let has_separator = name.contains(['/', '\\']);
    !has_separator && Path::new(name).file_name().is_some_and(|file| file == name)
}

/// Where each file goes, or `None` when the person cancels the dialog.
async fn destinations(state: &DesktopState, files: &[Decoded]) -> Option<Vec<PathBuf>> {
    if let Some(folder) = state.export_dir() {
        return Some(inside(folder, files));
    }
    if let [file] = files {
        return state
            .dialogs()
            .save_file(&file.name)
            .await
            .map(|path| vec![path]);
    }
    let folder = state.dialogs().pick_folder().await?;
    Some(inside(&folder, files))
}

/// The paths of `files` in `folder`.
fn inside(folder: &Path, files: &[Decoded]) -> Vec<PathBuf> {
    files.iter().map(|file| folder.join(&file.name)).collect()
}

/// Writes each file to its path, creating its folder when missing: `LP_EXPORT_DIR` may not
/// exist yet.
fn write_all(paths: &[PathBuf], files: &[Decoded]) -> Result<(), CommandError> {
    for (path, file) in paths.iter().zip(files) {
        let failed = |error| CommandError::export_write_failed(path, error);
        if let Some(folder) = path.parent() {
            fs::create_dir_all(folder).map_err(failed)?;
        }
        fs::write(path, &file.bytes).map_err(failed)?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_file_name_stays_in_its_folder() {
        assert!(is_plain_file_name("walk.gif"));
        assert!(is_plain_file_name("walk frame 1.png"));
        for name in [
            "",
            ".",
            "..",
            "../walk.gif",
            "sub/walk.gif",
            "sub\\walk.gif",
            "/walk.gif",
        ] {
            assert!(!is_plain_file_name(name), "{name:?}");
        }
    }
}
