//! `get_embed_snippet`: the code that plays the WebAssembly export in a project.

use life_pixel_compiler::Framework;
use life_pixel_core::limits::TAG_NAME_MAX_CHARS;
use life_pixel_service::animation::SnippetRequest;
use life_pixel_service::{AnimationId, CodedError};
use rmcp::model::CallToolResult;
use schemars::JsonSchema;
use serde::Deserialize;
use uuid::Uuid;

use super::arguments::{json_result, parse, schema_for};
use super::{ToolCall, ToolDefinition};
use crate::scope::Scope;

/// `get_embed_snippet`: `snippet`.
pub(super) const GET_EMBED_SNIPPET: ToolDefinition = ToolDefinition {
    name: "get_embed_snippet",
    scope: Scope::Read,
    description: "Returns the code that plays the animation's WebAssembly export — see `export` \
                  with the format `wasm` — in a plain HTML page, Angular, React or Vue, as \
                  `code`, and in `files` each file the project needs with where it goes. `src` \
                  and `loader` are the URLs the site serves the `.wasm` and `life-pixel.js` at; \
                  `tag` a tag to play; `alt` the accessible name, the title when omitted, \
                  decorative when empty.",
    schema: schema_for::<GetEmbedSnippetArguments>,
    run: |call| Box::pin(get_embed_snippet(call)),
};

/// The arguments of `get_embed_snippet`.
#[derive(Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
struct GetEmbedSnippetArguments {
    /// The animation's id.
    id: Uuid,
    /// The framework the code is for.
    framework: FrameworkArgument,
    /// The URL of the `.wasm` export; `/assets/<file name>.wasm` when omitted.
    #[serde(default)]
    src: Option<String>,
    /// The URL of the loader; `/assets/life-pixel.js` when omitted.
    #[serde(default)]
    loader: Option<String>,
    /// A tag of the animation to play; its whole animation when omitted.
    #[serde(default)]
    #[schemars(length(max = TAG_NAME_MAX_CHARS))]
    tag: Option<String>,
    /// The accessible name; the title when omitted, decorative when empty.
    #[serde(default)]
    alt: Option<String>,
}

/// A framework a snippet targets.
#[derive(Clone, Copy, Deserialize, JsonSchema)]
#[serde(rename_all = "lowercase")]
enum FrameworkArgument {
    /// A plain HTML page.
    Html,
    /// Angular.
    Angular,
    /// React.
    React,
    /// Vue.
    Vue,
}

async fn get_embed_snippet(call: ToolCall<'_>) -> Result<CallToolResult, CodedError> {
    let arguments: GetEmbedSnippetArguments = parse(call.arguments)?;
    let request = SnippetRequest {
        id: AnimationId::from_uuid(arguments.id),
        framework: arguments.framework.into(),
        src: arguments.src,
        loader: arguments.loader,
        tag: arguments.tag,
        alt: arguments.alt,
    };
    json_result(&call.server.editing().snippet(&call.owner, request).await?)
}

impl From<FrameworkArgument> for Framework {
    fn from(framework: FrameworkArgument) -> Self {
        match framework {
            FrameworkArgument::Html => Self::Html,
            FrameworkArgument::Angular => Self::Angular,
            FrameworkArgument::React => Self::React,
            FrameworkArgument::Vue => Self::Vue,
        }
    }
}
