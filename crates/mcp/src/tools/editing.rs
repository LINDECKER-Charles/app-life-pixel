//! The changes: `set_palette`, `write_frame`, `draw`, `edit_frames` and `set_tags`, each saved by
//! its use case with the version it read.

use life_pixel_core::edit::TagSpec;
use life_pixel_core::limits::{
    CANVAS_MAX_SIDE, DRAW_MAX_OPERATIONS, MAX_FRAMES, MAX_PALETTE_ENTRIES, MAX_TAGS,
};
use life_pixel_service::animation::{
    DrawRequest, EditFramesRequest, FrameView, SetPaletteRequest, SetTagsRequest, WriteFrameRequest,
};
use life_pixel_service::{AnimationId, CodedError};
use rmcp::model::CallToolResult;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::arguments::{COLOR_PATTERN, colors, json_result, parse, schema_for};
use super::operations::{DrawOperationArgument, FrameEditArgument, TagArgument};
use super::text::grid_alphabet;
use super::{ToolCall, ToolDefinition};
use crate::scope::Scope;

/// `set_palette`: the palette, replaced as a whole.
pub(super) const SET_PALETTE: ToolDefinition = ToolDefinition {
    name: "set_palette",
    scope: Scope::Write,
    description: "Replaces the palette with `palette`, `#rrggbbaa` colours in index order; entry \
                  0 stays transparent. Pixels keep their indices, so changing a colour repaints \
                  its pixels; dropping an entry still painted fails with `edit.palette_in_use`. \
                  Returns the view.",
    schema: schema_for::<SetPaletteArguments>,
    run: |call| Box::pin(set_palette(call)),
};

/// `write_frame`: a text grid as a layer's pixels on a frame.
pub(super) const WRITE_FRAME: ToolDefinition = ToolDefinition {
    name: "write_frame",
    scope: Scope::Write,
    description: concat!(
        "Replaces the pixels of one layer on one frame with a text grid of the canvas's size — \
         the top layer when `layer_id` is omitted — and returns the view. ",
        grid_alphabet!()
    ),
    schema: schema_for::<WriteFrameArguments>,
    run: |call| Box::pin(write_frame(call)),
};

/// `draw`: drawing operations on a layer of a frame.
pub(super) const DRAW: ToolDefinition = ToolDefinition {
    name: "draw",
    scope: Scope::Write,
    description: "Applies drawing operations in order to one layer of one frame — the top layer \
                  when `layer_id` is omitted: `pixel`, `line`, `rectangle` and `fill`, each with \
                  a palette index, 0 being transparent. If one fails, none is kept. Returns the \
                  view.",
    schema: schema_for::<DrawArguments>,
    run: |call| Box::pin(draw(call)),
};

/// `edit_frames`: frame edits, in order.
pub(super) const EDIT_FRAMES: ToolDefinition = ToolDefinition {
    name: "edit_frames",
    scope: Scope::Write,
    description: "Adds, duplicates, deletes and moves frames and sets their durations, in order, \
                  each edit reading positions after the edits before it; tags follow the frames \
                  they cover. If one fails, none is kept. Returns the frames and the tags.",
    schema: schema_for::<EditFramesArguments>,
    run: |call| Box::pin(edit_frames(call)),
};

/// `set_tags`: every tag, replaced at once.
pub(super) const SET_TAGS: ToolDefinition = ToolDefinition {
    name: "set_tags",
    scope: Scope::Write,
    description: "Replaces every tag with `tags`: named frame ranges an app can play on their \
                  own, each looping or played once. An empty list removes every tag. Returns the \
                  tags.",
    schema: schema_for::<SetTagsArguments>,
    run: |call| Box::pin(set_tags(call)),
};

/// The arguments of `set_palette`.
#[derive(Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
struct SetPaletteArguments {
    /// The animation's id.
    id: Uuid,
    /// Every colour as `#rrggbbaa`, entry 0 first and fully transparent.
    #[schemars(length(max = MAX_PALETTE_ENTRIES), inner(pattern(COLOR_PATTERN)))]
    palette: Vec<String>,
}

