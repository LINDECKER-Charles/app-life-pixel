//! The use cases of agents: what the MCP tools do to an animation, under the same validation,
//! limits and quota as the interface.
//!
//! Frames are addressed by position, from 0; layers by the ids [`AnimationEditing::describe`]
//! returns, the top layer when none is given. A change reads the document, applies its
//! operations with `core` on the blocking pool, and saves with the version it read; on a
//! conflict it reads again and retries once. Exports and snippets come from `compiler`.

mod addressing;
mod change;
mod describe;
mod edits;
mod error;
mod output;
mod preview;
mod view;

use std::sync::Arc;

pub use describe::DescribeRequest;
pub use edits::{
    DrawOperation, DrawRequest, EditFramesRequest, FrameEdit, SetPaletteRequest, SetTagsRequest,
    WriteFrameRequest,
};
pub use error::EditingError;
pub use output::{ExportRequest, Snippet, SnippetFile, SnippetRequest};
pub use preview::{Preview, PreviewCell, PreviewRequest};
pub use view::{AnimationView, FrameGrid, FrameView, LayerView};

use crate::library::Library;
use crate::ports::EventSink;

/// The animation use cases of the MCP tools, over the [`Library`] and its adapters.
#[derive(Clone)]
pub struct AnimationEditing {
    library: Library,
    events: Arc<dyn EventSink>,
}

impl AnimationEditing {
    /// The use cases over `library`; `events` receives `export_completed`, for accounts only.
    #[must_use]
    pub fn new(library: Library, events: Arc<dyn EventSink>) -> Self {
        Self { library, events }
    }
}
