//! The integration snippets, rendered from the templates of `player-js/snippets/`.

#![allow(
    clippy::unwrap_used,
    reason = "helpers of tests may unwrap too: a panic is a failed test"
)]

use life_pixel_compiler::{Framework, SnippetInput, render_snippet};

const ELEMENT: &str =
    r#"<life-pixel src="/assets/mascot.wasm" tag="idle" alt="The mascot waving"></life-pixel>"#;

fn input() -> SnippetInput {
    SnippetInput {
        src: "/assets/mascot.wasm".to_owned(),
        loader: "/assets/life-pixel.js".to_owned(),
        tag: Some("idle".to_owned()),
        alt: "The mascot waving".to_owned(),
        file_stem: "mascot-idle".to_owned(),
    }
}

fn with_alt(alt: &str) -> SnippetInput {
    SnippetInput {
        alt: alt.to_owned(),
        ..input()
    }
}

#[test]
fn the_html_snippet_loads_the_loader_then_shows_the_element() {
    let snippet = render_snippet(Framework::Html, &input());

    let script = r#"<script type="module" src="/assets/life-pixel.js"></script>"#;
    assert_eq!(snippet, format!("{script}\n{ELEMENT}\n"));
}

#[test]
fn every_snippet_fills_in_every_placeholder_and_shows_the_element() {
    for framework in Framework::ALL {
        let snippet = render_snippet(framework, &input());

        assert!(!snippet.contains("{{"), "{framework:?}: {snippet}");
        let element = if framework == Framework::React {
            ELEMENT.replace(r#"alt="The mascot waving""#, r#"alt={"The mascot waving"}"#)
        } else {
            ELEMENT.to_owned()
        };
        assert!(snippet.contains(&element), "{framework:?}: {snippet}");
    }
}

#[test]
fn only_the_html_snippet_loads_the_loader_the_others_import_the_package() {
    for framework in [Framework::Angular, Framework::React, Framework::Vue] {
        let snippet = render_snippet(framework, &input());

        assert!(!snippet.contains("/assets/life-pixel.js"), "{framework:?}");
        assert!(
            snippet.contains("import '@life-pixel/player';"),
            "{framework:?}"
        );
    }
}

#[test]
fn components_are_named_after_the_file_stem() {
    let angular = render_snippet(Framework::Angular, &input());
    let react = render_snippet(Framework::React, &input());

    assert!(angular.contains("export class MascotIdleAnimation {}"));
    assert!(react.contains("export function MascotIdleAnimation() {"));
}

#[test]
fn without_a_tag_no_tag_attribute_is_written() {
    for tag in [None, Some(String::new())] {
        let input = SnippetInput { tag, ..input() };
        for framework in Framework::ALL {
            let snippet = render_snippet(framework, &input);

            assert!(!snippet.contains(" tag="), "{framework:?}");
            assert!(snippet.contains(r#"<life-pixel src="/assets/mascot.wasm" alt"#));
        }
    }
}

#[test]
fn values_are_escaped_for_html_attributes() {
    let input = SnippetInput {
        src: "/a.wasm?v=1&t=2".to_owned(),
        ..with_alt(r#""Tom" & <Jerry>"#)
    };
    for framework in [Framework::Html, Framework::Vue] {
        let snippet = render_snippet(framework, &input);

        assert!(
            snippet.contains(r#"src="/a.wasm?v=1&amp;t=2""#),
            "{framework:?}"
        );
        assert!(snippet.contains(r#"alt="&quot;Tom&quot; &amp; &lt;Jerry&gt;""#));
    }
}

#[test]
fn react_writes_the_accessible_name_as_a_javascript_string() {
    let snippet = render_snippet(Framework::React, &with_alt("\"Tom\" \\ Jerry\n"));

    assert!(
        snippet.contains(r#"alt={"\"Tom\" \\ Jerry\n"}"#),
        "{snippet}"
    );
}

#[test]
fn angular_escapes_what_its_template_literal_or_interpolation_would_read() {
    let snippet = render_snippet(Framework::Angular, &with_alt("`${x}` {{y}}"));

    let alt = r#"alt="&#96;&#36;&#123;x&#125;&#96; &#123;&#123;y&#125;&#125;""#;
    assert!(snippet.contains(alt), "{snippet}");
}

#[test]
fn frameworks_serialize_as_their_shared_names() {
    let names = serde_json::to_value(Framework::ALL).unwrap();

    assert_eq!(
        names,
        serde_json::json!(["html", "angular", "react", "vue"])
    );
    let parsed: Vec<Framework> = serde_json::from_value(names).unwrap();
    assert_eq!(parsed, Framework::ALL);
}
