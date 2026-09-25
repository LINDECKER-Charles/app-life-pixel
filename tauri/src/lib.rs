//! The Life Pixel desktop app: `projects/app` in a Tauri 2 window, with `service` in-process on a
//! local library folder. It works offline, without an account, and sends nothing.

pub mod commands;
pub mod dialogs;
pub mod errors;
pub mod settings;
pub mod state;

use std::path::PathBuf;
use std::sync::Arc;

use life_pixel_service::local::default_library_path;
use tauri::{App, Context, Manager, Runtime};
use tracing_subscriber::EnvFilter;

use crate::dialogs::SystemDialogs;
use crate::settings::SETTINGS_FILE;
use crate::state::{DesktopState, StateOptions};

/// The library folder of this run, whatever the settings say — as for the CLI.
const LIBRARY_VARIABLE: &str = "LIFE_PIXEL_LIBRARY";
/// Where a debug build saves exports, without a dialog: for the end-to-end tests.
const EXPORT_DIR_VARIABLE: &str = "LP_EXPORT_DIR";
/// What the log keeps when `RUST_LOG` says nothing.
const DEFAULT_LOG_FILTER: &str = "warn";

/// Starts the app: the log, the dialog plugin, the state, the commands, the window.
pub fn run() {
    init_log();
    let started = tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .setup(|app| {
            let state = DesktopState::open(state_options(app)?, dialogs(app));
            app.manage(state);
            Ok(())
        })
        .invoke_handler(commands::handler())
        .run(context());
    if let Err(error) = started {
        tracing::error!(%error, "the app stopped");
        std::process::exit(1);
    }
}

/// The app's configuration and capabilities, compiled from `tauri.conf.json` and
/// `capabilities/`, for any runtime: the app's, and the tests' mock one.
#[must_use]
pub fn context<R: Runtime>() -> Context<R> {
    tauri::generate_context!()
}

/// The log on stderr, filtered by `RUST_LOG`.
fn init_log() {
    let filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| DEFAULT_LOG_FILTER.into());
    let subscriber = tracing_subscriber::fmt()
        .with_env_filter(filter)
        .with_writer(std::io::stderr);
    // Fails only when a subscriber is already set, which then keeps logging.
    drop(subscriber.try_init());
}

/// The state's files: the settings in the app's configuration folder, the default library in the
/// documents folder, and the variables of this run.
fn state_options(app: &App) -> tauri::Result<StateOptions> {
    let paths = app.path();
    let default_library = default_library_path(paths.document_dir().ok(), paths.home_dir()?);
    Ok(StateOptions {
        settings_file: paths.app_config_dir()?.join(SETTINGS_FILE),
        default_library,
        library_override: variable_path(LIBRARY_VARIABLE),
        export_dir: variable_path(EXPORT_DIR_VARIABLE).filter(|_| cfg!(debug_assertions)),
    })
}

/// The system's dialogs, attached to the app.
fn dialogs(app: &App) -> Arc<SystemDialogs<tauri::Wry>> {
    Arc::new(SystemDialogs::new(app.handle().clone()))
}

/// The path the environment variable `name` holds, when set and not empty.
fn variable_path(name: &str) -> Option<PathBuf> {
    std::env::var_os(name)
        .filter(|value| !value.is_empty())
        .map(PathBuf::from)
}
