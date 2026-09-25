use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use anyhow::{Context, bail, ensure};
use cargo_metadata::{Metadata, MetadataCommand, Package};

/// The engine's package.
const PACKAGE: &str = "life-pixel-editor-wasm";
/// The module Cargo builds, in `target/wasm32-unknown-unknown/release/`.
const CARGO_MODULE: &str = "life_pixel_editor_wasm.wasm";
/// The target the engine runs on.
const TARGET: &str = "wasm32-unknown-unknown";
/// The crate whose version, in Cargo.lock, the command-line tool must have.
const BINDGEN_CRATE: &str = "wasm-bindgen";
/// The name of every file wasm-bindgen writes: `editor_engine.js`, `editor_engine_bg.wasm`…
const OUT_NAME: &str = "editor_engine";
/// Where wasm-bindgen writes, inside the workspace's target directory, before the copies.
const STAGING_SUBDIRECTORY: &str = "editor-engine";
/// The JavaScript glue and its types, which the worker imports: relative to the workspace root.
const GLUE_DIRECTORY: &str = "frontend/projects/app/src/app/engine/wasm/generated";
const GLUE_FILES: [&str; 2] = ["editor_engine.js", "editor_engine.d.ts"];
/// The module, which the worker fetches from `/engine/`: relative to the workspace root.
const MODULE_DIRECTORY: &str = "frontend/projects/app/public/engine";
const MODULE_FILE: &str = "editor_engine_bg.wasm";

/// Builds the editor's engine, crates/editor-wasm, for the app's Web Worker.
///
/// It checks that `wasm-bindgen --version` is the `wasm-bindgen` of Cargo.lock, builds the crate
/// for wasm32-unknown-unknown in release, then runs `wasm-bindgen --target web`: the JavaScript
/// glue goes to frontend/projects/app/src/app/engine/wasm/generated/, the module to
/// frontend/projects/app/public/engine/. Both are ignored by Git.
#[derive(clap::Args)]
pub struct BuildEditor {}

impl BuildEditor {
    /// Checks the tool, builds the module, and writes the glue and the module into the app.
    pub fn run(self) -> anyhow::Result<()> {
        let metadata = MetadataCommand::new()
            .other_options(["--locked".to_owned()])
            .exec()
            .context("running cargo metadata")?;
        let root = metadata.workspace_root.as_std_path();
        check_bindgen(&locked_bindgen_version(&metadata)?)?;
        let module = build_module(root, metadata.target_directory.as_std_path())?;
        let staging = metadata.target_directory.join(STAGING_SUBDIRECTORY);
        run_bindgen(&module, staging.as_std_path())?;
        copy_outputs(staging.as_std_path(), root)?;
        let size = fs::metadata(root.join(MODULE_DIRECTORY).join(MODULE_FILE))?.len();
        println!("build-editor: {MODULE_DIRECTORY}/{MODULE_FILE}, {size} bytes");
        println!("build-editor: {GLUE_DIRECTORY}/{}", GLUE_FILES.join(", "));
        Ok(())
    }
}

/// The version of `wasm-bindgen` the engine is built with, as Cargo.lock pins it.
fn locked_bindgen_version(metadata: &Metadata) -> anyhow::Result<String> {
    let engine = find_package(metadata, PACKAGE)?;
    let resolve = metadata.resolve.as_ref().context("cargo metadata resolved nothing")?;
    let node = resolve.nodes.iter().find(|node| node.id == engine.id);
    let node = node.with_context(|| format!("{PACKAGE} is not in the resolved graph"))?;
    let dependency = node.deps.iter().find(|dependency| {
        metadata[&dependency.pkg].name.as_str() == BINDGEN_CRATE
    });
    let dependency = dependency.with_context(|| format!("{PACKAGE} does not use {BINDGEN_CRATE}"))?;
    Ok(metadata[&dependency.pkg].version.to_string())
}

