//! The document use cases — open, save, rename — and the owner's usage and deletion.

mod common;

use std::sync::Arc;

use common::unavailable::UnavailableStore;

use bytes::Bytes;
use common::{ACCOUNT, Harness, assert_code, assert_coded, canvas_params, name_params, sample};
use life_pixel_core::serialize::read_document;
use life_pixel_service::library::Usage;
use life_pixel_service::ports::ProductEvent;
use life_pixel_service::{AnimationId, Owner, PageRequest};
use serde_json::json;
use uuid::Uuid;

fn missing_animation() -> AnimationId {
    AnimationId::from_uuid(Uuid::from_u128(0xdead))
}

#[tokio::test]
async fn a_saved_document_gets_a_new_version_and_records_its_size() {
    let harness = Harness::default();
    let project = harness.project(&ACCOUNT, "Pets").await;
    let cat = harness.animation(project.id, "Cat").await;
    let (meta, document) = sample("Black cat");

    let saved = harness
        .library
        .save_document(&ACCOUNT, cat.id, cat.version, document.clone());

    let saved = saved.await.unwrap();
    assert_ne!(saved.version, cat.version);
    assert_eq!(saved.meta, meta);
    let (record, stored) = harness
        .library
        .open_document(&ACCOUNT, cat.id)
        .await
        .unwrap();
    assert_eq!((record, stored), (saved, document));
    let saved_event = ProductEvent {
        name: "document_saved",
        account: ACCOUNT.account(),
        properties: vec![("size", "lt_1k".to_owned())],
    };
    assert_eq!(harness.events().last(), Some(&saved_event));
}

#[tokio::test]
async fn a_save_on_a_stale_version_is_a_conflict_with_the_current_one() {
    let harness = Harness::default();
    let project = harness.project(&ACCOUNT, "Pets").await;
    let cat = harness.animation(project.id, "Cat").await;
    let library = &harness.library;
    let current = library.save_document(&ACCOUNT, cat.id, cat.version, sample("Black cat").1);
    let current = current.await.unwrap().version;

    let stale = library.save_document(&ACCOUNT, cat.id, cat.version, sample("White cat").1);

    let params = json!({ "current": current });
    assert_coded(
        &stale.await.unwrap_err(),
        "document.version_conflict",
        params,
    );
}

#[tokio::test]
async fn an_invalid_save_is_refused_before_anything_is_written() {
    let harness = Harness::default();
    let project = harness.project(&ACCOUNT, "Pets").await;
    let cat = harness.animation(project.id, "Cat").await;
    let mut invalid: serde_json::Value = serde_json::from_slice(&sample("Cat").1).unwrap();
    invalid["width"] = json!(0);
    let invalid = Bytes::from(serde_json::to_vec(&invalid).unwrap());
    let library = &harness.library;

    let refused = library
        .save_document(&ACCOUNT, cat.id, cat.version, invalid)
        .await;
    let missing = library.save_document(&ACCOUNT, missing_animation(), 1, sample("Cat").1);

    let canvas = canvas_params();
    assert_coded(&refused.unwrap_err(), "document.canvas_size", canvas);
    assert_code(&missing.await.unwrap_err(), "library.animation_not_found");
    let record = library.get_animation(&ACCOUNT, cat.id).await.unwrap();
    assert_eq!(record.version, cat.version);
}

#[tokio::test]
async fn a_save_growing_beyond_the_quota_is_refused_and_a_shrinking_one_passes() {
    let long = "A cat with a long title".repeat(4);
    let small = u64::try_from(sample("Cat").1.len()).unwrap();
    let large = u64::try_from(sample(&long).1.len()).unwrap();
    let harness = Harness::with_quota(small);
    let project = harness.project(&ACCOUNT, "Pets").await;
    let cat = harness.animation(project.id, "Cat").await;
    let library = &harness.library;

    let grown = library
        .save_document(&ACCOUNT, cat.id, cat.version, sample(&long).1)
        .await;
    let shrunk = library
        .save_document(&ACCOUNT, cat.id, cat.version, sample("C").1)
        .await;

    let params = json!({ "used": small, "limit": small, "requested": large - small });
    assert_coded(&grown.unwrap_err(), "quota.storage_exceeded", params);
    assert!(shrunk.is_ok());
    let names = harness.event_names();
    assert_eq!(
        names,
        ["animation_created", "quota_rejected", "document_saved"]
    );
}

