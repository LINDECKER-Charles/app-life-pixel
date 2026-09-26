//! The signed updater (desktop.md, T4): registered only when the build received both the public
//! key it trusts and the endpoint it checks, so a local or CI build has no updater at all. Once
//! registered, it checks ten seconds after start, then every six hours, stays silent offline or on
//! any other failure, and never restarts the app on its own: it only offers "Install and restart"
//! in a native dialog, worded in the person's language.

use std::time::Duration;

use serde_json::{Map, Value};
use tauri::plugin::TauriPlugin;
use tauri::{AppHandle, Manager, Runtime};
use tauri_plugin_dialog::{DialogExt, MessageDialogButtons, MessageDialogKind};
use tauri_plugin_updater::{Update, Updater, UpdaterExt as _};
use tokio::sync::oneshot;
use url::Url;

use crate::state::DesktopState;

/// This build's public key, baked in by `release.yml` alone.
const PUBKEY: Option<&str> = option_env!("LP_UPDATER_PUBKEY");
/// This build's endpoint, the same way.
const ENDPOINT: Option<&str> = option_env!("LP_UPDATER_ENDPOINT");

/// How long after start the first check runs.
const FIRST_CHECK_DELAY: Duration = Duration::from_secs(10);
/// How long between checks after that.
const CHECK_INTERVAL: Duration = Duration::from_secs(6 * 60 * 60);

/// The front end's catalogues, embedded at compile time: the offer is a native dialog, with no
/// webview to fetch them from.
const CATALOGUE_EN: &str = include_str!("../../i18n/en.json");
const CATALOGUE_FR: &str = include_str!("../../i18n/fr.json");

/// Whether this build received both [`PUBKEY`] and [`ENDPOINT`].
#[must_use]
pub fn is_enabled() -> bool {
    PUBKEY.is_some() && ENDPOINT.is_some()
}

/// The plugin, when this build has an updater; `None` keeps `tauri-plugin-updater` out of the
/// app entirely — the CLI's page then says so.
#[must_use]
pub fn plugin<R: Runtime>() -> Option<TauriPlugin<R, tauri_plugin_updater::Config>> {
    is_enabled().then(|| tauri_plugin_updater::Builder::new().build())
}

/// Starts the background checks, when this build has an updater: the first one after
/// [`FIRST_CHECK_DELAY`], then every [`CHECK_INTERVAL`], for as long as the app runs.
pub fn start_checking<R: Runtime>(app: AppHandle<R>) {
    if !is_enabled() {
        return;
    }
    tauri::async_runtime::spawn(async move {
        tokio::time::sleep(FIRST_CHECK_DELAY).await;
        loop {
            check_once(&app).await;
            tokio::time::sleep(CHECK_INTERVAL).await;
        }
    });
}

/// One check: silent when offline, unreachable or otherwise failing, an offer only when an
/// update is found.
async fn check_once<R: Runtime>(app: &AppHandle<R>) {
    let Some(updater) = build_updater(app) else {
        return;
    };
    match updater.check().await {
        Ok(Some(update)) => offer_install(app, update).await,
        Ok(None) => {}
        Err(error) => tracing::debug!(%error, "update check failed, staying silent"),
    }
}

/// The updater for this run, built from [`PUBKEY`] and [`ENDPOINT`]; `None` when either is
/// missing or malformed, which never happens in a build `is_enabled` allowed to start checking.
fn build_updater<R: Runtime>(app: &AppHandle<R>) -> Option<Updater> {
    let (pubkey, endpoint) = (PUBKEY?, ENDPOINT?);
    let endpoint: Url = match endpoint.parse() {
        Ok(endpoint) => endpoint,
        Err(error) => {
            tracing::warn!(%error, "LP_UPDATER_ENDPOINT is not a URL");
            return None;
        }
    };
    let builder = match app
        .updater_builder()
        .pubkey(pubkey)
        .endpoints(vec![endpoint])
    {
        Ok(builder) => builder,
        Err(error) => {
            tracing::warn!(%error, "updater endpoint rejected");
            return None;
        }
    };
    match builder.build() {
        Ok(updater) => Some(updater),
        Err(error) => {
            tracing::warn!(%error, "updater not built");
            None
        }
    }
}

