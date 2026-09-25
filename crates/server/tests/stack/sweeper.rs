//! The sweeper on the local stack: it deletes the old objects no row references, by batches of
//! keys, and keeps those a row points at.

use std::sync::Arc;
use std::time::Duration;

use crate::common::Stack;
use bytes::Bytes;
use life_pixel_server::config::StorageConfig;
use life_pixel_server::storage::{
    HostedLibraryStore, SWEEP_BATCH_KEYS, Sweeper, animation_prefix, object_store,
};
use life_pixel_server::testing::TestDatabase;
use life_pixel_service::ports::LibraryStore;
use life_pixel_service::ports::library_store::AnimationRecord;
use life_pixel_service::testing::{new_animation, new_project};
use life_pixel_service::{AccountId, Owner};
use object_store::path::Path;
use object_store::{ObjectStore, ObjectStoreExt};

/// A new animation of `owner`, in a new project.
async fn animation(store: &HostedLibraryStore, owner: &Owner) -> AnimationRecord {
    let project = new_project("Pets");
    store.create_project(owner, project.clone()).await.unwrap();
    let new = new_animation(project.id, "Cat");
    store.create_animation(owner, new, None).await.unwrap()
}

/// Puts an unreferenced object named `name` beside the documents of `record`.
async fn put_orphan(
    objects: &dyn ObjectStore,
    (account, record): (AccountId, &AnimationRecord),
    name: &str,
) {
    let orphan = animation_prefix(account, record.id).join(name);
    objects
        .put(&orphan, Bytes::from_static(b"{}").into())
        .await
        .unwrap();
}

#[tokio::test]
async fn the_sweeper_keeps_the_documents_rows_point_at() {
    let stack = Stack::new().await;
    let created = animation(&stack.store, &stack.owner()).await;
    let objects = stack.storage.objects();
    put_orphan(objects.as_ref(), (stack.account, &created), "orphan.json").await;

    let sweeper = Sweeper::new(stack.database.pool().clone(), objects);
    assert_eq!(sweeper.sweep().await.unwrap(), 0, "a young orphan is kept");
    let deleted = sweeper.with_min_age(Duration::ZERO).sweep().await.unwrap();

    assert_eq!(deleted, 1);
    let keys = stack.storage.keys().await.unwrap();
    assert_eq!(keys.len(), 1);
    assert!(!keys[0].ends_with("orphan.json"), "{keys:?}");
    let read = stack.store.read_document(&stack.owner(), created.id).await;
    assert!(read.is_ok());
}

#[tokio::test]
async fn the_sweeper_checks_the_keys_by_batches() {
    let database = TestDatabase::create().await.unwrap();
    let folder = tempfile::tempdir().unwrap();
    let root = folder.path().join("objects");
    let objects = object_store(&StorageConfig::File { root }).unwrap();
    let store = HostedLibraryStore::new(database.pool().clone(), Arc::clone(&objects));
    let account = database.create_account().await.unwrap();
    let created = animation(&store, &Owner::Account(account)).await;
    for index in 0..=SWEEP_BATCH_KEYS {
        let name = format!("orphan-{index:04}.json");
        put_orphan(objects.as_ref(), (account, &created), &name).await;
    }

    let sweeper = Sweeper::new(database.pool().clone(), Arc::clone(&objects));
    let deleted = sweeper.with_min_age(Duration::ZERO).sweep().await.unwrap();

    assert_eq!(deleted, u64::try_from(SWEEP_BATCH_KEYS + 1).unwrap());
    let prefix = Path::from("documents");
    let left: Vec<_> = futures_util::TryStreamExt::try_collect(objects.list(Some(&prefix)))
        .await
        .unwrap();
    assert_eq!(left.len(), 1);
    let read = store
        .read_document(&Owner::Account(account), created.id)
        .await;
    assert!(read.is_ok());
}
