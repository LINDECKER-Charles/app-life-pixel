//! Resolving and opening the local library: `--library`, else `LIFE_PIXEL_LIBRARY`, else the
//! default path `dirs` finds — `docs/v1/mcp-cli.md`'s "Library".

use std::path::{Path, PathBuf};
use std::sync::Arc;

use life_pixel_service::animation::AnimationEditing;
use life_pixel_service::library::{Library, LibraryPorts};
use life_pixel_service::local::{LocalLibrary, default_library_path};
use life_pixel_service::ports::{EventSink, SystemClock, UuidV7Ids};
use life_pixel_service::{CodedError, Plans};

use crate::ports::NoEvents;

/// The environment variable naming the library folder, when `--library` is absent.
const LIBRARY_ENV: &str = "LIFE_PIXEL_LIBRARY";

/// `--library`, else `LIFE_PIXEL_LIBRARY`, else the default library `dirs` finds.
#[must_use]
pub fn resolve_path(explicit: Option<PathBuf>) -> PathBuf {
    explicit
        .or_else(|| std::env::var_os(LIBRARY_ENV).map(PathBuf::from))
        .unwrap_or_else(default_path)
}

/// `<documents>/Life Pixel`, or `<home>/Life Pixel` without a documents folder — the desktop
/// app's default.
fn default_path() -> PathBuf {
    default_library_path(dirs::document_dir(), dirs::home_dir().unwrap_or_default())
}

/// Opens the library at `path`, creating it with its parents when missing, and the use cases —
/// [`Owner::Local`](life_pixel_service::Owner::Local)'s only — over it.
///
/// # Errors
///
/// `library.unavailable` or `library.unsupported_version`.
pub fn open(path: &Path) -> Result<(Library, AnimationEditing), CodedError> {
    let store = LocalLibrary::create(path).map_err(|error| CodedError::of(&error))?;
    Ok(use_cases(store))
}

/// The library and its agent use cases over `store`, with no quota and no telemetry: both stay
/// dead code for [`Owner::Local`](life_pixel_service::Owner::Local).
fn use_cases(store: LocalLibrary) -> (Library, AnimationEditing) {
    let events: Arc<dyn EventSink> = Arc::new(NoEvents);
    let ports = LibraryPorts {
        store: Arc::new(store),
        clock: Arc::new(SystemClock),
        ids: Arc::new(UuidV7Ids),
        events: Arc::clone(&events),
    };
    let plans = Plans {
        free_storage_bytes: 0,
        free_mcp_calls_per_day: 0,
    };
    let library = Library::new(ports, plans);
    let editing = AnimationEditing::new(library.clone(), events);
    (library, editing)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_library_flag_wins_over_the_environment_and_the_default() {
        let explicit = PathBuf::from("/explicit");
        assert_eq!(resolve_path(Some(explicit.clone())), explicit);
    }
}
