//! What every test of the engine starts from: requests written as the interface sends them, and
//! an engine with an animation open.

#![allow(
    clippy::unwrap_used,
    reason = "helpers of tests may unwrap too: a panic is a failed test"
)]
#![allow(dead_code, reason = "each test file uses the helpers it needs")]

use life_pixel_core::edit::Operation;
use life_pixel_editor_wasm::EngineCore;
use life_pixel_editor_wasm::export::ExportRequest;
use life_pixel_editor_wasm::new_animation::NewAnimationOptions;
use life_pixel_editor_wasm::render::RenderRequest;
use life_pixel_editor_wasm::state::EngineState;
use serde::de::DeserializeOwned;
use serde_json::{Value, json};

/// The id of a new animation's layer.
pub const LAYER: u32 = 1;
/// The id of a new animation's frame.
pub const FRAME: u32 = 2;
/// The title of the animation every test creates.
pub const TITLE: &str = "Engine Test";
/// The canvas side of the animation every test creates.
pub const SIDE: u32 = 4;

/// A request written as JSON, in the shape the interface gives it.
pub fn request<T: DeserializeOwned>(value: Value) -> T {
    serde_json::from_value(value).unwrap()
}

/// The interface's `NewAnimationOptions` of the test animation.
pub fn new_animation() -> NewAnimationOptions {
    request(json!({ "title": TITLE, "width": SIDE, "height": SIDE, "layerName": "Base" }))
}

/// An engine with the test animation open.
pub fn engine() -> EngineCore {
    let mut engine = EngineCore::default();
    engine.create(new_animation()).unwrap();
    engine
}

/// An operation written as JSON, in the shape of the interface's `EditOperation`.
pub fn operation(value: Value) -> Operation {
    request(value)
}

/// A new layer at the bottom of the stack.
pub fn add_layer() -> Operation {
    operation(json!({ "kind": "addLayer", "position": 0, "name": "Detail" }))
}

/// A one-pixel stroke of palette entry `index` at (`x`, `y`) on the test layer and frame.
pub fn dot(x: i32, y: i32, index: u32) -> Operation {
    operation(json!({
        "kind": "paintStroke", "layer": LAYER, "frame": FRAME,
        "points": [{ "x": x, "y": y }], "index": index,
    }))
}

/// A render request of the test frame, with an optional preview.
pub fn render(preview: Option<Operation>) -> RenderRequest {
    RenderRequest {
        frame: life_pixel_core::FrameId::new(FRAME),
        preview,
    }
}

/// An export request of `format`, as the interface names it.
pub fn export(format: &str) -> ExportRequest {
    request(json!({ "format": format }))
}

/// The number of layers the engine's state shows.
pub fn layer_count(state: &EngineState) -> usize {
    state.document.as_ref().unwrap().layers.len()
}
