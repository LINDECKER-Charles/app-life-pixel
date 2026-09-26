use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use anyhow::{Context, ensure};
use cargo_metadata::MetadataCommand;

/// The CLI's package.
const PACKAGE: &str = "life-pixel-cli";
/// The binary it builds, and the sidecar's name before its triple.
const BINARY: &str = "life-pixel";
/// Where the sidecars go, relative to the workspace root, ignored by Git: `bundle.conf.json`'s
/// `externalBin` names `binaries/life-pixel`, which Tauri completes with `-<triple>`.
const SIDECAR_DIRECTORY: &str = "tauri/binaries";
/// The macOS target made of both architectures, joined with `lipo`.
const UNIVERSAL_MACOS: &str = "universal-apple-darwin";
/// The architectures of [`UNIVERSAL_MACOS`].
const MACOS_ARCHITECTURES: [&str; 2] = ["aarch64-apple-darwin", "x86_64-apple-darwin"];
/// What starts the host's line in `rustc -vV`.
const HOST_PREFIX: &str = "host: ";

/// Builds the `life-pixel` CLI, crates/cli, in release, as the desktop app's sidecar.
///
/// It builds for the host's target triple, or the one given, and copies the binary to
/// tauri/binaries/life-pixel-<triple> (.exe on Windows), where `cargo tauri build --config
/// bundle.conf.json` finds it. `universal-apple-darwin` builds both macOS architectures and joins
/// them with `lipo`. A target other than the host's needs `rustup target add <triple>`.
#[derive(clap::Args)]
pub struct BuildSidecar {
    /// The target triple; the host's by default.
    #[arg(long)]
    target: Option<String>,
}

impl BuildSidecar {
    /// The sidecar of the host, which `build-desktop` bundles.
    pub fn for_host() -> Self {
        Self { target: None }
    }

    /// Builds the CLI for each architecture of the triple, then writes the sidecar.
    pub fn run(self) -> anyhow::Result<()> {
        let metadata = MetadataCommand::new()
            .no_deps()
            .exec()
            .context("running cargo metadata")?;
        let root = metadata.workspace_root.as_std_path();
        let host = host_triple(root)?;
        let triple = self.target.unwrap_or_else(|| host.clone());
        let target_directory = metadata.target_directory.as_std_path();
        let mut binaries = Vec::new();
        for architecture in architectures(&triple) {
            build(root, architecture, &host)?;
            binaries.push(binary_path(target_directory, architecture, &host));
        }
        let sidecar = root.join(SIDECAR_DIRECTORY).join(sidecar_name(&triple));
        fs::create_dir_all(root.join(SIDECAR_DIRECTORY))?;
        write_sidecar(&binaries, &sidecar)?;
        println!("build-sidecar: {}", sidecar.display());
        Ok(())
    }
}

/// The host's target triple, as the workspace's toolchain names it.
fn host_triple(root: &Path) -> anyhow::Result<String> {
    let output = Command::new("rustc")
        .arg("-vV")
        .current_dir(root)
        .output()
        .context("running rustc -vV")?;
    ensure!(output.status.success(), "rustc -vV failed: {}", output.status);
    let text = String::from_utf8(output.stdout).context("reading rustc -vV")?;
    parse_host(&text).context("rustc -vV names no host")
}

/// The triple of the `host:` line of `rustc -vV`.
fn parse_host(text: &str) -> Option<String> {
    let line = text.lines().find_map(|line| line.strip_prefix(HOST_PREFIX))?;
    Some(line.trim().to_owned())
}

/// The triples to build for `triple`: both macOS architectures for the universal one.
fn architectures(triple: &str) -> Vec<&str> {
    if triple == UNIVERSAL_MACOS {
        MACOS_ARCHITECTURES.to_vec()
    } else {
        vec![triple]
    }
}

