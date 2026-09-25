//! The integration snippet of the export dialog, through `compiler::render_snippet`.

use life_pixel_compiler::{Framework, SnippetInput, file_stem, render_snippet};
use life_pixel_core::Animation;
use serde::Deserialize;

/// The interface's `SnippetRequest`.
#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct SnippetRequest {
    /// Where the snippet goes.
    pub framework: Framework,
    /// The URL the page loads the export from.
    pub src: String,
    /// The URL of the loader.
    pub loader: String,
    /// The tag to play; the default range when absent or empty.
    pub tag: Option<String>,
    /// The accessible name; empty marks the animation decorative.
    pub alt: String,
}

/// The snippet that plays the export of `animation`: its component is named after the file stem
/// of the animation's title.
#[must_use]
pub fn render_request(animation: &Animation, request: &SnippetRequest) -> String {
    let input = SnippetInput {
        src: request.src.clone(),
        loader: request.loader.clone(),
        tag: request.tag.clone(),
        alt: request.alt.clone(),
        file_stem: file_stem(animation.title().as_str()),
    };
    render_snippet(request.framework, &input)
}
