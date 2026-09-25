//! `get_animation`: an animation's view, with pixels only when asked.

use life_pixel_core::limits::MAX_FRAMES;
use life_pixel_service::animation::DescribeRequest;
use life_pixel_service::{AnimationId, CodedError, Owner};
use rmcp::model::CallToolResult;
use schemars::JsonSchema;
use serde::Deserialize;
use uuid::Uuid;

use super::arguments::{json_result, parse, schema_for};
use super::text::{grid_alphabet, user_content};
use super::{ToolCall, ToolDefinition};
use crate::scope::Scope;
use crate::server::LifePixelMcp;

/// `get_animation`: `describe`.
pub(super) const GET_ANIMATION: ToolDefinition = ToolDefinition {
    name: "get_animation",
    scope: Scope::Read,
    description: concat!(
        "Returns an animation's view: title, size, `palette` as `#rrggbbaa` colours — the \
         position of a colour is its index, 0 being transparent —, `layers` from the bottom to \
         the top with the `id` other tools take as `layer_id`, `frames` in play order with their \
         `duration_ms`, and `tags`. No pixel unless `include_pixels` is true: `grids` then holds \
         the composited frames at the positions in `frames`, every frame when omitted. ",
        grid_alphabet!(),
        " ",
        user_content!()
    ),
    schema: schema_for::<GetAnimationArguments>,
    run: |call| Box::pin(get_animation(call)),
};

/// The arguments of `get_animation`.
#[derive(Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
struct GetAnimationArguments {
    /// The animation's id.
    id: Uuid,
    /// Whether to return composited frames as text grids.
    #[serde(default)]
    include_pixels: bool,
    /// The positions of the frames returned as grids, from 0; every frame when omitted.
    #[serde(default)]
    #[schemars(length(max = MAX_FRAMES), inner(range(max = MAX_FRAMES - 1)))]
    frames: Option<Vec<u16>>,
}

async fn get_animation(call: ToolCall<'_>) -> Result<CallToolResult, CodedError> {
    let arguments: GetAnimationArguments = parse(call.arguments)?;
    let id = AnimationId::from_uuid(arguments.id);
    let pixels = match (arguments.include_pixels, arguments.frames) {
        (false, _) => None,
        (true, Some(frames)) => Some(frames),
        (true, None) => Some(every_frame((call.server, &call.owner), id).await?),
    };
    let view = call
        .server
        .editing()
        .describe(&call.owner, DescribeRequest { id, pixels });
    json_result(&view.await?)
}

/// The position of every frame of the animation `id`.
async fn every_frame(
    (server, owner): (&LifePixelMcp, &Owner),
    id: AnimationId,
) -> Result<Vec<u16>, CodedError> {
    let record = server.library().get_animation(owner, id).await?;
    Ok((0..record.meta.frame_count).collect())
}
