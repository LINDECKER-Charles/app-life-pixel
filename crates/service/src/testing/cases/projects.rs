//! Projects: create, get, list, rename, delete, not found, and invisible to another owner.

use life_pixel_core::Name;
use time::OffsetDateTime;
use uuid::Uuid;

use super::{animation_ids, create_animation, create_project, document_write, project_ids};
use crate::ids::{AnimationId, ProjectId};
use crate::paging::PageRequest;
use crate::ports::library_store::{AnimationFilter, StoreError};
use crate::testing::{StoreFixture, new_animation};

/// A created project reads back with its name and no animation.
pub async fn project_is_created_and_read_back(fixture: StoreFixture) {
    let created = create_project(&fixture, "Mascots").await;

    let project = fixture
        .store()
        .get_project(&fixture.owner(), created.id)
        .await;

    let project = project.unwrap();
    assert_eq!(project.id, created.id);
    assert_eq!(project.name.as_str(), "Mascots");
    assert_eq!(project.animation_count, 0);
}

/// Every project of the owner is listed.
pub async fn projects_are_listed(fixture: StoreFixture) {
    let mut created = Vec::new();
    for name in ["One", "Two", "Three"] {
        created.push(create_project(&fixture, name).await.id);
    }

    let mut listed = project_ids(&fixture).await;

    listed.sort();
    created.sort();
    assert_eq!(listed, created);
}

/// A renamed project answers with, and keeps, its new name.
pub async fn project_is_renamed(fixture: StoreFixture) {
    let (store, owner) = (fixture.store(), fixture.owner());
    let created = create_project(&fixture, "Draft").await;
    let name = Name::new("Final").unwrap();

    let renamed = store.rename_project(&owner, created.id, name, OffsetDateTime::now_utc());

    assert_eq!(renamed.await.unwrap().name.as_str(), "Final");
    let project = store.get_project(&owner, created.id).await.unwrap();
    assert_eq!(project.name.as_str(), "Final");
}

/// A deleted project is gone from reads and lists.
pub async fn project_is_deleted(fixture: StoreFixture) {
    let (store, owner) = (fixture.store(), fixture.owner());
    let kept = create_project(&fixture, "Kept").await;
    let deleted = create_project(&fixture, "Deleted").await;

    store.delete_project(&owner, deleted.id).await.unwrap();

    let read = store.get_project(&owner, deleted.id).await;
    assert_eq!(read.unwrap_err(), StoreError::ProjectNotFound);
    assert_eq!(project_ids(&fixture).await, [kept.id]);
}

/// A project that does not exist is not found, wherever it is named.
pub async fn missing_project_is_not_found(fixture: StoreFixture) {
    let (store, owner) = (fixture.store(), fixture.owner());
    let missing = ProjectId::from_uuid(Uuid::now_v7());
    let name = Name::new("Nowhere").unwrap();
    let now = OffsetDateTime::now_utc();
    let project = create_project(&fixture, "Here").await;
    let animation = create_animation(&fixture, project.id, "Cat").await;

    let not_found = Err(StoreError::ProjectNotFound);
    assert_eq!(store.get_project(&owner, missing).await, not_found);
    let renamed = store.rename_project(&owner, missing, name, now).await;
    assert_eq!(renamed, not_found);
    assert_eq!(
        store.delete_project(&owner, missing).await,
        Err(StoreError::ProjectNotFound)
    );
    let created = store
        .create_animation(&owner, new_animation(missing, "Cat"), None)
        .await;
    assert_eq!(created, Err(StoreError::ProjectNotFound));
    let moved = store
        .move_animation(&owner, animation.id, missing, now)
        .await;
    assert_eq!(moved, Err(StoreError::ProjectNotFound));
}

/// Another owner sees none of the owner's projects and animations, and cannot change them.
pub async fn project_is_invisible_to_another_owner(fixture: StoreFixture) {
    let project = create_project(&fixture, "Private").await;
    let animation = create_animation(&fixture, project.id, "Secret").await;

    assert_project_invisible(&fixture, project.id).await;
    assert_animation_invisible(&fixture, animation.id, animation.version).await;

    let usage = fixture.store().usage(&fixture.other_owner()).await;
    assert_eq!(usage, Ok(0));
    let kept = animation_ids(&fixture, Some(project.id)).await;
    assert_eq!(kept, [animation.id]);
}

/// Another owner reads, lists, renames and deletes nothing of the project `id`.
async fn assert_project_invisible(fixture: &StoreFixture, id: ProjectId) {
    let (store, other) = (fixture.store(), fixture.other_owner());
    let name = Name::new("Taken").unwrap();
    let now = OffsetDateTime::now_utc();
    let not_found = StoreError::ProjectNotFound;
    let read = store.get_project(&other, id).await;
    assert_eq!(read.unwrap_err(), not_found);
    let renamed = store.rename_project(&other, id, name, now).await;
    assert_eq!(renamed.unwrap_err(), not_found);
    let deleted = store.delete_project(&other, id).await;
    assert_eq!(deleted.unwrap_err(), not_found);
    let listed = store.list_projects(&other, PageRequest::default()).await;
    assert!(listed.unwrap().items.iter().all(|project| project.id != id));
}

/// Another owner reads, lists, writes, moves and deletes nothing of the animation `id`.
async fn assert_animation_invisible(fixture: &StoreFixture, id: AnimationId, version: u64) {
    let (store, other) = (fixture.store(), fixture.other_owner());
    let project = create_project(fixture, "Elsewhere").await;
    let now = OffsetDateTime::now_utc();
    let not_found = StoreError::AnimationNotFound;
    let read = store.get_animation(&other, id).await;
    assert_eq!(read.unwrap_err(), not_found);
    let opened = store.read_document(&other, id).await;
    assert_eq!(opened.unwrap_err(), not_found);
    let (write, _) = document_write(id, version, "Stolen");
    let written = store.write_document(&other, write, None).await;
    assert_eq!(written.unwrap_err(), not_found);
    let moved = store.move_animation(&other, id, project.id, now).await;
    assert!(moved.is_err(), "another owner moved the animation");
    let deleted = store.delete_animation(&other, id).await;
    assert_eq!(deleted.unwrap_err(), not_found);
    let filter = AnimationFilter::default();
    let listed = store
        .list_animations(&other, filter, PageRequest::default())
        .await;
    assert!(
        listed
            .unwrap()
            .items
            .iter()
            .all(|animation| animation.id != id)
    );
}
