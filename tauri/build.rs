//! Declares the app's commands, so that each gets an `allow-<command>` permission that
//! `capabilities/default.json` grants to the main window, and nothing else can call them.

/// Every command of `src/commands/`, as `generate_handler!` registers it.
const COMMANDS: &[&str] = &[
    "library_list_projects",
    "library_create_project",
    "library_rename_project",
    "library_duplicate_project",
    "library_delete_project",
    "library_list_animations",
    "library_create_animation",
    "library_open_document",
    "library_save_document",
    "library_rename_animation",
    "library_move_animation",
    "library_duplicate_animation",
    "library_delete_animation",
    "library_usage",
    "export_save_files",
    "settings_get",
    "settings_set",
    "settings_pick_library_folder",
    "platform_info",
];

fn main() {
    let manifest = tauri_build::AppManifest::new().commands(COMMANDS);
    let attributes = tauri_build::Attributes::new().app_manifest(manifest);
    if let Err(error) = tauri_build::try_build(attributes) {
        panic!("tauri-build failed: {error:#}");
    }
}
