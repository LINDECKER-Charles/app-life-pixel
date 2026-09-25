//! The settings commands: read and change the settings, and choose the library folder.

use tauri::State;

use crate::errors::CommandError;
use crate::settings::Settings;
use crate::state::DesktopState;

/// The settings, `libraryPath` being the library folder in use.
#[tauri::command]
pub async fn settings_get(state: State<'_, DesktopState>) -> Result<Settings, CommandError> {
    Ok(state.settings().await)
}

/// Saves the settings; a new `libraryPath` reopens the library there, `null` in the default
/// folder. Answers the settings now in force.
#[tauri::command]
pub async fn settings_set(
    state: State<'_, DesktopState>,
    settings: Settings,
) -> Result<Settings, CommandError> {
    if !settings.has_valid_language() {
        return Err(CommandError::malformed(
            "settings.language",
            "not a language code",
        ));
    }
    state.apply_settings(settings).await?;
    Ok(state.settings().await)
}

/// A folder chosen in the system's folder picker, for the library; `null` when the person
/// cancels. Choosing does not change the settings: `settings_set` does.
#[tauri::command]
pub async fn settings_pick_library_folder(
    state: State<'_, DesktopState>,
) -> Result<Option<String>, CommandError> {
    let folder = state.dialogs().pick_folder().await;
    Ok(folder.map(|folder| folder.display().to_string()))
}
