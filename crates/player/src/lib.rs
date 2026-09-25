//! The Life Pixel player: the WebAssembly module every export carries, which plays the payload
//! appended to it through player ABI v1. `crates/format/README.md` specifies both.
//!
//! On `wasm32` the crate is `no_std`: [`Player`] sits in a `static`, the exports of `abi` call it,
//! a bump allocator serves its memory, and a panic traps. The module imports nothing. On the host
//! the crate keeps `std`, so that its tests and the workspace's lints build there;
//! `cargo xtask build-player` builds the module, and its `--check` plays the v1 fixture in it.

#![cfg_attr(all(target_arch = "wasm32", not(test)), no_std)]

extern crate alloc;

#[cfg(all(target_arch = "wasm32", not(test)))]
mod abi;
#[cfg(all(target_arch = "wasm32", not(test)))]
mod allocator;
mod animation;
mod call_error;
mod frames;
mod memory;
mod playback;
mod player;
mod span;

pub use call_error::CallError;
pub use playback::{FRAME_CHANGED, RANGE_ENDED, RANGE_STOPPED};
pub use player::Player;

/// Traps: a panic never crosses the WebAssembly boundary, and the module imports nothing that
/// could report it. A checked payload never panics the player; this is its last guard.
#[cfg(all(target_arch = "wasm32", not(test)))]
#[panic_handler]
fn trap(_: &core::panic::PanicInfo<'_>) -> ! {
    core::arch::wasm32::unreachable()
}