/// Builds the CLI in release for `triple`.
fn build(root: &Path, triple: &str, host: &str) -> anyhow::Result<()> {
    let arguments = cargo_arguments(triple, host);
    println!("build-sidecar: cargo {}", arguments.join(" "));
    let status = Command::new(env!("CARGO"))
        .args(&arguments)
        .current_dir(root)
        .status()
        .context("running cargo build")?;
    ensure!(
        status.success(),
        "building {PACKAGE} for {triple} failed: {status} (is the target installed? \
         `rustup target add {triple}`)"
    );
    Ok(())
}

/// The arguments of `cargo` that build the CLI for `triple`: the host's build shares
/// `target/release` with the workspace's other release builds.
fn cargo_arguments(triple: &str, host: &str) -> Vec<String> {
    let mut arguments: Vec<String> = ["build", "--release", "--locked", "--package", PACKAGE]
        .map(str::to_owned)
        .to_vec();
    if triple != host {
        arguments.extend(["--target".to_owned(), triple.to_owned()]);
    }
    arguments
}

/// Where Cargo puts the CLI built for `triple`.
fn binary_path(target_directory: &Path, triple: &str, host: &str) -> PathBuf {
    let profile = if triple == host {
        target_directory.join("release")
    } else {
        target_directory.join(triple).join("release")
    };
    profile.join(format!("{BINARY}{}", executable_suffix(triple)))
}

/// The sidecar's file name for `triple`, as Tauri looks it up.
fn sidecar_name(triple: &str) -> String {
    format!("{BINARY}-{triple}{}", executable_suffix(triple))
}

/// `.exe` for a Windows triple, nothing otherwise.
fn executable_suffix(triple: &str) -> &'static str {
    if triple.contains("windows") {
        ".exe"
    } else {
        ""
    }
}

/// Copies the one binary to `sidecar`, or joins several with `lipo`.
fn write_sidecar(binaries: &[PathBuf], sidecar: &Path) -> anyhow::Result<()> {
    if let [binary] = binaries {
        fs::copy(binary, sidecar).with_context(|| format!("copying {}", binary.display()))?;
        return Ok(());
    }
    let status = Command::new("lipo")
        .arg("-create")
        .arg("-output")
        .arg(sidecar)
        .args(binaries)
        .status()
        .context("running lipo")?;
    ensure!(status.success(), "lipo failed: {status}");
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_sidecar_is_named_for_its_triple() {
        assert_eq!(
            sidecar_name("aarch64-apple-darwin"),
            "life-pixel-aarch64-apple-darwin"
        );
        assert_eq!(
            sidecar_name("x86_64-unknown-linux-gnu"),
            "life-pixel-x86_64-unknown-linux-gnu"
        );
        assert_eq!(
            sidecar_name("x86_64-pc-windows-msvc"),
            "life-pixel-x86_64-pc-windows-msvc.exe"
        );
    }

    #[test]
    fn the_host_is_read_from_rustc() {
        let text = "rustc 1.98.0 (abc 2026-08-01)\nbinary: rustc\nhost: aarch64-apple-darwin\n";
        assert_eq!(parse_host(text).as_deref(), Some("aarch64-apple-darwin"));
        assert_eq!(parse_host("rustc 1.98.0"), None);
    }

    #[test]
    fn the_host_builds_into_target_release_another_triple_into_its_own_folder() {
        let host = "aarch64-apple-darwin";
        let target = Path::new("target");
        assert!(!cargo_arguments(host, host).contains(&"--target".to_owned()));
        assert_eq!(
            binary_path(target, host, host),
            target.join("release").join("life-pixel")
        );
        let windows = "x86_64-pc-windows-msvc";
        let arguments = cargo_arguments(windows, host);
        assert_eq!(arguments[arguments.len() - 2..], ["--target", windows]);
        let expected = target.join(windows).join("release").join("life-pixel.exe");
        assert_eq!(binary_path(target, windows, host), expected);
    }

    #[test]
    fn the_universal_macos_sidecar_joins_both_architectures() {
        assert_eq!(architectures(UNIVERSAL_MACOS), MACOS_ARCHITECTURES);
        assert_eq!(
            architectures("x86_64-unknown-linux-gnu"),
            ["x86_64-unknown-linux-gnu"]
        );
    }
}
