//! `snippet`: the integration code of `compiler::render_snippet`, and where each file goes.

use life_pixel_compiler::{Framework, SnippetInput, file_stem, render_snippet};
use life_pixel_core::Animation;
use serde::Serialize;

use super::{LOADER_FILE_NAME, checked_tag, placement};
use crate::animation::{AnimationEditing, EditingError};
use crate::ids::AnimationId;
use crate::owner::Owner;

/// Where a site serves its assets by default, as the export dialog proposes it.
const ASSETS: &str = "/assets";

/// What `snippet` fills the template with; what is omitted takes the export dialog's default.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SnippetRequest {
    /// The animation.
    pub id: AnimationId,
    /// The framework the snippet targets.
    pub framework: Framework,
    /// The URL of the `.wasm` export; `/assets/<stem>.wasm` when `None`.
    pub src: Option<String>,
    /// The URL of the loader; `/assets/life-pixel.js` when `None`.
    pub loader: Option<String>,
    /// The tag to play, which the animation must have; its default range when `None`.
    pub tag: Option<String>,
    /// The accessible name; the animation's title when `None`, decorative when empty.
    pub alt: Option<String>,
}

/// The integration code, and where each file goes.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct Snippet {
    /// The code to paste.
    pub code: String,
    /// The files to add to the project.
    pub files: Vec<SnippetFile>,
}

/// A file of the integration, and where it goes, in English: it addresses the agent.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct SnippetFile {
    /// The file's name.
    pub name: String,
    /// Where it goes, and what it needs.
    pub placement: String,
}

impl AnimationEditing {
    /// The snippet that plays the animation's WebAssembly export in `framework`, and where each
    /// of its files goes.
    ///
    /// # Errors
    ///
    /// `export.tag_not_found` for a tag the animation lacks; the library's codes.
    pub async fn snippet(
        &self,
        owner: &Owner,
        request: SnippetRequest,
    ) -> Result<Snippet, EditingError> {
        let (_, animation) = self.read(owner, request.id).await?;
        let framework = request.framework;
        let input = snippet_input(&animation, request)?;
        Ok(Snippet {
            code: render_snippet(framework, &input),
            files: placement::files(framework, &input),
        })
    }
}

/// The request's values, or their defaults, once its tag is found.
fn snippet_input(
    animation: &Animation,
    request: SnippetRequest,
) -> Result<SnippetInput, EditingError> {
    let title = animation.title().as_str();
    let stem = file_stem(title);
    Ok(SnippetInput {
        src: request
            .src
            .unwrap_or_else(|| format!("{ASSETS}/{stem}.wasm")),
        loader: request
            .loader
            .unwrap_or_else(|| format!("{ASSETS}/{LOADER_FILE_NAME}")),
        tag: checked_tag(animation, request.tag)?,
        alt: request.alt.unwrap_or_else(|| title.to_owned()),
        file_stem: stem,
    })
}
