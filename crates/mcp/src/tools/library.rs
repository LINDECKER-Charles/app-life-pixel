//! `list_animations` and `create_animation`: the library's use cases.

use life_pixel_core::limits::{
    CANVAS_MAX_SIDE, CANVAS_MIN_SIDE, MAX_FRAME_DURATION_MS, MAX_PALETTE_ENTRIES,
    MCP_PAGE_SIZE_DEFAULT, MCP_PAGE_SIZE_MAX, MIN_FRAME_DURATION_MS, NAME_MAX_CHARS,
};
use life_pixel_core::{Name, NewAnimation, Palette};
use life_pixel_service::animation::DescribeRequest;
use life_pixel_service::library::Library;
use life_pixel_service::ports::library_store::{AnimationFilter, AnimationRecord};
use life_pixel_service::{CodedError, Cursor, Owner, PageRequest, ProjectId};
use rmcp::model::CallToolResult;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use time::OffsetDateTime;
use uuid::Uuid;

use super::arguments::{COLOR_PATTERN, colors, json_result, parse, schema_for};
use super::text::user_content;
use super::{ToolCall, ToolDefinition};
use crate::errors::malformed;
use crate::scope::Scope;

/// The smallest page of a listing.
const MCP_PAGE_SIZE_MIN: u16 = 1;
/// The name of a new animation's layer when the call names none.
const DEFAULT_LAYER_NAME: &str = "Layer 1";

/// `list_animations`: a page of the caller's animations.
pub(super) const LIST_ANIMATIONS: ToolDefinition = ToolDefinition {
    name: "list_animations",
    scope: Scope::Read,
    description: concat!(
        "Lists your animations, the most recently updated first, a page at a time: filter by \
         `query`, a case-insensitive part of the title, or by `project_id`, and pass \
         `next_cursor` as `cursor` for the next page; `next_cursor` is null on the last page. ",
        user_content!()
    ),
    schema: schema_for::<ListAnimationsArguments>,
    run: |call| Box::pin(list_animations(call)),
};

/// `create_animation`: a blank animation, in a project found or created by name.
pub(super) const CREATE_ANIMATION: ToolDefinition = ToolDefinition {
    name: "create_animation",
    scope: Scope::Write,
    description: "Creates a blank animation — one layer, one transparent frame, no tag — and \
                  returns its view. Give `project_id`, or `project_name` to use your project of \
                  that name, created when you have none. The palette is a list of `#rrggbbaa` \
                  colours whose entry 0 is fully transparent; a default palette when omitted.",
    schema: schema_for::<CreateAnimationArguments>,
    run: |call| Box::pin(create_animation(call)),
};

/// The arguments of `list_animations`.
#[derive(Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
struct ListAnimationsArguments {
    /// Only the animations whose title contains this text, case-insensitively.
    #[serde(default)]
    #[schemars(length(max = NAME_MAX_CHARS))]
    query: Option<String>,
    /// Only the animations of this project.
    #[serde(default)]
    project_id: Option<Uuid>,
    /// The `next_cursor` of the previous page; the first page when omitted.
    #[serde(default)]
    cursor: Option<String>,
    /// The most animations in the page.
    #[serde(default = "default_limit")]
    #[schemars(range(min = MCP_PAGE_SIZE_MIN, max = MCP_PAGE_SIZE_MAX))]
    limit: u16,
}

/// The arguments of `create_animation`.
#[derive(Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
struct CreateAnimationArguments {
    /// The title.
    #[schemars(length(max = NAME_MAX_CHARS))]
    title: String,
    /// The canvas width, in pixels.
    #[schemars(range(min = CANVAS_MIN_SIDE, max = CANVAS_MAX_SIDE))]
    width: u16,
    /// The canvas height, in pixels.
    #[schemars(range(min = CANVAS_MIN_SIDE, max = CANVAS_MAX_SIDE))]
    height: u16,
    /// The project to create it in; give this or `project_name`.
    #[serde(default)]
    project_id: Option<Uuid>,
    /// The name of your project to create it in, created when missing; or `project_id`.
    #[serde(default)]
    #[schemars(length(max = NAME_MAX_CHARS))]
    project_name: Option<String>,
    /// The palette as `#rrggbbaa` colours, entry 0 first and fully transparent.
    #[serde(default)]
    #[schemars(length(max = MAX_PALETTE_ENTRIES), inner(pattern(COLOR_PATTERN)))]
    palette: Option<Vec<String>>,
    /// How long the first frame shows, in milliseconds.
    #[serde(default)]
    #[schemars(range(min = MIN_FRAME_DURATION_MS, max = MAX_FRAME_DURATION_MS))]
    frame_duration_ms: Option<u16>,
    /// The name of its layer.
    #[serde(default)]
    #[schemars(length(max = NAME_MAX_CHARS))]
    layer_name: Option<String>,
}

