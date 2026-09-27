use std::path::Path;
use std::process::Command;

use anyhow::{Context, bail, ensure};
use cargo_metadata::MetadataCommand;

use super::build_sidecar::BuildSidecar;

/// The Tauri project, relative to the workspace root.
const TAURI_DIR: &str = "tauri";

/// The overlay that bundles the sidecar, relative to the Tauri project.
const BUNDLE_CONFIG: &str = "bundle.conf.json";

/// What `npm ci` leaves in the front-end, which `beforeBuildCommand` builds.
const FRONTEND_MODULES: &str = "frontend/node_modules";

/// A catalogue the built app must serve at `/i18n/`, relative to the workspace root.
const BUILT_CATALOGUE: &str = "frontend/dist/app/browser/i18n/en.json";

/// How to install the Tauri CLI when it is missing.
const INSTALL_TAURI_CLI: &str = "cargo install tauri-cli --version ^2 --locked";

/// Builds the desktop app with the Tauri CLI: `build-sidecar` for the host first, then
/// `cargo tauri build --config bundle.conf.json`, which builds the front-end, then bundles it with
/// the `life-pixel` CLI for the host — the macOS `.app`, a Linux `.deb`, nothing on Windows.
/// Never a DMG, whose creation drives the Finder, nor an installer that downloads its tools.
///
/// Needs the Tauri CLI and `npm ci --prefix frontend`; no signing key: release builds are
/// release.yml's.
#[derive(clap::Args)]
pub struct BuildDesktop {
    /// A debug build: quicker, with the webview's inspector, and `LP_EXPORT_DIR` honoured.
    #[arg(long)]
    debug: bool,
}

impl BuildDesktop {
    /// Checks the prerequisites, builds the sidecar, runs `cargo tauri build`, then checks the
    /// catalogues were built.
    pub fn run(self) -> anyhow::Result<()> {
        let metadata = MetadataCommand::new()
            .no_deps()
            .exec()
            .context("running cargo metadata")?;
        let root = metadata.workspace_root.as_std_path();
        check_prerequisites(root)?;
        BuildSidecar::for_host().run()?;
        let arguments = tauri_arguments(self.debug, std::env::consts::OS);
        println!("build-desktop: cargo {}", arguments.join(" "));
        let status = Command::new(env!("CARGO"))
            .args(&arguments)
            .current_dir(root.join(TAURI_DIR))
            .status()
            .context("running cargo tauri build")?;
        ensure!(status.success(), "cargo tauri build failed: {status}");
        ensure!(
            root.join(BUILT_CATALOGUE).is_file(),
            "{BUILT_CATALOGUE} is missing: the app would have no catalogue at /i18n/"
        );
        let profile = if self.debug { "debug" } else { "release" };
        let bundles = metadata.target_directory.join(profile).join("bundle");
        println!("build-desktop: built into {bundles}");
        Ok(())
    }
}

/// Fails with what to run when the Tauri CLI or the front-end's packages are missing.
fn check_prerequisites(root: &Path) -> anyhow::Result<()> {
    let version = Command::new(env!("CARGO")).args(["tauri", "--version"]).output();
    if !version.is_ok_and(|output| output.status.success()) {
        bail!("the Tauri CLI is missing: run `{INSTALL_TAURI_CLI}`");
    }
    if !root.join(FRONTEND_MODULES).is_dir() {
        bail!("the front-end's packages are missing: run `npm ci --prefix frontend`");
    }
    Ok(())
}

/// The arguments of `cargo` that build the app on `os`, as `std::env::consts::OS` names it.
fn tauri_arguments(debug: bool, os: &str) -> Vec<&'static str> {
    let mut arguments = vec!["tauri", "build", "--ci", "--config", BUNDLE_CONFIG];
    if debug {
        arguments.push("--debug");
    }
    match host_bundle(os) {
        Some(bundle) => arguments.extend(["--bundles", bundle]),
        None => arguments.push("--no-bundle"),
    }
    arguments
}

/// The bundle a build makes on `os`: one needing no dialog and no download.
fn host_bundle(os: &str) -> Option<&'static str> {
    match os {
        "macos" => Some("app"),
        "linux" => Some("deb"),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_debug_build_bundles_the_macos_app_never_a_dmg() {
        let arguments = tauri_arguments(true, "macos");
        assert_eq!(
            arguments,
            [
                "tauri",
                "build",
                "--ci",
                "--config",
                "bundle.conf.json",
                "--debug",
                "--bundles",
                "app"
            ]
        );
    }

    #[test]
    fn linux_bundles_a_deb_and_windows_nothing() {
        assert!(tauri_arguments(false, "linux").ends_with(&["--bundles", "deb"]));
        assert!(tauri_arguments(false, "windows").ends_with(&["--no-bundle"]));
    }

    #[test]
    fn every_build_bundles_the_sidecar() {
        for os in ["macos", "linux", "windows"] {
            let arguments = tauri_arguments(false, os);
            assert!(arguments.windows(2).any(|pair| pair == ["--config", BUNDLE_CONFIG]));
        }
    }
}
