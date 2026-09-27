//! What the commands share: the library, opened on the chosen folder, and the settings.

use std::path::{Path, PathBuf};
use std::sync::{Arc, OnceLock};

use life_pixel_service::Plans;
use life_pixel_service::library::{Library, LibraryPorts};
use life_pixel_service::local::{LocalLibrary, LocalLibraryError};
use life_pixel_service::ports::{EventSink, ProductEvent, SystemClock, UuidV7Ids};
use tokio::sync::RwLock;

use crate::dialogs::Dialogs;
use crate::errors::CommandError;
use crate::settings::Settings;
use crate::watcher::LibraryWatcher;

/// The local library has no plan: its values are never read for [`Owner::Local`].
///
/// [`Owner::Local`]: life_pixel_service::Owner::Local
const NO_PLAN: Plans = Plans {
    free_storage_bytes: 0,
    free_mcp_calls_per_day: 0,
};

/// Where the state finds its files, decided at start-up.
#[derive(Clone, Debug)]
pub struct StateOptions {
    /// `settings.json`, in the app's configuration folder.
    pub settings_file: PathBuf,
    /// The library folder when the settings name none.
    pub default_library: PathBuf,
    /// `LIFE_PIXEL_LIBRARY`: the library folder of this run, whatever the settings say.
    pub library_override: Option<PathBuf>,
    /// `LP_EXPORT_DIR`, debug builds only: where exports are saved without a dialog.
    pub export_dir: Option<PathBuf>,
    /// The bundled `life-pixel` CLI, when the build ships it.
    pub cli_path: Option<PathBuf>,
}

/// The library folder in use and the library over it — or why it could not be opened, which
/// every library command answers until another folder is chosen.
struct Opened {
    path: PathBuf,
    library: Result<Library, CommandError>,
}

/// The state every command receives.
pub struct DesktopState {
    options: StateOptions,
    dialogs: Arc<dyn Dialogs>,
    settings: RwLock<Settings>,
    opened: RwLock<Opened>,
    watcher: OnceLock<LibraryWatcher>,
}

impl DesktopState {
    /// Reads the settings and opens the library: `LIFE_PIXEL_LIBRARY`, else the settings' folder,
    /// else the default one, created when missing. Blocks on the file system.
    #[must_use]
    pub fn open(options: StateOptions, dialogs: Arc<dyn Dialogs>) -> Self {
        let settings = Settings::read(&options.settings_file);
        let chosen = settings.library_path.as_ref().map(PathBuf::from);
        let opened = match (&options.library_override, chosen) {
            (Some(path), _) => create_library(path),
            (None, Some(path)) => open_library(&path),
            (None, None) => create_library(&options.default_library),
        };
        Self {
            options,
            dialogs,
            settings: RwLock::new(settings),
            opened: RwLock::new(opened),
            watcher: OnceLock::new(),
        }
    }

    /// Hands the library folder in use to `watcher`, and each folder a settings change opens
    /// from now on. Only the first watcher is kept.
    pub async fn watch_library(&self, watcher: LibraryWatcher) {
        let opened = self.opened.read().await;
        watcher.watch(&opened.path);
        if self.watcher.set(watcher).is_err() {
            tracing::warn!("the library already has a watcher");
        }
    }

    /// The library, or why its folder could not be opened.
    ///
    /// # Errors
    ///
    /// `library.unavailable` or `library.unsupported_version` for the folder in use.
    pub async fn library(&self) -> Result<Library, CommandError> {
        self.opened.read().await.library.clone()
    }

    /// The library folder in use.
    pub async fn library_path(&self) -> PathBuf {
        self.opened.read().await.path.clone()
    }

    /// The settings, `libraryPath` being the folder in use.
    pub async fn settings(&self) -> Settings {
        let mut settings = self.settings.read().await.clone();
        settings.library_path = Some(self.library_path().await.display().to_string());
        settings
    }

