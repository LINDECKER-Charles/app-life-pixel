//! `list`: the library's animations, as a table or JSON — `docs/v1/mcp-cli.md`'s "A4 — CLI"
//! tests.

#![allow(clippy::unwrap_used, reason = "a panic is a failed test")]

mod common;

use common::{TestLibrary, library_args, life_pixel, stdout};
use serde_json::Value;

#[tokio::test]
async fn a_table_lists_the_header_then_the_seeded_animation() {
    let library = TestLibrary::seeded("Mascot").await;
    let mut args = vec!["list".to_owned()];
    args.extend(library_args(&library));

    let output = life_pixel(library.path(), &str_args(&args), &[]);

    assert!(output.status.success(), "{}", common::stderr(&output));
    let text = stdout(&output);
    assert!(
        text.starts_with("ID\tTITLE\tPROJECT\tSIZE\tFRAMES\tUPDATED"),
        "{text}"
    );
    assert!(text.contains("Mascot"), "{text}");
    assert!(
        text.contains(&library.animation.uuid().to_string()),
        "{text}"
    );
    assert!(text.contains(&library.project.uuid().to_string()), "{text}");
    assert!(text.contains("4x4"), "{text}");
}

#[tokio::test]
async fn json_lists_the_seeded_animation_with_camel_case_fields() {
    let library = TestLibrary::seeded("Mascot").await;
    let mut args = vec!["list".to_owned(), "--json".to_owned()];
    args.extend(library_args(&library));

    let output = life_pixel(library.path(), &str_args(&args), &[]);

    assert!(output.status.success(), "{}", common::stderr(&output));
    let value: Value = serde_json::from_str(&stdout(&output)).unwrap();
    let animations = value.as_array().unwrap();
    assert_eq!(animations.len(), 1);
    assert_eq!(animations[0]["title"], "Mascot");
    assert_eq!(animations[0]["id"], library.animation.uuid().to_string());
    assert_eq!(animations[0]["frameCount"], 1);
}

#[tokio::test]
async fn an_empty_library_prints_the_empty_message_in_a_table_and_an_empty_array_as_json() {
    let library = TestLibrary::empty();

    let table = life_pixel(library.path(), &["list", "--library", &path(&library)], &[]);
    let json = life_pixel(
        library.path(),
        &["list", "--library", &path(&library), "--json"],
        &[],
    );

    assert_eq!(stdout(&table).trim(), "No animations in this library.");
    let value: Value = serde_json::from_str(&stdout(&json)).unwrap();
    assert_eq!(value, serde_json::json!([]));
}

#[tokio::test]
async fn the_query_filter_keeps_only_a_matching_title() {
    let library = TestLibrary::seeded("Mascot").await;
    let args = [
        "list",
        "--library",
        &path(&library),
        "--query",
        "cat",
        "--json",
    ];

    let output = life_pixel(library.path(), &args, &[]);

    let value: Value = serde_json::from_str(&stdout(&output)).unwrap();
    assert_eq!(value, serde_json::json!([]));
}

fn path(library: &TestLibrary) -> String {
    library.path().display().to_string()
}

fn str_args(args: &[String]) -> Vec<&str> {
    args.iter().map(String::as_str).collect()
}
