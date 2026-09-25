//! Turns a Life Pixel animation into files.
//!
//! [`wasm_export`] writes the self-contained WebAssembly bundle: the [`player`] built from the same
//! commit, with the animation's payload appended as a custom section — data, never code.
//! [`classic`] writes the formats other tools already read — GIF, APNG, a sprite sheet with its
//! JSON, a zip of PNG frames —, [`naming`] names them after the animation's title. Every pixel
//! comes from `life-pixel-core`'s rendering, so an export shows what the editor shows. The crate is
//! pure: no I/O, no clock, no randomness, so the same animation always gives the same bytes.

pub mod classic;
pub mod error;
pub mod export_file;
pub mod export_format;
pub mod naming;
pub mod player;
pub mod wasm_export;

pub use classic::{
    ClassicOptions, export_apng, export_gif, export_png_frames, export_sprite_sheet,
};
pub use error::ExportError;
pub use export_file::ExportFile;
pub use export_format::ExportFormat;
pub use naming::file_stem;
pub use player::{LOADER_JS, PLAYER_WASM};
pub use wasm_export::export_wasm;
