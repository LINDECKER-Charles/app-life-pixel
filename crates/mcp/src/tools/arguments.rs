//! Reading a tool's arguments and writing its result: the schema of the arguments comes from
//! their type, bounds from `core::limits`, and a result is JSON, user content included as data.

use life_pixel_core::Rgba;
use life_pixel_service::CodedError;
use rmcp::model::{CallToolResult, ContentBlock, JsonObject};
use schemars::JsonSchema;
use schemars::generate::SchemaSettings;
use serde::Serialize;
use serde::de::DeserializeOwned;
use serde_json::Value;

use crate::errors::{malformed, unavailable};
use crate::server::LifePixelMcp;

/// The pattern of a colour: `#rrggbbaa`.
pub(crate) const COLOR_PATTERN: &str = "^#[0-9a-fA-F]{8}$";

/// The colours of `texts`, each `#rrggbbaa`; `document.palette` for one that is not.
pub(crate) fn colors(texts: &[String]) -> Result<Vec<Rgba>, CodedError> {
    let colors = texts.iter().map(|text| text.parse::<Rgba>());
    Ok(colors.collect::<Result<_, _>>()?)
}

/// The arguments of a call as `T`; `request.malformed` when they do not match its schema.
pub(crate) fn parse<T: DeserializeOwned>(arguments: JsonObject) -> Result<T, CodedError> {
    serde_json::from_value(Value::Object(arguments)).map_err(|_| malformed())
}

/// The key of a schema's title, which would name a Rust type.
const TITLE: &str = "title";

/// The JSON Schema of `T`, in the draft MCP names, as a tool's `inputSchema`, untitled.
pub(crate) fn schema_of<T: JsonSchema>() -> JsonObject {
    let generator = SchemaSettings::draft2020_12().into_generator();
    let schema = generator.into_root_schema_for::<T>();
    let mut schema = schema.as_object().cloned().unwrap_or_default();
    schema.remove(TITLE);
    schema
}

/// [`schema_of`], as a [`ToolDefinition`](super::ToolDefinition) asks for it.
pub(crate) fn schema_for<T: JsonSchema>(_server: &LifePixelMcp) -> JsonObject {
    schema_of::<T>()
}

/// A successful result holding `value` as its JSON text, and as structured content when it is
/// an object — the only structured content the protocol allows.
pub(crate) fn json_result<T: Serialize>(value: &T) -> Result<CallToolResult, CodedError> {
    let value = serde_json::to_value(value).map_err(|_| unavailable())?;
    if value.is_object() {
        return Ok(CallToolResult::structured(value));
    }
    Ok(CallToolResult::success(vec![ContentBlock::text(
        value.to_string(),
    )]))
}
