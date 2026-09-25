//! Builds the player every WebAssembly export starts with, from the sources of this commit, with
//! the function `cargo xtask build-player` runs: the compiler embeds the very module that command
//! checks against `crates/player/player.sha256`.
//!
//! The build goes into `$OUT_DIR/player-target`, away from the outer build's target directory.
//! `build_player` removes from the command's environment what the outer build would otherwise
//! pass on — its compiler wrappers (`clippy-driver` under `cargo clippy`), its flags, its target
//! and its target directory — and replaces `CARGO_ENCODED_RUSTFLAGS` with its own, so that the
//! module is the same whatever builds the compiler, for whichever target.

use std::path::{Path, PathBuf};

include!("../../xtask/src/player_build.rs");

/// What the player's bytes depend on, relative to the workspace root.
const PLAYER_INPUTS: [&str; 5] = [
    "crates/player",
    "crates/format",
    "xtask/src/player_build.rs",
    "Cargo.toml",
    "Cargo.lock",
];

/// Where the player is built, inside `OUT_DIR`.
const PLAYER_TARGET: &str = "player-target";

/// The variable through which `src/player.rs` finds the module to embed.
const PLAYER_WASM_VARIABLE: &str = "LIFE_PIXEL_PLAYER_WASM";

fn main() -> std::io::Result<()> {
    let root = workspace_root()?;
    for input in PLAYER_INPUTS {
        println!("cargo::rerun-if-changed={}", root.join(input).display());
    }
    let out_dir = std::env::var_os("OUT_DIR")
        .map(PathBuf::from)
        .ok_or_else(|| std::io::Error::other("cargo sets no OUT_DIR"))?;
    let module = build_player(root, &out_dir.join(PLAYER_TARGET))?;
    println!(
        "cargo::rustc-env={PLAYER_WASM_VARIABLE}={}",
        module.display()
    );
    Ok(())
}

/// The workspace root, two levels above this crate: the path `cargo metadata` gives
/// `cargo xtask build-player`, so that both remap the same prefix.
fn workspace_root() -> std::io::Result<&'static Path> {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    manifest_dir
        .parent()
        .and_then(Path::parent)
        .ok_or_else(|| std::io::Error::other("the compiler is not in crates/ of a workspace"))
}
