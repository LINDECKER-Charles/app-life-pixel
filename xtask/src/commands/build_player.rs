use std::fs;
use std::path::Path;

use anyhow::{Context, bail, ensure};
use cargo_metadata::MetadataCommand;
use sha2::{Digest, Sha256};

mod fixture;
mod play_fixture;
mod wasm_player;

include!("../player_build.rs");

/// The player's size budget, from docs/export.md: 16 KiB, without payload.
const BUDGET_BYTES: usize = 16 * 1024;

/// The hash every change of the player or the format updates, relative to the workspace root.
const HASH_FILE: &str = "crates/player/player.sha256";

/// The module's file name, as the hash file names it.
const MODULE_NAME: &str = "life_pixel_player.wasm";

/// Where the player is built, inside the workspace's target directory.
const TARGET_SUBDIRECTORY: &str = "player";

/// Builds the WebAssembly player, reproducibly, into target/player/.
///
/// With `--check`, it also checks that the module hashes to crates/player/player.sha256, fits
/// the 16 KiB budget, imports nothing, and plays every frame of the v1 fixture — run in wasmi —
/// with the colours its expected indices give through the palette.
#[derive(clap::Args)]
pub struct BuildPlayer {
    /// Check the hash, the size budget and the frames the player plays from the v1 fixture.
    #[arg(long)]
    check: bool,
    /// Write the module's hash into crates/player/player.sha256, before any check: every change
    /// of the player or the format commits it.
    #[arg(long)]
    write_hash: bool,
}

impl BuildPlayer {
    /// Builds the player, then writes its hash and runs the checks it was asked for.
    pub fn run(self) -> anyhow::Result<()> {
        let metadata = MetadataCommand::new()
            .no_deps()
            .exec()
            .context("running cargo metadata")?;
        let root = metadata.workspace_root.as_std_path();
        let target_dir = metadata.target_directory.join(TARGET_SUBDIRECTORY);
        let path = build_player(root, target_dir.as_std_path()).context("building the player")?;
        let module = fs::read(&path).with_context(|| format!("reading {}", path.display()))?;
        let hash = sha256(&module);
        println!("build-player: {}", path.display());
        println!("build-player: {} bytes, sha256 {hash}", module.len());
        if self.write_hash {
            write_hash(root, &hash)?;
        }
        if self.check {
            check(root, &module, &hash)?;
        }
        Ok(())
    }
}

fn sha256(bytes: &[u8]) -> String {
    let digest = Sha256::digest(bytes);
    digest.iter().map(|byte| format!("{byte:02x}")).collect()
}

/// Writes `hash` in the format of `sha256sum`, so that `sha256sum -c` checks it from the
/// module's directory.
fn write_hash(root: &Path, hash: &str) -> anyhow::Result<()> {
    fs::write(root.join(HASH_FILE), format!("{hash}  {MODULE_NAME}\n"))
        .with_context(|| format!("writing {HASH_FILE}"))?;
    println!("build-player: wrote {HASH_FILE}");
    Ok(())
}

/// Runs every check and reports each one, then fails if any did.
fn check(root: &Path, module: &[u8], hash: &str) -> anyhow::Result<()> {
    let checks = [
        ("hash", check_hash(root, hash)),
        ("size budget", check_budget(module.len())),
        ("v1 fixture", play_fixture::check(root, module)),
    ];
    let mut failures = 0;
    for (name, result) in checks {
        match result {
            Ok(report) => println!("build-player: {name}: {report}"),
            Err(error) => {
                failures += 1;
                eprintln!("build-player: {name} FAILED: {error:#}");
            }
        }
    }
    ensure!(failures == 0, "{failures} player check(s) failed");
    Ok(())
}

fn check_hash(root: &Path, hash: &str) -> anyhow::Result<String> {
    let file = fs::read_to_string(root.join(HASH_FILE))
        .with_context(|| format!("reading {HASH_FILE}"))?;
    let expected = file.split_whitespace().next().unwrap_or_default();
    if expected != hash {
        bail!(
            "the module hashes to {hash}, {HASH_FILE} says {expected}: when the player or the \
             format changed, run `cargo xtask build-player --write-hash` and commit the file"
        );
    }
    Ok(format!("matches {HASH_FILE}"))
}

fn check_budget(size: usize) -> anyhow::Result<String> {
    ensure!(
        size <= BUDGET_BYTES,
        "{size} bytes, over the budget of {BUDGET_BYTES}: raising it is a design discussion"
    );
    Ok(format!("{size} of {BUDGET_BYTES} bytes"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_hash_is_lowercase_hexadecimal_sha256() {
        assert_eq!(
            sha256(b"abc"),
            "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad"
        );
    }

    #[test]
    fn the_budget_is_16_kib_included() {
        assert!(check_budget(16_384).is_ok());
        assert!(check_budget(16_385).is_err());
    }
}
