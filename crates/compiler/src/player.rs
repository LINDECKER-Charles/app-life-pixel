//! What an export ships besides the animation: the player module and its loader, both built from
//! the sources of this commit.

/// The player: the WebAssembly module every export starts with, built by this crate's build script
/// with the function `cargo xtask build-player` runs, so that it hashes to
/// `crates/player/player.sha256`. It imports nothing and plays the payload appended to it.
pub const PLAYER_WASM: &[u8] = include_bytes!(env!("LIFE_PIXEL_PLAYER_WASM"));

/// The loader, `player-js/life-pixel.js`: the committed build of `@life-pixel/player`, an ES
/// module that defines `<life-pixel>`. One copy serves every export of a site.
pub const LOADER_JS: &str = include_str!("../../../player-js/life-pixel.js");
