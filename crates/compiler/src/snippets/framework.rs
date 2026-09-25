//! The frameworks a snippet targets, each with its template.

use serde::{Deserialize, Serialize};

/// `player-js/snippets/html.html`: the loader's script and the element.
const HTML_TEMPLATE: &str = include_str!("../../../../player-js/snippets/html.html");
/// `player-js/snippets/angular.ts`: the import for `main.ts` and a standalone component.
const ANGULAR_TEMPLATE: &str = include_str!("../../../../player-js/snippets/angular.ts");
/// `player-js/snippets/react.tsx`: a component and the JSX declaration TypeScript needs.
const REACT_TEMPLATE: &str = include_str!("../../../../player-js/snippets/react.tsx");
/// `player-js/snippets/vue.vue`: a single-file component.
const VUE_TEMPLATE: &str = include_str!("../../../../player-js/snippets/vue.vue");

/// Where a snippet goes. Serialized `"html"`, `"angular"`, `"react"`, `"vue"`: the names the
/// export dialog and the MCP tool `get_embed_snippet` use.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Framework {
    /// A plain HTML page.
    Html,
    /// Angular, through `CUSTOM_ELEMENTS_SCHEMA`.
    Angular,
    /// React 19.
    React,
    /// Vue, through `isCustomElement`.
    Vue,
}

impl Framework {
    /// Every framework, in the order the export dialog lists them.
    pub const ALL: [Self; 4] = [Self::Html, Self::Angular, Self::React, Self::Vue];

    /// The framework's template, with its `{{placeholders}}`.
    pub(super) fn template(self) -> &'static str {
        match self {
            Self::Html => HTML_TEMPLATE,
            Self::Angular => ANGULAR_TEMPLATE,
            Self::React => REACT_TEMPLATE,
            Self::Vue => VUE_TEMPLATE,
        }
    }
}
