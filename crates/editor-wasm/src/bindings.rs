//! The `#[wasm_bindgen]` face of [`EngineCore`]: the class `EngineCore` of the generated module,
//! one method per method of the interface. Values cross as JavaScript values through
//! `serde-wasm-bindgen` — objects, never `Map`s, and `null` for an absent value, as the interface's
//! types say —, bytes as `Uint8Array`. A refused request throws its `{ code, params }`.

use serde::Serialize;
use serde::de::DeserializeOwned;
use serde_wasm_bindgen::Serializer;
use wasm_bindgen::prelude::wasm_bindgen;
use wasm_bindgen::{JsError, JsValue};

use crate::engine_core::EngineCore;
use crate::errors::EngineError;

/// How answers cross: objects rather than `Map`s, `null` for an absent value, `Uint8Array` for
/// bytes — `Serializer::json_compatible` would turn bytes into arrays of numbers.
const SERIALIZER: Serializer = Serializer::new()
    .serialize_missing_as_null(true)
    .serialize_maps_as_objects(true);

/// The engine the worker owns, `EngineCore` in JavaScript.
#[wasm_bindgen(js_name = EngineCore)]
#[derive(Debug, Default)]
pub struct EngineBindings {
    core: EngineCore,
}

#[wasm_bindgen(js_class = EngineCore)]
impl EngineBindings {
    /// An engine without an animation.
    #[wasm_bindgen(constructor)]
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// [`EngineCore::create`], from a `NewAnimationOptions`.
    ///
    /// # Errors
    ///
    /// The `{ code, params }` of the refusal.
    pub fn create(
        &mut self,
        #[wasm_bindgen(unchecked_param_type = "unknown")] options: JsValue,
    ) -> Result<(), JsValue> {
        self.core.create(from_js(options)?).map_err(to_js_error)
    }

    /// [`EngineCore::open`].
    ///
    /// # Errors
    ///
    /// The `{ code, params }` of the refusal.
    pub fn open(&mut self, document: &[u8]) -> Result<(), JsValue> {
        self.core.open(document).map_err(to_js_error)
    }

    /// [`EngineCore::restore`].
    ///
    /// # Errors
    ///
    /// The `{ code, params }` of the refusal.
    pub fn restore(&mut self, document: &[u8]) -> Result<(), JsValue> {
        self.core.restore(document).map_err(to_js_error)
    }

    /// [`EngineCore::apply`], from an `EditOperation`.
    ///
    /// # Errors
    ///
    /// The `{ code, params }` of the refusal.
    pub fn apply(
        &mut self,
        #[wasm_bindgen(unchecked_param_type = "unknown")] operation: JsValue,
    ) -> Result<(), JsValue> {
        self.core.apply(from_js(operation)?).map_err(to_js_error)
    }

    /// [`EngineCore::undo`]: whether there was a step to undo.
    pub fn undo(&mut self) -> bool {
        self.core.undo()
    }

    /// [`EngineCore::redo`]: whether there was a step to redo.
    pub fn redo(&mut self) -> bool {
        self.core.redo()
    }

    /// [`EngineCore::mark_saved`].
    #[wasm_bindgen(js_name = markSaved)]
    pub fn mark_saved(&mut self) {
        self.core.mark_saved();
    }

    /// [`EngineCore::state`], as an `EngineState`.
    ///
    /// # Errors
    ///
    /// `internal.error` when the state cannot cross, which only a bug causes.
    #[wasm_bindgen(unchecked_return_type = "unknown")]
    pub fn state(&self) -> Result<JsValue, JsValue> {
        to_js(&self.core.state())
    }

    /// [`EngineCore::render`], from a `RenderRequest`, as a `{ width, height, pixels }` whose
    /// pixels are a `Uint8Array`.
    ///
    /// # Errors
    ///
    /// The `{ code, params }` of the refusal.
    #[wasm_bindgen(unchecked_return_type = "unknown")]
    pub fn render(
        &self,
        #[wasm_bindgen(unchecked_param_type = "unknown")] request: JsValue,
    ) -> Result<JsValue, JsValue> {
        to_js(&self.core.render(&from_js(request)?).map_err(to_js_error)?)
    }

    /// [`EngineCore::serialize`].
    ///
    /// # Errors
    ///
    /// The `{ code, params }` of the refusal.
    pub fn serialize(&self) -> Result<Vec<u8>, JsValue> {
        self.core.serialize().map_err(to_js_error)
    }

    /// [`EngineCore::export`], from an `ExportRequest`, as an `ExportResult`.
    ///
    /// # Errors
    ///
    /// The `{ code, params }` of the refusal.
    #[wasm_bindgen(unchecked_return_type = "unknown")]
    pub fn export(
        &self,
        #[wasm_bindgen(unchecked_param_type = "unknown")] request: JsValue,
    ) -> Result<JsValue, JsValue> {
        to_js(&self.core.export(&from_js(request)?).map_err(to_js_error)?)
    }

    /// [`EngineCore::snippet`], from a `SnippetRequest`.
    ///
    /// # Errors
    ///
    /// The `{ code, params }` of the refusal.
    pub fn snippet(
        &self,
        #[wasm_bindgen(unchecked_param_type = "unknown")] request: JsValue,
    ) -> Result<String, JsValue> {
        self.core.snippet(&from_js(request)?).map_err(to_js_error)
    }

    /// [`EngineCore::limits`], as a `Limits`.
    ///
    /// # Errors
    ///
    /// `internal.error` when the limits cannot cross, which only a bug causes.
    #[wasm_bindgen(unchecked_return_type = "unknown")]
    pub fn limits() -> Result<JsValue, JsValue> {
        to_js(&EngineCore::limits())
    }
}

/// A request's value, or `request.malformed` when it lacks the interface's shape.
fn from_js<T: DeserializeOwned>(value: JsValue) -> Result<T, JsValue> {
    serde_wasm_bindgen::from_value(value).map_err(|_| to_js_error(EngineError::malformed_request()))
}

fn to_js<T: Serialize>(value: &T) -> Result<JsValue, JsValue> {
    value
        .serialize(&SERIALIZER)
        .map_err(|_| to_js_error(EngineError::internal()))
}

/// The error as `{ code, params }`; an `Error` naming its code in the impossible case where it
/// cannot cross.
fn to_js_error(error: EngineError) -> JsValue {
    error
        .serialize(&SERIALIZER)
        .unwrap_or_else(|_| JsError::new(error.code).into())
}