/// Shows the native "Install and restart" offer; installs and restarts only once it is accepted,
/// and never on its own otherwise.
async fn offer_install<R: Runtime>(app: &AppHandle<R>, update: Update) {
    let strings = Strings::for_language(language(app).await);
    let (sender, receiver) = oneshot::channel();
    app.dialog()
        .message(strings.message(&update.version))
        .title(strings.title)
        .kind(MessageDialogKind::Info)
        .buttons(MessageDialogButtons::OkCancelCustom(
            strings.install_and_restart,
            strings.later,
        ))
        .show(move |accepted| {
            let _ = sender.send(accepted);
        });
    if receiver.await != Ok(true) {
        return;
    }
    match update
        .download_and_install(|_chunk, _total| {}, || {})
        .await
    {
        Ok(()) => app.restart(),
        Err(error) => tracing::warn!(%error, "update not installed"),
    }
}

/// The settings' language, `"en"` until the person chooses one or it names neither catalogue.
async fn language<R: Runtime>(app: &AppHandle<R>) -> &'static str {
    let code = app.state::<DesktopState>().settings().await.language;
    match code.as_deref().and_then(|code| code.split('-').next()) {
        Some("fr") => "fr",
        _ => "en",
    }
}

/// The updater's strings, read from the catalogue of `language`.
struct Strings {
    /// `updater.title`.
    title: String,
    /// `updater.message`, with `{version}` still in it.
    message_template: String,
    /// `updater.install_and_restart`.
    install_and_restart: String,
    /// `updater.later`.
    later: String,
}

impl Strings {
    /// Reads the embedded catalogue of `language`, `en` covering every key.
    fn for_language(language: &str) -> Self {
        let text = if language == "fr" {
            CATALOGUE_FR
        } else {
            CATALOGUE_EN
        };
        let catalogue: Map<String, Value> = serde_json::from_str(text).unwrap_or_default();
        Self {
            title: string(&catalogue, "updater.title"),
            message_template: string(&catalogue, "updater.message"),
            install_and_restart: string(&catalogue, "updater.install_and_restart"),
            later: string(&catalogue, "updater.later"),
        }
    }

    /// The message with `{version}` replaced: a native dialog has no ICU engine, so the
    /// catalogue's placeholder is filled by hand, as `platform.version`'s already is in the app.
    fn message(&self, version: &str) -> String {
        self.message_template.replace("{version}", version)
    }
}

/// The catalogue's text at `key`, or `key` itself when the catalogue lacks it.
fn string(catalogue: &Map<String, Value>, key: &str) -> String {
    catalogue
        .get(key)
        .and_then(Value::as_str)
        .unwrap_or(key)
        .to_owned()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_build_without_the_two_variables_has_no_updater() {
        // Neither `LP_UPDATER_PUBKEY` nor `LP_UPDATER_ENDPOINT` is set for this build (`cargo
        // test` runs without them, as `cargo xtask build-desktop` does): only `release.yml` sets
        // both, so every other build carries no updater at all.
        assert!(!is_enabled());
        assert!(plugin::<tauri::Wry>().is_none());
    }

    #[test]
    fn every_updater_key_is_in_both_catalogues() {
        for text in [CATALOGUE_EN, CATALOGUE_FR] {
            let catalogue: Map<String, Value> = serde_json::from_str(text).unwrap();
            for key in [
                "updater.title",
                "updater.message",
                "updater.install_and_restart",
                "updater.later",
            ] {
                assert!(catalogue.contains_key(key), "{key} missing");
            }
        }
    }

    #[test]
    fn the_message_template_fills_its_placeholder() {
        let strings = Strings {
            title: "Update available".to_owned(),
            message_template: "Life Pixel {version} is ready.".to_owned(),
            install_and_restart: "Install and restart".to_owned(),
            later: "Later".to_owned(),
        };
        assert_eq!(strings.message("1.2.3"), "Life Pixel 1.2.3 is ready.");
    }
}
