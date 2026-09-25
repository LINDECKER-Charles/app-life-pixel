//! What an animation gives an agent's project: exported files, and the snippet that plays them.

mod export;
mod placement;
mod snippet;

pub use export::ExportRequest;
use life_pixel_compiler::ExportError;
use life_pixel_core::Animation;
pub use snippet::{Snippet, SnippetFile, SnippetRequest};

/// The loader's file name, beside every WebAssembly export.
const LOADER_FILE_NAME: &str = "life-pixel.js";

/// `tag` itself, when the animation has a tag of that name; no tag, or an empty name, is the
/// whole animation.
fn checked_tag(animation: &Animation, tag: Option<String>) -> Result<Option<String>, ExportError> {
    let Some(name) = tag.filter(|name| !name.is_empty()) else {
        return Ok(None);
    };
    let exists = animation
        .tags()
        .iter()
        .any(|tag| tag.name().as_str() == name);
    if exists {
        Ok(Some(name))
    } else {
        Err(ExportError::TagNotFound { name })
    }
}
