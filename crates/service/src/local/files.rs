//! The file operations every write goes through: the library's lock, and a replacement that a
//! reader never sees half done.

use std::fs::{File, Metadata, OpenOptions};
use std::io::{self, Write as _};
use std::path::Path;
use std::time::SystemTime;

/// The advisory lock file, held while writing.
const LOCK_FILE: &str = ".lock";
/// The start of a temporary file's name: never a `<uuid>.json`, so lists ignore it.
const TEMPORARY_PREFIX: &str = ".writing-";

/// The result of `work`, run while holding the library's `.lock` for writing: the desktop app,
/// the CLI and an agent write one at a time.
pub(super) fn locked<T>(root: &Path, work: impl FnOnce() -> T) -> io::Result<T> {
    let file = OpenOptions::new()
        .create(true)
        .truncate(false)
        .write(true)
        .open(root.join(LOCK_FILE))?;
    let mut lock = fd_lock::RwLock::new(file);
    let _guard = lock.write()?;
    Ok(work())
}

/// Replaces `target` with `bytes`: written to a temporary file beside it, flushed, then renamed
/// over it. `created`, the creation time of the file it replaces, is kept where the platform
/// allows. Returns the new file's metadata.
pub(super) fn write_atomically(
    target: &Path,
    bytes: &[u8],
    created: Option<SystemTime>,
) -> io::Result<Metadata> {
    let directory = target.parent().ok_or(io::ErrorKind::InvalidInput)?;
    let mut temporary = tempfile::Builder::new()
        .prefix(TEMPORARY_PREFIX)
        .tempfile_in(directory)?;
    temporary.write_all(bytes)?;
    if let Some(created) = created {
        keep_creation(temporary.as_file(), created);
    }
    temporary.as_file().sync_all()?;
    let metadata = temporary.as_file().metadata()?;
    temporary.persist(target).map_err(|error| error.error)?;
    Ok(metadata)
}

/// Sets the modification time of the file at `path`.
pub(super) fn touch(path: &Path, modified: SystemTime) -> io::Result<()> {
    let file = OpenOptions::new().write(true).open(path)?;
    file.set_modified(modified)
}

/// Whether `error` says the file is not there.
pub(super) fn is_not_found(error: &io::Error) -> bool {
    error.kind() == io::ErrorKind::NotFound
}

/// Gives `file` the creation time `created`; a file system that refuses keeps the new time,
/// which only changes what the record shows.
#[cfg(any(target_os = "macos", windows))]
fn keep_creation(file: &File, created: SystemTime) {
    #[cfg(target_os = "macos")]
    use std::os::macos::fs::FileTimesExt as _;
    #[cfg(windows)]
    use std::os::windows::fs::FileTimesExt as _;

    let times = std::fs::FileTimes::new().set_created(created);
    if let Err(error) = file.set_times(times) {
        tracing::debug!(%error, "creation time not kept");
    }
}

/// Elsewhere a creation time cannot be set: a rewritten file is created anew.
#[cfg(not(any(target_os = "macos", windows)))]
fn keep_creation(_file: &File, _created: SystemTime) {}