fn find_package<'metadata>(
    metadata: &'metadata Metadata,
    name: &str,
) -> anyhow::Result<&'metadata Package> {
    let mut packages = metadata.packages.iter();
    packages
        .find(|package| package.name.as_str() == name)
        .with_context(|| format!("no package {name} in the workspace"))
}

/// Fails, with the command that fixes it, unless `wasm-bindgen --version` prints `expected`: the
/// tool and the crate must agree on the schema of the custom section they share.
fn check_bindgen(expected: &str) -> anyhow::Result<()> {
    let install = format!("cargo install wasm-bindgen-cli --version {expected} --locked");
    let Ok(output) = Command::new("wasm-bindgen").arg("--version").output() else {
        bail!("wasm-bindgen is not installed: run `{install}`");
    };
    let text = String::from_utf8_lossy(&output.stdout);
    let found = printed_version(&text);
    ensure!(
        output.status.success() && found == expected,
        "wasm-bindgen {found} does not match the {BINDGEN_CRATE} {expected} of Cargo.lock: run \
         `{install}`"
    );
    println!("build-editor: wasm-bindgen {found}, as Cargo.lock pins it");
    Ok(())
}

/// The version in what `wasm-bindgen --version` prints: `wasm-bindgen 0.2.129`.
fn printed_version(text: &str) -> &str {
    text.split_whitespace().nth(1).unwrap_or_default()
}

/// Builds the engine in release for [`TARGET`], and returns the path of the module.
fn build_module(root: &Path, target_dir: &Path) -> anyhow::Result<PathBuf> {
    let cargo = std::env::var_os("CARGO").unwrap_or_else(|| "cargo".into());
    let status = Command::new(cargo)
        .current_dir(root)
        .args(["build", "--package", PACKAGE, "--lib", "--release", "--locked"])
        .args(["--target", TARGET])
        .status()
        .with_context(|| format!("running cargo build for {PACKAGE}"))?;
    ensure!(status.success(), "building {PACKAGE}: {status}");
    Ok(target_dir.join(TARGET).join("release").join(CARGO_MODULE))
}

/// Runs wasm-bindgen for a browser's ES module into an empty `staging` directory.
fn run_bindgen(module: &Path, staging: &Path) -> anyhow::Result<()> {
    remove_directory(staging)?;
    let status = Command::new("wasm-bindgen")
        .args(["--target", "web", "--out-name", OUT_NAME, "--out-dir"])
        .arg(staging)
        .arg(module)
        .status()
        .context("running wasm-bindgen")?;
    ensure!(status.success(), "wasm-bindgen: {status}");
    Ok(())
}

/// Replaces the app's glue and module with those of `staging`.
fn copy_outputs(staging: &Path, root: &Path) -> anyhow::Result<()> {
    let outputs = GLUE_FILES
        .iter()
        .map(|file| (GLUE_DIRECTORY, *file))
        .chain([(MODULE_DIRECTORY, MODULE_FILE)]);
    for directory in [GLUE_DIRECTORY, MODULE_DIRECTORY] {
        remove_directory(&root.join(directory))?;
        fs::create_dir_all(root.join(directory))
            .with_context(|| format!("creating {directory}"))?;
    }
    for (directory, file) in outputs {
        fs::copy(staging.join(file), root.join(directory).join(file))
            .with_context(|| format!("copying {file} to {directory}"))?;
    }
    Ok(())
}

fn remove_directory(directory: &Path) -> anyhow::Result<()> {
    match fs::remove_dir_all(directory) {
        Err(error) if error.kind() != std::io::ErrorKind::NotFound => {
            Err(error).with_context(|| format!("removing {}", directory.display()))
        }
        _ => Ok(()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_version_is_the_second_word_of_what_the_tool_prints() {
        assert_eq!(printed_version("wasm-bindgen 0.2.129\n"), "0.2.129");
        assert_eq!(printed_version(""), "");
    }
}
