//! What a snippet is filled in with.

/// The values of a snippet, unescaped: [`render_snippet`](crate::render_snippet) escapes each for
/// the snippet's language.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct SnippetInput {
    /// The URL the page loads the export from: `/assets/mascot.wasm`, for example.
    pub src: String,
    /// The URL of the loader, `life-pixel.js`. Only the HTML snippet loads it; the others import
    /// the package `@life-pixel/player`.
    pub loader: String,
    /// The tag to play, by name; `None` — or an empty name — plays the default range, and
    /// writes no `tag` attribute.
    pub tag: Option<String>,
    /// The accessible name; empty marks the animation decorative.
    pub alt: String,
    /// The stem of the export's files, from [`file_stem`](crate::file_stem): the component is
    /// named after it.
    pub file_stem: String,
}
