//! The animation use cases: create, import, read, list, move, duplicate, delete.

mod common;

use bytes::Bytes;
use common::{
    ACCOUNT, Harness, START, assert_code, assert_coded, canvas_params, name_params, sample, spec,
};
use life_pixel_core::limits::MAX_DOCUMENT_BYTES;
use life_pixel_core::serialize::read_document;
use life_pixel_service::memory::SequentialIds;
use life_pixel_service::ports::ProductEvent;
use life_pixel_service::ports::library_store::AnimationFilter;
use life_pixel_service::{AnimationId, PageRequest, ProjectId};
use serde_json::json;
use time::Duration;
use uuid::Uuid;

fn missing_animation() -> AnimationId {
    AnimationId::from_uuid(Uuid::from_u128(0xdead))
}

fn missing_project() -> ProjectId {
    ProjectId::from_uuid(Uuid::from_u128(0xdead))
}

#[tokio::test]
async fn a_blank_animation_is_created_from_a_spec_and_recorded() {
    let harness = Harness::default();
    let project = harness.project(&ACCOUNT, "Pets").await;

    let created = harness
        .library
        .create_animation(&ACCOUNT, project.id, spec("Cat", 16, 8));

    let created = created.await.unwrap();
    assert_eq!(created.id.uuid(), SequentialIds::id(2));
    assert_eq!(created.meta.title.as_str(), "Cat");
    assert_eq!(
        (
            created.meta.width,
            created.meta.height,
            created.meta.frame_count
        ),
        (16, 8, 1)
    );
    assert_eq!(created.created_at, START);
    let (_, document) = harness
        .library
        .open_document(&ACCOUNT, created.id)
        .await
        .unwrap();
    assert_eq!(read_document(&document).unwrap().width(), 16);
    let account_id = ACCOUNT.account();
    let created_event = ProductEvent {
        name: "animation_created",
        account: account_id,
        properties: Vec::new(),
    };
    assert_eq!(harness.events(), [created_event]);
}

#[tokio::test]
async fn an_invalid_spec_or_a_missing_project_creates_nothing() {
    let harness = Harness::default();
    let project = harness.project(&ACCOUNT, "Pets").await;

    let too_wide = harness
        .library
        .create_animation(&ACCOUNT, project.id, spec("Cat", 513, 8));
    let nowhere = harness
        .library
        .create_animation(&ACCOUNT, missing_project(), spec("Cat", 8, 8));

    let canvas = canvas_params();
    assert_coded(&too_wide.await.unwrap_err(), "document.canvas_size", canvas);
    assert_code(&nowhere.await.unwrap_err(), "library.project_not_found");
    assert!(harness.events().is_empty());
}

#[tokio::test]
async fn an_imported_document_is_stored_as_core_serializes_it() {
    let harness = Harness::default();
    let project = harness.project(&ACCOUNT, "Pets").await;
    let (meta, canonical) = sample("Cat");
    let value: serde_json::Value = serde_json::from_slice(&canonical).unwrap();
    let compact = Bytes::from(serde_json::to_vec(&value).unwrap());

    let imported = harness
        .library
        .import_animation(&ACCOUNT, project.id, compact)
        .await;

    let imported = imported.unwrap();
    assert_eq!(imported.meta, meta);
    let (_, stored) = harness
        .library
        .open_document(&ACCOUNT, imported.id)
        .await
        .unwrap();
    assert_eq!(stored, canonical);
}

#[tokio::test]
async fn an_invalid_import_is_refused_with_the_code_of_core() {
    let harness = Harness::default();
    let project = harness.project(&ACCOUNT, "Pets").await;
    let too_large = Bytes::from(vec![b' '; MAX_DOCUMENT_BYTES + 1]);

    let malformed = harness
        .library
        .import_animation(&ACCOUNT, project.id, "{".into())
        .await;
    let oversized = harness
        .library
        .import_animation(&ACCOUNT, project.id, too_large)
        .await;

    assert_code(&malformed.unwrap_err(), "document.malformed");
    let max_bytes = json!({ "maxBytes": MAX_DOCUMENT_BYTES });
    assert_coded(&oversized.unwrap_err(), "document.too_large", max_bytes);
}

#[tokio::test]
async fn a_create_beyond_the_free_quota_is_refused_and_recorded() {
    let bytes = u64::try_from(sample("Cat").1.len()).unwrap();
    let harness = Harness::with_quota(bytes + 1);
    let project = harness.project(&ACCOUNT, "Pets").await;
    harness.animation(project.id, "Cat").await;

    let refused = harness
        .library
        .import_animation(&ACCOUNT, project.id, sample("Cat").1)
        .await;

    let params = json!({ "used": bytes, "limit": bytes + 1, "requested": bytes });
    assert_coded(&refused.unwrap_err(), "quota.storage_exceeded", params);
    assert_eq!(
        harness.event_names(),
        ["animation_created", "quota_rejected"]
    );
}

