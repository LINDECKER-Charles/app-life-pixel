//! The project use cases: create, read, list, rename, duplicate, delete.

mod common;

use common::{
    ACCOUNT, Harness, OTHER_ACCOUNT, START, assert_code, assert_coded, name_params, sample,
};
use life_pixel_service::memory::SequentialIds;
use life_pixel_service::ports::library_store::{AnimationFilter, AnimationRecord};
use life_pixel_service::{PageRequest, ProjectId};
use serde_json::json;
use time::Duration;
use uuid::Uuid;

fn missing_project() -> ProjectId {
    ProjectId::from_uuid(Uuid::from_u128(0xdead))
}

#[tokio::test]
async fn a_project_is_created_with_a_trimmed_name_a_new_id_and_the_time() {
    let harness = Harness::default();

    let project = harness
        .library
        .create_project(&ACCOUNT, "  Pets ")
        .await
        .unwrap();

    assert_eq!(project.id.uuid(), SequentialIds::id(1));
    assert_eq!(project.name.as_str(), "Pets");
    assert_eq!((project.created_at, project.updated_at), (START, START));
    let read = harness.library.get_project(&ACCOUNT, project.id).await;
    assert_eq!(read.unwrap(), project);
}

#[tokio::test]
async fn an_invalid_project_name_is_document_name() {
    let harness = Harness::default();

    for name in ["", "   ", "tab\there"] {
        let error = harness
            .library
            .create_project(&ACCOUNT, name)
            .await
            .unwrap_err();
        assert_coded(&error, "document.name", name_params());
    }
    let listed = harness
        .library
        .list_projects(&ACCOUNT, PageRequest::default())
        .await;
    assert!(listed.unwrap().items.is_empty());
}

#[tokio::test]
async fn a_missing_project_is_library_project_not_found() {
    let harness = Harness::default();
    let library = &harness.library;

    let read = library.get_project(&ACCOUNT, missing_project()).await;
    let renamed = library
        .rename_project(&ACCOUNT, missing_project(), "Name")
        .await;
    let deleted = library.delete_project(&ACCOUNT, missing_project()).await;
    let copied = library
        .duplicate_project(&ACCOUNT, missing_project(), "Copy")
        .await;

    assert_code(&read.unwrap_err(), "library.project_not_found");
    assert_code(&renamed.unwrap_err(), "library.project_not_found");
    assert_code(&deleted.unwrap_err(), "library.project_not_found");
    assert_code(&copied.unwrap_err(), "library.project_not_found");
}

#[tokio::test]
async fn projects_are_listed_from_the_most_recently_updated_by_page() {
    let harness = Harness::default();
    let mut created = Vec::new();
    for name in ["One", "Two", "Three"] {
        created.push(harness.project(&ACCOUNT, name).await.id);
        harness.clock.advance(Duration::seconds(1));
    }
    harness.project(&OTHER_ACCOUNT, "Not mine").await;

    let first = harness
        .library
        .list_projects(&ACCOUNT, PageRequest::new(None, Some(2)))
        .await;
    let first = first.unwrap();
    let rest = PageRequest::new(first.next_cursor, Some(2));
    let rest = harness.library.list_projects(&ACCOUNT, rest).await.unwrap();

    let listed: Vec<ProjectId> = first
        .items
        .iter()
        .chain(&rest.items)
        .map(|p| p.id)
        .collect();
    created.reverse();
    assert_eq!(listed, created);
    assert_eq!(rest.next_cursor, None);
}

#[tokio::test]
async fn a_renamed_project_takes_its_name_and_the_time() {
    let harness = Harness::default();
    let project = harness.project(&ACCOUNT, "Draft").await;
    harness.clock.advance(Duration::minutes(5));

    let renamed = harness
        .library
        .rename_project(&ACCOUNT, project.id, " Final ")
        .await;

    let renamed = renamed.unwrap();
    assert_eq!(renamed.name.as_str(), "Final");
    assert_eq!(renamed.updated_at, START + Duration::minutes(5));
    let invalid = harness
        .library
        .rename_project(&ACCOUNT, project.id, "")
        .await;
    assert_coded(&invalid.unwrap_err(), "document.name", name_params());
}

#[tokio::test]
async fn deleting_a_project_deletes_its_animations() {
    let harness = Harness::default();
    let project = harness.project(&ACCOUNT, "Pets").await;
    let animation = harness.animation(project.id, "Cat").await;

    harness
        .library
        .delete_project(&ACCOUNT, project.id)
        .await
        .unwrap();

    let opened = harness.library.open_document(&ACCOUNT, animation.id).await;
    assert_code(&opened.unwrap_err(), "library.animation_not_found");
    assert_eq!(harness.library.usage(&ACCOUNT).await.unwrap().used_bytes, 0);
}

#[tokio::test]
async fn a_duplicated_project_copies_its_animations_with_new_ids() {
    let harness = Harness::default();
    let project = harness.project(&ACCOUNT, "Pets").await;
    let cat = harness.animation(project.id, "Cat").await;
    let dog = harness.animation(project.id, "Dog").await;

    let copy = harness
        .library
        .duplicate_project(&ACCOUNT, project.id, "Pets 2");

    let copy = copy.await.unwrap();
    assert_eq!((copy.name.as_str(), copy.animation_count), ("Pets 2", 2));
    assert_ne!(copy.id, project.id);
    let copies = animations_of(&harness, copy.id).await;
    assert!(
        copies
            .iter()
            .all(|copied| ![cat.id, dog.id].contains(&copied.id))
    );
    let mut titles: Vec<&str> = copies
        .iter()
        .map(|copied| copied.meta.title.as_str())
        .collect();
    titles.sort_unstable();
    assert_eq!(titles, ["Cat", "Dog"]);
    let usage = harness.library.usage(&ACCOUNT).await.unwrap().used_bytes;
    assert_eq!(usage, 2 * (cat.document_bytes + dog.document_bytes));
}

/// The first page of the animations of `project`.
async fn animations_of(harness: &Harness, project: ProjectId) -> Vec<AnimationRecord> {
    let filter = AnimationFilter {
        project: Some(project),
        query: None,
    };
    let page = harness
        .library
        .list_animations(&ACCOUNT, filter, PageRequest::default());
    let page = page.await;
    page.unwrap_or_else(|error| panic!("listing failed: {error}"))
        .items
}

#[tokio::test]
async fn a_project_too_large_to_copy_is_refused_whole_before_anything_is_copied() {
    let bytes = u64::try_from(sample("Cat").1.len()).unwrap();
    let limit = bytes * 3 / 2;
    let harness = Harness::with_quota(limit);
    let project = harness.project(&ACCOUNT, "Pets").await;
    harness.animation(project.id, "Cat").await;

    let copied = harness
        .library
        .duplicate_project(&ACCOUNT, project.id, "Pets 2")
        .await;

    let params = json!({ "used": bytes, "limit": limit, "requested": bytes });
    assert_coded(&copied.unwrap_err(), "quota.storage_exceeded", params);
    let projects = harness
        .library
        .list_projects(&ACCOUNT, PageRequest::default())
        .await;
    assert_eq!(projects.unwrap().items.len(), 1);
    assert_eq!(harness.event_names().last(), Some(&"quota_rejected"));
    let invalid = harness
        .library
        .duplicate_project(&ACCOUNT, project.id, "")
        .await;
    assert_coded(&invalid.unwrap_err(), "document.name", name_params());
}
