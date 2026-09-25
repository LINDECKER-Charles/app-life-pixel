//! The tools of mcp.md, grouped by what they touch. Each is a [`ToolDefinition`]: its name, scope,
//! description and schema in English — they address the model —, and the use case it runs.

mod arguments;
mod editing;
mod export;
mod library;
mod operations;
mod preview;
mod read;
mod snippet;
mod text;

use std::future::Future;
use std::pin::Pin;

use life_pixel_service::{CodedError, Owner};
use rmcp::model::{CallToolResult, JsonObject, Tool, ToolAnnotations};

use crate::scope::Scope;
use crate::server::LifePixelMcp;

/// What a tool's run gives back: its result, or the coded failure the server turns into one.
pub(crate) type ToolFuture<'a> =
    Pin<Box<dyn Future<Output = Result<CallToolResult, CodedError>> + Send + 'a>>;

/// A call a tool runs: the server's use cases, the caller's owner, and the arguments as sent.
pub(crate) struct ToolCall<'a> {
    /// The server, holding the use cases.
    pub(crate) server: &'a LifePixelMcp,
    /// Whose library the call works on.
    pub(crate) owner: Owner,
    /// The arguments, not yet read.
    pub(crate) arguments: JsonObject,
}

/// A tool: what the client lists, what the policy checks, and what runs.
pub(crate) struct ToolDefinition {
    /// The tool's name, in `snake_case`.
    pub(crate) name: &'static str,
    /// The scope a call needs.
    pub(crate) scope: Scope,
    /// What it does, for the model.
    pub(crate) description: &'static str,
    /// The JSON Schema of its arguments; `export`'s depends on the transport.
    pub(crate) schema: fn(&LifePixelMcp) -> JsonObject,
    /// Runs a call.
    pub(crate) run: for<'a> fn(ToolCall<'a>) -> ToolFuture<'a>,
}

/// Every tool, in the order of mcp.md.
const TOOLS: [&ToolDefinition; 11] = [
    &library::LIST_ANIMATIONS,
    &library::CREATE_ANIMATION,
    &read::GET_ANIMATION,
    &editing::SET_PALETTE,
    &editing::WRITE_FRAME,
    &editing::DRAW,
    &editing::EDIT_FRAMES,
    &editing::SET_TAGS,
    &preview::RENDER_PREVIEW,
    &export::EXPORT,
    &snippet::GET_EMBED_SNIPPET,
];

/// Every tool.
pub(crate) fn all() -> impl Iterator<Item = &'static ToolDefinition> {
    TOOLS.into_iter()
}

/// The tool named `name`, if there is one.
pub(crate) fn find(name: &str) -> Option<&'static ToolDefinition> {
    all().find(|tool| tool.name == name)
}

impl ToolDefinition {
    /// The tool as `tools/list` shows it on `server`.
    pub(crate) fn tool(&self, server: &LifePixelMcp) -> Tool {
        let annotations = ToolAnnotations::new().read_only(self.scope == Scope::Read);
        Tool::new(self.name, self.description, (self.schema)(server)).with_annotations(annotations)
    }
}
