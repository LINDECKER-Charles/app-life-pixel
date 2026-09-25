//! The animation and document commands.

use life_pixel_desktop::commands::library::*;
use life_pixel_desktop::commands::shapes::Animation;
use serde_json::{Value, json};

use crate::harness::{Harness, base64, code, decode, document};

/// A project named `name`, by id.
async fn project(harness: &Harness, name: &str) -> String {
    let project = library_create_project(harness.state(), name.into())
        .await
        .unwrap();
    serde_json::to_value(project).unwrap()["id"]
        .as_str()
        .unwrap()
        .to_owned()
}

async fn animation(harness: &Harness, project: &str, title: &str) -> Animation {
    let created = library_create_animation(harness.state(), project.into(), document(title));
    created.await.unwrap()
}

#[tokio::test]
async fn an_animation_has_the_shape_of_the_api() {
    let harness = Harness::new();
    let project = project(&harness, "P").await;
    let json = serde_json::to_value(animation(&harness, &project, "Walk").await).unwrap();
    let keys: Vec<&str> = json
        .as_object()
        .unwrap()
        .keys()
        .map(String::as_str)
        .collect();
    let mut expected = [
        "id",
        "projectId",
        "title",
        "width",
        "height",
        "frameCount",
        "documentBytes",
        "version",
        "createdAt",
        "updatedAt",
    ];
    expected.sort_unstable();
    let mut keys = keys;
    keys.sort_unstable();
    assert_eq!(keys, expected);
    assert_eq!(json["title"], "Walk");
    assert_eq!(json["projectId"], project.as_str());
    assert!(json["createdAt"].as_str().unwrap().ends_with('Z'));
}

#[tokio::test]
async fn a_document_is_opened_and_saved_as_base64_with_its_version() {
    let harness = Harness::new();
    let project = project(&harness, "P").await;
    let created = animation(&harness, &project, "Walk").await;
    let opened = library_open_document(harness.state(), created.id.clone())
        .await
        .unwrap();
    assert_eq!(opened.summary, created);
    assert_eq!(opened.document, document("Walk"));

    let saved = library_save_document(
        harness.state(),
        created.id.clone(),
        created.version,
        document("Walk, longer"),
    );
    let saved = saved.await.unwrap();
    assert_ne!(saved.version, created.version);
    let stale = library_save_document(harness.state(), created.id, created.version, document("X"));
    let error = stale.await.unwrap_err();
    assert_eq!(error.code, "document.version_conflict");
    assert_eq!(
        Value::Object(error.params),
        json!({ "current": saved.version })
    );
}

#[tokio::test]
async fn a_bad_document_is_refused_with_its_code() {
    let harness = Harness::new();
    let project = project(&harness, "P").await;
    let not_base64 = library_create_animation(harness.state(), project.clone(), "%%".into());
    assert_eq!(code(not_base64.await), "request.malformed");
    let not_json = library_create_animation(harness.state(), project, base64(b"nope"));
    assert_eq!(code(not_json.await), "document.malformed");
    let missing = library_open_document(harness.state(), uuid()).await;
    assert_eq!(code(missing), "library.animation_not_found");
}

#[tokio::test]
async fn animations_are_renamed_moved_duplicated_and_deleted() {
    let harness = Harness::new();
    let (first, second) = (project(&harness, "A").await, project(&harness, "B").await);
    let walk = animation(&harness, &first, "Walk").await;
    let renamed =
        library_rename_animation(harness.state(), walk.id.clone(), walk.version, "Run".into());
    let renamed = renamed.await.unwrap();
    let opened = library_open_document(harness.state(), walk.id.clone())
        .await
        .unwrap();
    assert!(
        String::from_utf8(decode(&opened.document))
            .unwrap()
            .contains("Run")
    );

    let moved = library_move_animation(harness.state(), walk.id.clone(), second.clone());
    assert_eq!(moved.await.unwrap().project_id, second);
    let copy = library_duplicate_animation(harness.state(), walk.id.clone(), "Run 2".into(), None);
    assert_eq!(copy.await.unwrap().project_id, second);
    let elsewhere = library_duplicate_animation(
        harness.state(),
        walk.id.clone(),
        "Run 3".into(),
        Some(first),
    );
    assert_ne!(elsewhere.await.unwrap().id, renamed.id);

    library_delete_animation(harness.state(), walk.id.clone())
        .await
        .unwrap();
    let gone = library_delete_animation(harness.state(), walk.id).await;
    assert_eq!(code(gone), "library.animation_not_found");
}

#[tokio::test]
async fn animations_are_listed_by_project_and_by_title() {
    let harness = Harness::new();
    let (first, second) = (project(&harness, "A").await, project(&harness, "B").await);
    animation(&harness, &first, "Walk").await;
    animation(&harness, &first, "Jump").await;
    animation(&harness, &second, "Walk back").await;
    let titles = |page: life_pixel_desktop::commands::shapes::ListPage<Animation>| {
        let json = serde_json::to_value(page).unwrap();
        let mut titles: Vec<String> = json["items"]
            .as_array()
            .unwrap()
            .iter()
            .map(|item| item["title"].as_str().unwrap().to_owned())
            .collect();
        titles.sort();
        titles
    };
    let of_first = library_list_animations(harness.state(), Some(first), None, None, None);
    assert_eq!(titles(of_first.await.unwrap()), ["Jump", "Walk"]);
    let walks = library_list_animations(harness.state(), None, Some("walk".into()), None, None);
    assert_eq!(titles(walks.await.unwrap()), ["Walk", "Walk back"]);
    let all = library_list_animations(harness.state(), None, None, None, Some(2));
    assert!(all.await.unwrap().next_cursor.is_some());
}

fn uuid() -> String {
    "0190a4b2-0000-7000-8000-000000000000".to_owned()
}