#[tokio::test]
async fn a_renamed_animation_carries_its_title_in_its_document() {
    let harness = Harness::default();
    let project = harness.project(&ACCOUNT, "Pets").await;
    let cat = harness.animation(project.id, "Cat").await;
    let library = &harness.library;

    let renamed = library
        .rename_animation(&ACCOUNT, cat.id, cat.version, " Tabby ")
        .await;

    let renamed = renamed.unwrap();
    assert_eq!(renamed.meta.title.as_str(), "Tabby");
    assert_ne!(renamed.version, cat.version);
    let (_, document) = library.open_document(&ACCOUNT, cat.id).await.unwrap();
    assert_eq!(read_document(&document).unwrap().title().as_str(), "Tabby");
    assert_eq!(harness.event_names().last(), Some(&"document_saved"));
}

#[tokio::test]
async fn a_rename_is_refused_with_the_code_of_its_error() {
    let harness = Harness::default();
    let project = harness.project(&ACCOUNT, "Pets").await;
    let cat = harness.animation(project.id, "Cat").await;
    let library = &harness.library;

    let untitled = library
        .rename_animation(&ACCOUNT, cat.id, cat.version, "")
        .await;
    let stale = library
        .rename_animation(&ACCOUNT, cat.id, cat.version + 1, "Tabby")
        .await;
    let missing = library
        .rename_animation(&ACCOUNT, missing_animation(), 1, "Tabby")
        .await;

    assert_coded(&untitled.unwrap_err(), "document.name", name_params());
    let current = json!({ "current": cat.version });
    assert_coded(&stale.unwrap_err(), "document.version_conflict", current);
    assert_code(&missing.unwrap_err(), "library.animation_not_found");
    let missing = library.open_document(&ACCOUNT, missing_animation()).await;
    assert_code(&missing.unwrap_err(), "library.animation_not_found");
}

#[tokio::test]
async fn an_account_gets_the_free_quota_and_the_local_library_none() {
    let harness = Harness::with_quota(1);
    let project = harness.project(&Owner::Local, "Pets").await;
    let (_, document) = sample("Cat");
    let cat = harness
        .library
        .import_animation(&Owner::Local, project.id, document);
    let cat = cat.await.unwrap();

    let local = harness.library.usage(&Owner::Local).await.unwrap();
    let hosted = harness.library.usage(&ACCOUNT).await.unwrap();

    assert_eq!(
        local,
        Usage {
            used_bytes: cat.document_bytes,
            limit_bytes: None
        }
    );
    assert_eq!(
        hosted,
        Usage {
            used_bytes: 0,
            limit_bytes: Some(1)
        }
    );
    assert!(
        harness.events().is_empty(),
        "the local library records no event"
    );
}

#[tokio::test]
async fn deleting_everything_empties_the_owner_only() {
    let harness = Harness::default();
    let mine = harness.project(&ACCOUNT, "Pets").await;
    harness.animation(mine.id, "Cat").await;
    let local = harness.project(&Owner::Local, "Pets").await;

    harness.library.delete_everything(&ACCOUNT).await.unwrap();

    assert_eq!(harness.library.usage(&ACCOUNT).await.unwrap().used_bytes, 0);
    let listed = harness
        .library
        .list_projects(&ACCOUNT, PageRequest::default())
        .await;
    assert!(listed.unwrap().items.is_empty());
    assert!(
        harness
            .library
            .get_project(&Owner::Local, local.id)
            .await
            .is_ok()
    );
}

#[tokio::test]
async fn a_failing_store_is_service_unavailable_without_its_detail() {
    let harness = Harness::over(Arc::new(UnavailableStore), 1);
    let library = &harness.library;

    let created = library.create_project(&ACCOUNT, "Pets").await;
    let usage = library.usage(&ACCOUNT).await;
    let opened = library.open_document(&ACCOUNT, missing_animation()).await;

    for error in [
        created.unwrap_err(),
        usage.unwrap_err(),
        opened.unwrap_err(),
    ] {
        assert_code(&error, "service.unavailable");
        assert!(!error.to_string().contains("connection refused"));
    }
}
