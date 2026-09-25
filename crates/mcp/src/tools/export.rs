//! `export`: the files of an export, delivered where the transport puts them.

use life_pixel_compiler::ExportFormat;
use life_pixel_core::limits::{EXPORT_MAX_SCALE, EXPORT_MIN_SCALE, TAG_NAME_MAX_CHARS};
use life_pixel_service::animation::ExportRequest;
use life_pixel_service::{AnimationId, CodedError};
use rmcp::model::{CallToolResult, JsonObject};
use schemars::JsonSchema;
use serde::Deserialize;
use serde_json::Value;
use uuid::Uuid;

use super::arguments::{json_result, parse, schema_of};
use super::{ToolCall, ToolDefinition};
use crate::export_call::ExportCall;
use crate::scope::Scope;
use crate::server::LifePixelMcp;

/// The key of a schema's properties.
const PROPERTIES: &str = "properties";
/// The key of a schema's required properties.
const REQUIRED: &str = "required";

/// `export`: `export`, then the transport's delivery.
pub(super) const EXPORT: ToolDefinition = ToolDefinition {
    name: "export",
    scope: Scope::Export,
    description: "Exports the animation as files: `wasm`, the self-contained WebAssembly bundle \
                  with its loader `life-pixel.js`, which `get_embed_snippet` wires into a \
                  project; `gif`; `apng`; `sprite_sheet`, a PNG and its JSON; `png_frames`, a \
                  zip of one PNG per frame. `tag` limits the export to a tag the animation has; \
                  `scale` repeats each pixel of the image formats. Where the files go depends on \
                  the server, and the result says where.",
    schema: export_schema,
    run: |call| Box::pin(export(call)),
};

/// The arguments of `export` every transport shares; the others are the transport's own.
#[derive(Deserialize, JsonSchema)]
struct ExportArguments {
    /// The animation's id.
    id: Uuid,
    /// The format of the files.
    format: ExportFormatArgument,
    /// A tag of the animation whose frames to export; the whole animation when omitted.
    #[serde(default)]
    #[schemars(length(max = TAG_NAME_MAX_CHARS))]
    tag: Option<String>,
    /// How many times each pixel is repeated across and down, for the image formats.
    #[serde(default)]
    #[schemars(range(min = EXPORT_MIN_SCALE, max = EXPORT_MAX_SCALE))]
    scale: Option<u8>,
}

/// A format of `export`.
#[derive(Clone, Copy, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
enum ExportFormatArgument {
    /// The WebAssembly bundle and its loader.
    Wasm,
    /// An animated GIF.
    Gif,
    /// An animated PNG.
    Apng,
    /// Every frame in one PNG, with a JSON of where each one lies.
    SpriteSheet,
    /// A zip of one PNG per frame.
    PngFrames,
}

/// The shared arguments' schema, joined by the properties of the transport's own.
fn export_schema(server: &LifePixelMcp) -> JsonObject {
    let mut schema = schema_of::<ExportArguments>();
    let own = server.delivery().export_schema();
    let Some(own) = own.as_object() else {
        return schema;
    };
    for key in [PROPERTIES, REQUIRED] {
        join(&mut schema, (key, own.get(key)));
    }
    schema
}

/// Adds the object or array `addition` to the entry `key` of `schema`.
fn join(schema: &mut JsonObject, (key, addition): (&str, Option<&Value>)) {
    let entry = schema.entry(key);
    match (entry.or_insert_with(|| empty_like(addition)), addition) {
        (Value::Object(target), Some(Value::Object(added))) => target.extend(added.clone()),
        (Value::Array(target), Some(Value::Array(added))) => target.extend(added.clone()),
        _ => {}
    }
}

/// An empty object or array, as `value` is.
fn empty_like(value: Option<&Value>) -> Value {
    match value {
        Some(Value::Array(_)) => Value::Array(Vec::new()),
        _ => Value::Object(JsonObject::new()),
    }
}

async fn export(call: ToolCall<'_>) -> Result<CallToolResult, CodedError> {
    let (shared, options) = split(call.arguments);
    let arguments: ExportArguments = parse(shared)?;
    let (server, owner) = (call.server, &call.owner);
    let id = AnimationId::from_uuid(arguments.id);
    let version = server.library().get_animation(owner, id).await?.version;
    let request = ExportRequest {
        id,
        format: arguments.format.into(),
        tag: arguments.tag,
        scale: arguments.scale,
    };
    let files = server.editing().export(owner, request.clone()).await?;
    let export = ExportCall {
        id,
        version,
        format: request.format,
        tag: request.tag,
        scale: request.scale,
        options,
    };
    json_result(&server.delivery().deliver(owner, export, files).await?)
}

/// The shared arguments, and the transport's own: every other key.
fn split(mut arguments: JsonObject) -> (JsonObject, JsonObject) {
    let schema = schema_of::<ExportArguments>();
    let shared_keys = schema.get(PROPERTIES).and_then(Value::as_object);
    let shared = shared_keys
        .into_iter()
        .flat_map(|properties| properties.keys())
        .filter_map(|key| arguments.remove_entry(key))
        .collect();
    (shared, arguments)
}

impl From<ExportFormatArgument> for ExportFormat {
    fn from(format: ExportFormatArgument) -> Self {
        match format {
            ExportFormatArgument::Wasm => Self::Wasm,
            ExportFormatArgument::Gif => Self::Gif,
            ExportFormatArgument::Apng => Self::Apng,
            ExportFormatArgument::SpriteSheet => Self::SpriteSheet,
            ExportFormatArgument::PngFrames => Self::PngFrames,
        }
    }
}
