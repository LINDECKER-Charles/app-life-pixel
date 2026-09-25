//! The system's file dialogs, opened from Rust so that the webview holds no file-system
//! permission. Commands ask [`Dialogs`]; tests give them fixed answers.

use std::path::PathBuf;

use async_trait::async_trait;
use tauri::{AppHandle, Runtime};
use tauri_plugin_dialog::{DialogExt, FilePath};
use tokio::sync::oneshot;

/// Where the person chooses to save or to keep things; `None` when they cancel.
#[async_trait]
pub trait Dialogs: Send + Sync {
    /// The file to save `file_name` as, from the system's save dialog.
    async fn save_file(&self, file_name: &str) -> Option<PathBuf>;
    /// A folder, from the system's folder picker.
    async fn pick_folder(&self) -> Option<PathBuf>;
}

/// The dialogs of `tauri-plugin-dialog`, attached to the app.
pub struct SystemDialogs<R: Runtime> {
    app: AppHandle<R>,
}

impl<R: Runtime> SystemDialogs<R> {
    /// The dialogs of `app`.
    #[must_use]
    pub fn new(app: AppHandle<R>) -> Self {
        Self { app }
    }
}

#[async_trait]
impl<R: Runtime> Dialogs for SystemDialogs<R> {
    async fn save_file(&self, file_name: &str) -> Option<PathBuf> {
        let (sender, receiver) = oneshot::channel();
        self.app
            .dialog()
            .file()
            .set_file_name(file_name)
            .save_file(move |path| drop(sender.send(path)));
        chosen_path(receiver.await.ok().flatten())
    }

    async fn pick_folder(&self) -> Option<PathBuf> {
        let (sender, receiver) = oneshot::channel();
        self.app
            .dialog()
            .file()
            .pick_folder(move |path| drop(sender.send(path)));
        chosen_path(receiver.await.ok().flatten())
    }
}

/// The local path of a dialog's answer; a URL, which only mobile dialogs give, counts as none.
fn chosen_path(path: Option<FilePath>) -> Option<PathBuf> {
    path?.into_path().ok()
}