    /// Saves `settings`. A `libraryPath` other than the folder in use opens that folder first —
    /// `None` being the default one — and nothing changes when it cannot be opened; the folder in
    /// use is never written to the file, so that `LIFE_PIXEL_LIBRARY` stays a run's choice.
    ///
    /// # Errors
    ///
    /// `library.unavailable` or `library.unsupported_version` for the new folder;
    /// `service.unavailable` when the settings file cannot be written.
    pub async fn apply_settings(&self, mut settings: Settings) -> Result<(), CommandError> {
        let mut stored = self.settings.write().await;
        let mut opened = self.opened.write().await;
        let requested = settings.library_path.as_ref().map(PathBuf::from);
        let default = &self.options.default_library;
        let reopened = if requested.as_ref().unwrap_or(default) == &opened.path {
            settings.library_path.clone_from(&stored.library_path);
            None
        } else {
            Some(self.reopen(requested).await?)
        };
        let file = self.options.settings_file.clone();
        let written = settings.clone();
        blocking(move || written.write(&file).map_err(CommandError::unavailable)).await??;
        *stored = settings;
        if let Some(reopened) = reopened {
            self.rewatch(&reopened.path);
            *opened = reopened;
        }
        Ok(())
    }

    /// Points the watcher, once there is one, at the library folder `path`.
    fn rewatch(&self, path: &Path) {
        if let Some(watcher) = self.watcher.get() {
            watcher.watch(path);
        }
    }

    /// The library in the folder `requested`, or in the default one — created — when `None`.
    async fn reopen(&self, requested: Option<PathBuf>) -> Result<Opened, CommandError> {
        let default = self.options.default_library.clone();
        let reopened = blocking(move || match requested {
            Some(path) => open_library(&path),
            None => create_library(&default),
        });
        let reopened = reopened.await?;
        reopened.library.as_ref().map_err(Clone::clone)?;
        Ok(reopened)
    }

    /// The system's dialogs.
    #[must_use]
    pub fn dialogs(&self) -> &dyn Dialogs {
        self.dialogs.as_ref()
    }

    /// The bundled `life-pixel` CLI, when the build ships it.
    #[must_use]
    pub fn cli_path(&self) -> Option<&Path> {
        self.options.cli_path.as_deref()
    }

    /// Where exports go without a dialog: `LP_EXPORT_DIR`, in debug builds only.
    #[must_use]
    pub fn export_dir(&self) -> Option<&Path> {
        self.options.export_dir.as_deref()
    }
}

/// The result of `work` on tokio's blocking pool.
async fn blocking<T, F>(work: F) -> Result<T, CommandError>
where
    T: Send + 'static,
    F: FnOnce() -> T + Send + 'static,
{
    let task = tokio::task::spawn_blocking(work);
    task.await.map_err(CommandError::unavailable)
}

/// The library in the existing folder `path`: one the person chose.
fn open_library(path: &Path) -> Opened {
    opened(path, LocalLibrary::open(path))
}

/// The library in the folder `path`, created when missing: the default one, or the override.
fn create_library(path: &Path) -> Opened {
    opened(path, LocalLibrary::create(path))
}

/// The library over `local`, once opened.
fn opened(path: &Path, local: Result<LocalLibrary, LocalLibraryError>) -> Opened {
    let library = local.map(library_over).map_err(CommandError::from);
    Opened {
        path: path.to_path_buf(),
        library,
    }
}

/// The use cases over the local library, with the system's clock and ids.
fn library_over(local: LocalLibrary) -> Library {
    let ports = LibraryPorts {
        store: Arc::new(local),
        clock: Arc::new(SystemClock),
        ids: Arc::new(UuidV7Ids),
        events: Arc::new(NoEvents),
    };
    Library::new(ports, NO_PLAN)
}

/// The desktop sends nothing (D24): the local owner records no event anyway.
struct NoEvents;

impl EventSink for NoEvents {
    fn record(&self, _event: ProductEvent) {}
}
