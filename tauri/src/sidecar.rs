//! Where the bundled `life-pixel` CLI is (desktop.md, T3): beside the app's executable, where
//! `bundle.conf.json`'s `externalBin` puts it. Inside an AppImage, whose mount point changes at
//! each launch, a copy in the app's data folder gives an agent's configuration a stable path.

use std::fs::{self, File};
use std::io;
use std::path::{Path, PathBuf};

/// The sidecar's name, without the system's executable suffix.
const SIDECAR_NAME: &str = "life-pixel";
/// The folder of the app's data folder holding the AppImage's copy.
const BIN_FOLDER: &str = "bin";

/// The bundled CLI beside `executable`, when the build shipped it; with `stable_folder`, its copy
/// there, in `bin/`, made when missing or not the bundled one. `None` when there is no sidecar —
/// a build without `bundle.conf.json` — or its copy failed, which the log says.
#[must_use]
pub fn locate(executable: &Path, stable_folder: Option<&Path>) -> Option<PathBuf> {
    let bundled = beside(executable)?;
    let Some(folder) = stable_folder else {
        return Some(bundled);
    };
    let copy = folder.join(BIN_FOLDER).join(file_name());
    match refresh_copy(&bundled, &copy) {
        Ok(()) => Some(copy),
        Err(error) => {
            tracing::warn!(path = %copy.display(), %error, "bundled CLI not copied");
            None
        }
    }
}

/// The sidecar's file name on this system.
fn file_name() -> String {
    format!("{SIDECAR_NAME}{}", std::env::consts::EXE_SUFFIX)
}

/// The sidecar in `executable`'s folder, when it is there.
fn beside(executable: &Path) -> Option<PathBuf> {
    let sidecar = executable.parent()?.join(file_name());
    sidecar.is_file().then_some(sidecar)
}

/// Copies `bundled` to `copy` unless `copy` already has its modification time — which the copy
/// keeps, so that another version of the app, older or newer, replaces it. The copy is written
/// beside its target and renamed over it, so that a running agent never sees half a binary.
fn refresh_copy(bundled: &Path, copy: &Path) -> io::Result<()> {
    let modified = fs::metadata(bundled)?.modified()?;
    let current = fs::metadata(copy).and_then(|metadata| metadata.modified());
    if current.is_ok_and(|time| time == modified) {
        return Ok(());
    }
    let folder = copy.parent().unwrap_or_else(|| Path::new("."));
    fs::create_dir_all(folder)?;
    let temporary = tempfile::Builder::new()
        .prefix(".copying-")
        .tempfile_in(folder)?;
    fs::copy(bundled, temporary.path())?;
    File::options()
        .write(true)
        .open(temporary.path())?
        .set_modified(modified)?;
    temporary.persist(copy).map_err(|error| error.error)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use std::time::{Duration, SystemTime};

    use super::*;

    /// An app folder holding an executable and, when `with_sidecar`, the CLI beside it.
    fn app_folder(with_sidecar: bool) -> (tempfile::TempDir, PathBuf) {
        let folder = tempfile::tempdir().unwrap();
        let executable = folder.path().join("life-pixel-desktop");
        fs::write(&executable, b"app").unwrap();
        if with_sidecar {
            fs::write(folder.path().join(file_name()), b"cli v1").unwrap();
        }
        (folder, executable)
    }

    #[test]
    fn the_cli_is_found_beside_the_executable() {
        let (folder, executable) = app_folder(true);
        let expected = folder.path().join(file_name());
        assert_eq!(locate(&executable, None), Some(expected));
        let (_folder, executable) = app_folder(false);
        assert_eq!(locate(&executable, None), None);
    }

    #[test]
    fn an_appimage_gives_a_copy_in_the_data_folder_refreshed_when_it_changes() {
        let (folder, executable) = app_folder(true);
        let data = tempfile::tempdir().unwrap();
        let copy = locate(&executable, Some(data.path())).unwrap();
        assert_eq!(copy, data.path().join(BIN_FOLDER).join(file_name()));
        assert_eq!(fs::read(&copy).unwrap(), b"cli v1");

        let bundled = folder.path().join(file_name());
        fs::write(&bundled, b"cli v2").unwrap();
        let later = SystemTime::now() + Duration::from_secs(60);
        File::options()
            .write(true)
            .open(&bundled)
            .unwrap()
            .set_modified(later)
            .unwrap();
        assert_eq!(locate(&executable, Some(data.path())), Some(copy.clone()));
        assert_eq!(fs::read(&copy).unwrap(), b"cli v2");
    }
}
