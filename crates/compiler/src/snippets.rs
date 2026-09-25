//! The integration snippets of the export dialog and of MCP: the templates of
//! `player-js/snippets/`, shipped with the loader under MIT, filled in with an export's values.

mod escape;
mod framework;
mod placeholders;
mod snippet_input;
mod values;

pub use framework::Framework;
pub use snippet_input::SnippetInput;

use values::Values;

/// The snippet that plays an export in `framework`: its template with every placeholder replaced
/// by `input`'s value, escaped for the snippet's language — HTML attribute escaping, and
/// JavaScript string escaping for React's `alt` expression.
#[must_use]
pub fn render_snippet(framework: Framework, input: &SnippetInput) -> String {
    let values = Values::new(framework, input);
    placeholders::fill(framework.template(), |name| values.get(name))
}
