//! The editor's engine: `core` and `compiler` compiled to WebAssembly for the editor's Web Worker.
//!
//! [`EngineCore`] is the engine as plain Rust, tested on the host: one animation, its history,
//! and every method of the editor's `EditorEngine` interface — editing through `core::edit`,
//! rendering through `core::render`, exports and snippets through `compiler`. [`bindings`] is its
//! `#[wasm_bindgen]` face, which `cargo xtask build-editor` turns into the module the worker
//! loads. [`errors`] gives every refusal as `{ code, params }`. No request panics: every value from
//! the interface is validated into an error first.

pub mod bindings;
pub mod engine_core;
pub mod errors;
pub mod export;
pub mod new_animation;
pub mod render;
pub mod snippet;
pub mod state;

pub use engine_core::EngineCore;
pub use errors::EngineError;
