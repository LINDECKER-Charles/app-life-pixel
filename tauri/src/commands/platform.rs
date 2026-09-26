//! What the app needs to know of the platform it runs on.

use serde::Serialize;
use tauri::State;

use crate::errors::CommandError;
use crate::state::DesktopState;

/// The app's version, its system and its folders.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PlatformInfo {
    /// The app's version.
    pub version: &'static str,
    /// `macos`, `windows` or `linux`.
    pub os: &'static str,
    /// The library folder in use.
    pub library_path: String,
    /// The bundled `life-pixel` CLI; `null` in a build without it.
    pub cli_path: Option<String>,
}

/// The app's version, its system, the library folder and the bundled CLI.
#[tauri::command]
pub async fn platform_info(state: State<'_, DesktopState>) -> Result<PlatformInfo, CommandError> {
    Ok(PlatformInfo {
        version: env!("CARGO_PKG_VERSION"),
        os: std::env::consts::OS,
        library_path: state.library_path().await.display().to_string(),
        cli_path: state.cli_path().map(|path| path.display().to_string()),
    })
}