/// The arguments of `write_frame`.
#[derive(Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
struct WriteFrameArguments {
    /// The animation's id.
    id: Uuid,
    /// The frame's position, from 0.
    #[schemars(range(max = MAX_FRAMES - 1))]
    frame: u16,
    /// The layer's id, from `get_animation`; the top layer when omitted.
    #[serde(default)]
    layer_id: Option<u32>,
    /// The rows, top to bottom: as many as the canvas is high, each as wide as it.
    #[schemars(length(max = CANVAS_MAX_SIDE))]
    grid: Vec<String>,
}

/// The arguments of `draw`.
#[derive(Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
struct DrawArguments {
    /// The animation's id.
    id: Uuid,
    /// The frame's position, from 0.
    #[schemars(range(max = MAX_FRAMES - 1))]
    frame: u16,
    /// The layer's id, from `get_animation`; the top layer when omitted.
    #[serde(default)]
    layer_id: Option<u32>,
    /// The operations, in drawing order.
    #[schemars(length(max = DRAW_MAX_OPERATIONS))]
    operations: Vec<DrawOperationArgument>,
}

/// The arguments of `edit_frames`.
#[derive(Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
struct EditFramesArguments {
    /// The animation's id.
    id: Uuid,
    /// The edits, applied in order.
    edits: Vec<FrameEditArgument>,
}

/// The arguments of `set_tags`.
#[derive(Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
struct SetTagsArguments {
    /// The animation's id.
    id: Uuid,
    /// The new tags.
    #[schemars(length(max = MAX_TAGS))]
    tags: Vec<TagArgument>,
}

/// The result of `edit_frames`.
#[derive(Serialize)]
struct FramesAndTags<'a> {
    frames: &'a [FrameView],
    tags: &'a [TagSpec],
}

/// The result of `set_tags`.
#[derive(Serialize)]
struct Tags<'a> {
    tags: &'a [TagSpec],
}

async fn set_palette(call: ToolCall<'_>) -> Result<CallToolResult, CodedError> {
    let arguments: SetPaletteArguments = parse(call.arguments)?;
    let request = SetPaletteRequest {
        id: AnimationId::from_uuid(arguments.id),
        colors: colors(&arguments.palette)?,
    };
    json_result(
        &call
            .server
            .editing()
            .set_palette(&call.owner, request)
            .await?,
    )
}

async fn write_frame(call: ToolCall<'_>) -> Result<CallToolResult, CodedError> {
    let arguments: WriteFrameArguments = parse(call.arguments)?;
    let request = WriteFrameRequest {
        id: AnimationId::from_uuid(arguments.id),
        frame: arguments.frame,
        layer: arguments.layer_id,
        grid: arguments.grid,
    };
    json_result(
        &call
            .server
            .editing()
            .write_frame(&call.owner, request)
            .await?,
    )
}

async fn draw(call: ToolCall<'_>) -> Result<CallToolResult, CodedError> {
    let arguments: DrawArguments = parse(call.arguments)?;
    let request = DrawRequest {
        id: AnimationId::from_uuid(arguments.id),
        frame: arguments.frame,
        layer: arguments.layer_id,
        operations: arguments.operations.into_iter().map(Into::into).collect(),
    };
    json_result(&call.server.editing().draw(&call.owner, request).await?)
}

async fn edit_frames(call: ToolCall<'_>) -> Result<CallToolResult, CodedError> {
    let arguments: EditFramesArguments = parse(call.arguments)?;
    let request = EditFramesRequest {
        id: AnimationId::from_uuid(arguments.id),
        edits: arguments.edits.into_iter().map(Into::into).collect(),
    };
    let view = call
        .server
        .editing()
        .edit_frames(&call.owner, request)
        .await?;
    json_result(&FramesAndTags {
        frames: &view.frames,
        tags: &view.tags,
    })
}

async fn set_tags(call: ToolCall<'_>) -> Result<CallToolResult, CodedError> {
    let arguments: SetTagsArguments = parse(call.arguments)?;
    let request = SetTagsRequest {
        id: AnimationId::from_uuid(arguments.id),
        tags: arguments.tags.into_iter().map(Into::into).collect(),
    };
    let view = call.server.editing().set_tags(&call.owner, request).await?;
    json_result(&Tags { tags: &view.tags })
}
