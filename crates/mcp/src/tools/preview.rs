//! `render_preview`: a PNG the agent looks at, within the preview's caps.

use base64::Engine as _;
use base64::engine::general_purpose::STANDARD;
use life_pixel_core::limits::{EXPORT_MAX_SCALE, EXPORT_MIN_SCALE, MAX_FRAMES};
use life_pixel_service::animation::PreviewRequest;
use life_pixel_service::{AnimationId, CodedError};
use rmcp::model::{CallToolResult, ContentBlock};
use schemars::JsonSchema;
use serde::Deserialize;
use uuid::Uuid;

use super::arguments::{parse, schema_for};
use super::{ToolCall, ToolDefinition};
use crate::errors::unavailable;
use crate::scope::Scope;

/// The media type of a preview.
const PNG_MEDIA_TYPE: &str = "image/png";

/// `render_preview`: `preview`.
pub(super) const RENDER_PREVIEW: ToolDefinition = ToolDefinition {
    name: "render_preview",
    scope: Scope::Read,
    description: "Renders a PNG so that you can see what you drew: one composited frame, or a \
                  contact sheet of every frame, row by row, when `frame` is omitted. Each pixel \
                  is repeated `scale` times, chosen to fit about 512 pixels when omitted. Returns \
                  the image and a text giving its `width`, `height`, `scale` and `layout`: where \
                  each frame lies. Beyond the preview's caps it fails with `preview.too_large`.",
    schema: schema_for::<RenderPreviewArguments>,
    run: |call| Box::pin(render_preview(call)),
};

/// The arguments of `render_preview`.
#[derive(Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
struct RenderPreviewArguments {
    /// The animation's id.
    id: Uuid,
    /// The frame's position, from 0; a contact sheet of every frame when omitted.
    #[serde(default)]
    #[schemars(range(max = MAX_FRAMES - 1))]
    frame: Option<u16>,
    /// How many times each pixel is repeated across and down; automatic when omitted.
    #[serde(default)]
    #[schemars(range(min = EXPORT_MIN_SCALE, max = EXPORT_MAX_SCALE))]
    scale: Option<u8>,
}

async fn render_preview(call: ToolCall<'_>) -> Result<CallToolResult, CodedError> {
    let arguments: RenderPreviewArguments = parse(call.arguments)?;
    let request = PreviewRequest {
        id: AnimationId::from_uuid(arguments.id),
        frame: arguments.frame,
        scale: arguments.scale,
    };
    let preview = call.server.editing().preview(&call.owner, request).await?;
    let layout = serde_json::to_string(&preview).map_err(|_| unavailable())?;
    Ok(CallToolResult::success(vec![
        ContentBlock::image(STANDARD.encode(&preview.png), PNG_MEDIA_TYPE),
        ContentBlock::text(layout),
    ]))
}