#[tokio::test]
async fn animations_are_listed_by_project_and_searched() {
    let harness = Harness::default();
    let pets = harness.project(&ACCOUNT, "Pets").await;
    let plants = harness.project(&ACCOUNT, "Plants").await;
    let cat = harness.animation(pets.id, "Walking Cat").await;
    harness.animation(pets.id, "Dog").await;
    let fern = harness.animation(plants.id, "Fern").await;

    let in_plants = AnimationFilter {
        project: Some(plants.id),
        query: None,
    };
    let searched = AnimationFilter {
        project: None,
        query: Some("CAT".to_owned()),
    };
    let library = &harness.library;
    let in_plants = library
        .list_animations(&ACCOUNT, in_plants, PageRequest::default())
        .await;
    let searched = library
        .list_animations(&ACCOUNT, searched, PageRequest::default())
        .await;

    assert_eq!(in_plants.unwrap().items, [fern]);
    assert_eq!(searched.unwrap().items, std::slice::from_ref(&cat));
    assert_eq!(library.get_animation(&ACCOUNT, cat.id).await.unwrap(), cat);
    let missing = library.get_animation(&ACCOUNT, missing_animation()).await;
    assert_code(&missing.unwrap_err(), "library.animation_not_found");
}

#[tokio::test]
async fn an_animation_moves_to_an_existing_project() {
    let harness = Harness::default();
    let from = harness.project(&ACCOUNT, "Drafts").await;
    let to = harness.project(&ACCOUNT, "Final").await;
    let animation = harness.animation(from.id, "Cat").await;
    harness.clock.advance(Duration::hours(1));
    let library = &harness.library;

    let moved = library
        .move_animation(&ACCOUNT, animation.id, to.id)
        .await
        .unwrap();
    let nowhere = library
        .move_animation(&ACCOUNT, animation.id, missing_project())
        .await;
    let missing = library
        .move_animation(&ACCOUNT, missing_animation(), to.id)
        .await;

    assert_eq!(
        (moved.project, moved.updated_at),
        (to.id, START + Duration::hours(1))
    );
    assert_code(&nowhere.unwrap_err(), "library.project_not_found");
    assert_code(&missing.unwrap_err(), "library.animation_not_found");
}

#[tokio::test]
async fn a_duplicate_gets_a_new_id_and_its_title() {
    let harness = Harness::default();
    let pets = harness.project(&ACCOUNT, "Pets").await;
    let other = harness.project(&ACCOUNT, "Other").await;
    let cat = harness.animation(pets.id, "Cat").await;
    let library = &harness.library;

    let beside = library
        .duplicate_animation(&ACCOUNT, cat.id, "Cat 2", None)
        .await
        .unwrap();
    let there = library
        .duplicate_animation(&ACCOUNT, cat.id, "Cat 3", Some(other.id))
        .await;

    assert_ne!(beside.id, cat.id);
    assert_eq!(
        (beside.project, beside.meta.title.as_str()),
        (pets.id, "Cat 2")
    );
    let (_, document) = library.open_document(&ACCOUNT, beside.id).await.unwrap();
    assert_eq!(read_document(&document).unwrap().title().as_str(), "Cat 2");
    assert_eq!(there.unwrap().project, other.id);
    assert_eq!(harness.event_names(), ["animation_created"; 3]);
}

#[tokio::test]
async fn a_duplicate_is_refused_with_the_code_of_its_error() {
    let bytes = u64::try_from(sample("Cat").1.len()).unwrap();
    let harness = Harness::with_quota(bytes + 1);
    let pets = harness.project(&ACCOUNT, "Pets").await;
    let cat = harness.animation(pets.id, "Cat").await;
    let library = &harness.library;

    let untitled = library
        .duplicate_animation(&ACCOUNT, cat.id, " ", None)
        .await;
    let missing = library
        .duplicate_animation(&ACCOUNT, missing_animation(), "Cat", None)
        .await;
    let nowhere = library.duplicate_animation(&ACCOUNT, cat.id, "Cat", Some(missing_project()));
    let full = library
        .duplicate_animation(&ACCOUNT, cat.id, "Cat", None)
        .await;

    assert_coded(&untitled.unwrap_err(), "document.name", name_params());
    assert_code(&missing.unwrap_err(), "library.animation_not_found");
    assert_code(&nowhere.await.unwrap_err(), "library.project_not_found");
    let params = json!({ "used": bytes, "limit": bytes + 1, "requested": bytes });
    assert_coded(&full.unwrap_err(), "quota.storage_exceeded", params);
}

#[tokio::test]
async fn a_deleted_animation_is_gone() {
    let harness = Harness::default();
    let project = harness.project(&ACCOUNT, "Pets").await;
    let cat = harness.animation(project.id, "Cat").await;

    harness
        .library
        .delete_animation(&ACCOUNT, cat.id)
        .await
        .unwrap();

    let again = harness.library.delete_animation(&ACCOUNT, cat.id).await;
    assert_code(&again.unwrap_err(), "library.animation_not_found");
    assert_eq!(harness.library.usage(&ACCOUNT).await.unwrap().used_bytes, 0);
}
