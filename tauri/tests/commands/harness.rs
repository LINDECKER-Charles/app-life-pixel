//! A test state over a temporary library, in a mock app, with dialogs that answer fixed paths.

use std::path::{Path, PathBuf};
use std::sync::Arc;

use async_trait::async_trait;
use base64::Engine as _;
use base64::engine::general_purpose::STANDARD;
use life_pixel_desktop::dialogs::Dialogs;
use life_pixel_desktop::errors::CommandError;
use life_pixel_desktop::state::{DesktopState, StateOptions};
use life_pixel_service::testing::sample_document;
use tauri::test::{MockRuntime, mock_app, mock_builder};
use tauri::{App, Manager, State};
use tempfile::TempDir;

/// What the dialogs answer; `None` is a cancel.
#[derive(Clone, Default)]
pub struct FakeDialogs {
    pub save_file: Option<PathBuf>,
    pub folder: Option<PathBuf>,
}

#[async_trait]
impl Dialogs for FakeDialogs {
    async fn save_file(&self, _file_name: &str) -> Option<PathBuf> {
        self.save_file.clone()
    }

    async fn pick_folder(&self) -> Option<PathBuf> {
        self.folder.clone()
    }
}

/// A mock app managing a [`DesktopState`] whose files are all in a temporary folder.
pub struct Harness {
    pub app: App<MockRuntime>,
    pub folder: TempDir,
}

impl Harness {
    /// The state on the default library, with dialogs that cancel.
    pub fn new() -> Self {
        Self::with(FakeDialogs::default(), |_| {})
    }

    /// The state with `dialogs`, its options changed by `adjust`.
    pub fn with(dialogs: FakeDialogs, adjust: impl FnOnce(&mut StateOptions)) -> Self {
        let folder = tempfile::tempdir().unwrap();
        let mut options = options_in(folder.path());
        adjust(&mut options);
        Self::start(folder, options, dialogs)
    }

    /// A state reading the same files, as after a restart.
    pub fn restart(self) -> Self {
        let options = options_in(self.folder.path());
        Self::start(self.folder, options, FakeDialogs::default())
    }

    /// The state in an app that registers the commands, as `run()` does, for calls through the
    /// IPC.
    pub fn with_commands() -> Self {
        let folder = tempfile::tempdir().unwrap();
        let app = mock_builder()
            .invoke_handler(life_pixel_desktop::commands::handler())
            .build(life_pixel_desktop::context())
            .unwrap();
        let options = options_in(folder.path());
        app.manage(DesktopState::open(
            options,
            Arc::new(FakeDialogs::default()),
        ));
        Self { app, folder }
    }

    fn start(folder: TempDir, options: StateOptions, dialogs: FakeDialogs) -> Self {
        let app = mock_app();
        app.manage(DesktopState::open(options, Arc::new(dialogs)));
        Self { app, folder }
    }

    pub fn state(&self) -> State<'_, DesktopState> {
        self.app.state::<DesktopState>()
    }

    /// A folder of the temporary folder, created.
    pub fn new_folder(&self, name: &str) -> PathBuf {
        let path = self.folder.path().join(name);
        std::fs::create_dir_all(&path).unwrap();
        path
    }

    pub fn default_library(&self) -> PathBuf {
        options_in(self.folder.path()).default_library
    }

    pub fn settings_file(&self) -> PathBuf {
        options_in(self.folder.path()).settings_file
    }
}

fn options_in(folder: &Path) -> StateOptions {
    StateOptions {
        settings_file: folder.join("config").join("settings.json"),
        default_library: folder.join("Documents").join("Life Pixel"),
        library_override: None,
        export_dir: None,
    }
}

/// A valid document titled `title`, in base64.
pub fn document(title: &str) -> String {
    STANDARD.encode(sample_document(title).1)
}

pub fn base64(bytes: &[u8]) -> String {
    STANDARD.encode(bytes)
}

pub fn decode(text: &str) -> Vec<u8> {
    STANDARD.decode(text).unwrap()
}

/// The code of a failed command.
pub fn code<T: std::fmt::Debug>(result: Result<T, CommandError>) -> &'static str {
    result.unwrap_err().code
}
