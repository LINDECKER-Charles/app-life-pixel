//! The project commands and the library's usage.

use life_pixel_desktop::commands::library::*;
use serde_json::Value;

use crate::harness::{Harness, code, document};

/// A new project named `name`, as JSON.
async fn create(harness: &Harness, name: &str) -> Value {
    let project = library_create_project(harness.state(), name.into());
    serde_json::to_value(project.await.unwrap()).unwrap()
}

#[tokio::test]
async fn a_project_is_created_empty_then_renamed() {
    let harness = Harness::new();
    let walk = create(&harness, "Walk").await;
    assert_eq!(walk["name"], "Walk");
    assert_eq!(walk["animationCount"], 0);
    let id = walk["id"].as_str().unwrap().to_owned();
    let renamed = library_rename_project(harness.state(), id, "Run".into());
    assert_eq!(
        serde_json::to_value(renamed.await.unwrap()).unwrap()["name"],
        "Run"
    );
}

#[tokio::test]
async fn a_project_is_duplicated_with_its_animations_then_deleted() {
    let harness = Harness::new();
    let id = create(&harness, "Walk").await["id"]
        .as_str()
        .unwrap()
        .to_owned();
    let step = library_create_animation(harness.state(), id.clone(), document("Step"));
    step.await.unwrap();
    let copy = library_duplicate_project(harness.state(), id.clone(), "Walk 2".into());
    assert_eq!(
        serde_json::to_value(copy.await.unwrap()).unwrap()["animationCount"],
        1
    );

    library_delete_project(harness.state(), id.clone())
        .await
        .unwrap();
    let page = library_list_projects(harness.state(), None, None)
        .await
        .unwrap();
    let page = serde_json::to_value(page).unwrap();
    assert_eq!(page["items"].as_array().unwrap().len(), 1);
    assert_eq!(page["items"][0]["name"], "Walk 2");
    assert_eq!(page["nextCursor"], Value::Null);
    let deleted_again = library_delete_project(harness.state(), id).await;
    assert_eq!(code(deleted_again), "library.project_not_found");
}

#[tokio::test]
async fn projects_are_listed_page_by_page() {
    let harness = Harness::new();
    for name in ["One", "Two", "Three"] {
        library_create_project(harness.state(), name.into())
            .await
            .unwrap();
    }
    let first = library_list_projects(harness.state(), None, Some(2))
        .await
        .unwrap();
    assert_eq!(first.items.len(), 2);
    let cursor = first.next_cursor.clone();
    assert!(cursor.is_some());
    let second = library_list_projects(harness.state(), cursor, Some(2))
        .await
        .unwrap();
    assert_eq!(second.items.len(), 1);
    assert_eq!(second.next_cursor, None);
}

#[tokio::test]
async fn a_bad_argument_is_request_malformed() {
    let harness = Harness::new();
    let state = || harness.state();
    let bad_cursor = library_list_projects(state(), Some("!".into()), None).await;
    assert_eq!(code(bad_cursor), "request.malformed");
    let bad_id = library_rename_project(state(), "nope".into(), "Name".into()).await;
    assert_eq!(code(bad_id), "request.malformed");
    let bad_name = library_create_project(state(), String::new()).await;
    assert_eq!(code(bad_name), "document.name");
}

#[tokio::test]
async fn the_usage_counts_the_documents_without_a_quota() {
    let harness = Harness::new();
    let project = library_create_project(harness.state(), "P".into())
        .await
        .unwrap();
    let project = serde_json::to_value(project).unwrap()["id"]
        .as_str()
        .unwrap()
        .to_owned();
    let animation = library_create_animation(harness.state(), project, document("A")).await;
    let bytes = serde_json::to_value(animation.unwrap()).unwrap()["documentBytes"].clone();
    let usage = serde_json::to_value(library_usage(harness.state()).await.unwrap()).unwrap();
    assert_eq!(
        usage,
        serde_json::json!({ "usedBytes": bytes, "limitBytes": null })
    );
}