/// A page of animations.
#[derive(Serialize)]
struct AnimationList {
    animations: Vec<AnimationSummary>,
    next_cursor: Option<String>,
}

/// An animation in a list: its record, without its document.
#[derive(Serialize)]
struct AnimationSummary {
    id: Uuid,
    title: String,
    project_id: Uuid,
    width: u16,
    height: u16,
    frame_count: u16,
    #[serde(serialize_with = "time::serde::rfc3339::serialize")]
    updated_at: OffsetDateTime,
}

fn default_limit() -> u16 {
    u16::try_from(MCP_PAGE_SIZE_DEFAULT).unwrap_or(MCP_PAGE_SIZE_MIN)
}

async fn list_animations(call: ToolCall<'_>) -> Result<CallToolResult, CodedError> {
    let arguments: ListAnimationsArguments = parse(call.arguments)?;
    let cursor = arguments.cursor.as_deref().map(str::parse::<Cursor>);
    let cursor = cursor.transpose().map_err(|error| CodedError::of(&error))?;
    let max = u16::try_from(MCP_PAGE_SIZE_MAX).unwrap_or(MCP_PAGE_SIZE_MIN);
    let page = PageRequest::new(cursor, Some(arguments.limit.clamp(MCP_PAGE_SIZE_MIN, max)));
    let filter = AnimationFilter {
        project: arguments.project_id.map(ProjectId::from_uuid),
        query: arguments.query,
    };
    let listed = call
        .server
        .library()
        .list_animations(&call.owner, filter, page);
    let page = listed.await?;
    json_result(&AnimationList {
        animations: page.items.iter().map(AnimationSummary::of).collect(),
        next_cursor: page.next_cursor.map(|cursor| cursor.to_string()),
    })
}

async fn create_animation(call: ToolCall<'_>) -> Result<CallToolResult, CodedError> {
    let arguments: CreateAnimationArguments = parse(call.arguments)?;
    let spec = arguments.spec()?;
    let (library, owner) = (call.server.library(), &call.owner);
    let project = match (arguments.project_id, arguments.project_name) {
        (Some(id), None) => ProjectId::from_uuid(id),
        (None, Some(name)) => project_named(library, owner, &name).await?,
        _ => return Err(malformed()),
    };
    let record = library.create_animation(owner, project, spec).await?;
    let request = DescribeRequest {
        id: record.id,
        pixels: None,
    };
    json_result(&call.server.editing().describe(owner, request).await?)
}

impl CreateAnimationArguments {
    /// The new animation, its names and palette checked by `core`.
    fn spec(&self) -> Result<NewAnimation, CodedError> {
        let palette = self.palette.as_deref().map(colors).transpose()?;
        let layer_name = self.layer_name.as_deref().unwrap_or(DEFAULT_LAYER_NAME);
        Ok(NewAnimation {
            title: Name::new(&self.title)?,
            width: self.width,
            height: self.height,
            layer_name: Name::new(layer_name)?,
            frame_duration_ms: self.frame_duration_ms,
            palette: palette.map(Palette::new).transpose()?,
        })
    }
}

/// The owner's project named `name`, created when there is none.
async fn project_named(
    library: &Library,
    owner: &Owner,
    name: &str,
) -> Result<ProjectId, CodedError> {
    let wanted = Name::new(name)?;
    let mut page = PageRequest::new(None, None);
    loop {
        let projects = library.list_projects(owner, page.clone()).await?;
        let found = projects.items.iter().find(|project| project.name == wanted);
        if let Some(project) = found {
            return Ok(project.id);
        }
        let Some(next) = projects.next_cursor else {
            break;
        };
        page.cursor = Some(next);
    }
    Ok(library.create_project(owner, wanted.as_str()).await?.id)
}

impl AnimationSummary {
    fn of(record: &AnimationRecord) -> Self {
        Self {
            id: record.id.uuid(),
            title: record.meta.title.as_str().to_owned(),
            project_id: record.project.uuid(),
            width: record.meta.width,
            height: record.meta.height,
            frame_count: record.meta.frame_count,
            updated_at: record.updated_at,
        }
    }
}
