//! The commands the app invokes. Each is declared in `build.rs`, which gives it the permission
//! `capabilities/default.json` grants; arguments are camelCase, documents and files base64.

mod arguments;
pub mod export;
pub mod library;
pub mod platform;
pub mod settings;
pub mod shapes;

use tauri::Runtime;
use tauri::ipc::Invoke;

/// The handler of every command, for any runtime: the app's, and the tests' mock one. `build.rs`
/// lists the same names.
pub fn handler<R: Runtime>() -> impl Fn(Invoke<R>) -> bool + Send + Sync + 'static {
    tauri::generate_handler![
        library::library_list_projects,
        library::library_create_project,
        library::library_rename_project,
        library::library_duplicate_project,
        library::library_delete_project,
        library::library_list_animations,
        library::library_create_animation,
        library::library_open_document,
        library::library_save_document,
        library::library_rename_animation,
        library::library_move_animation,
        library::library_duplicate_animation,
        library::library_delete_animation,
        library::library_usage,
        export::export_save_files,
        settings::settings_get,
        settings::settings_set,
        settings::settings_pick_library_folder,
        platform::platform_info,
    ]
}
