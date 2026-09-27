//! Where each file of an integration goes, per framework, in English: the agent reads it.

use life_pixel_compiler::{Framework, SnippetInput};

use super::LOADER_FILE_NAME;
use super::snippet::SnippetFile;

/// Where the loader goes, after the URL it loads at.
const HTML_LOADER: &str = "One copy serves every animation of the site; paste the code into the \
                           page that shows the animation.";
/// Where Angular's component goes.
const ANGULAR_COMPONENT: &str = "The component, with the other components under src/app/. \
                                 Install @life-pixel/player with npm, and move the snippet's \
                                 first import to src/main.ts.";
/// Where React's component goes.
const REACT_COMPONENT: &str = "The component, with the other components under src/. Install \
                               @life-pixel/player with npm; the JSX declaration it holds is \
                               needed once per project.";
/// Where Vue's component goes.
const VUE_COMPONENT: &str = "The single-file component, under src/components/. Install \
                             @life-pixel/player with npm, and set isCustomElement in \
                             vite.config.ts as its comment shows.";

/// The files a project needs to play the export in `framework`: the `.wasm`, then the loader
/// for a page, or the component the snippet holds.
pub(super) fn files(framework: Framework, input: &SnippetInput) -> Vec<SnippetFile> {
    let module = file(
        format!("{}.wasm", input.file_stem),
        module_placement(framework, input),
    );
    vec![module, companion(framework, input)]
}

/// The loader of a page, or the component the snippet holds.
fn companion(framework: Framework, input: &SnippetInput) -> SnippetFile {
    let stem = &input.file_stem;
    match framework {
        Framework::Html => file(
            LOADER_FILE_NAME.to_owned(),
            format!(
                "Serve it so that it loads at {}. {HTML_LOADER}",
                input.loader
            ),
        ),
        Framework::Angular => file(
            format!("{stem}-animation.component.ts"),
            ANGULAR_COMPONENT.to_owned(),
        ),
        Framework::React => file(format!("{stem}-animation.tsx"), REACT_COMPONENT.to_owned()),
        Framework::Vue => file(format!("{stem}-animation.vue"), VUE_COMPONENT.to_owned()),
    }
}

/// Where the `.wasm` goes: wherever the site serves it at `src`.
fn module_placement(framework: Framework, input: &SnippetInput) -> String {
    let folder = match framework {
        Framework::Html => "the site's static files",
        Framework::Angular => "the project's public/ folder (src/assets/ in older projects)",
        Framework::React | Framework::Vue => "the project's public/ folder",
    };
    format!(
        "The animation, from the WebAssembly export: put it in {folder}, so that it is served \
         at {}.",
        input.src
    )
}

fn file(name: String, placement: String) -> SnippetFile {
    SnippetFile { name, placement }
}
