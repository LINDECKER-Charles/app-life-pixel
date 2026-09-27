//! The three-step write on the local stack: concurrent writes on one version, a failure after the
//! object write and the sweeper collecting its orphan, and the objects a deletion frees.

use std::time::Duration;

use crate::common::Stack;
use bytes::Bytes;
use life_pixel_server::storage::{Sweeper, animation_prefix};
use life_pixel_service::ports::LibraryStore;
use life_pixel_service::ports::library_store::{AnimationRecord, DocumentWrite, StoreError};
use life_pixel_service::testing::{new_animation, new_project, sample_document};
use life_pixel_service::{Owner, ProjectId};
use time::OffsetDateTime;

/// How many writes race on one version.
const RACING_WRITES: usize = 8;

const INJECT_FAILURE: &str = "
    create function lp_test_fail() returns trigger language plpgsql
      as $$ begin raise exception 'injected failure'; end $$;
    create trigger lp_test_fail before update on animations
      for each row execute function lp_test_fail();";
const REMOVE_FAILURE: &str = "drop trigger lp_test_fail on animations";

async fn project(stack: &Stack) -> ProjectId {
    let project = new_project("Pets");
    stack
        .store
        .create_project(&stack.owner(), project.clone())
        .await
        .unwrap();
    project.id
}

async fn animation(stack: &Stack, title: &str) -> AnimationRecord {
    let new = new_animation(project(stack).await, title);
    stack
        .store
        .create_animation(&stack.owner(), new, None)
        .await
        .unwrap()
}

fn write(record: &AnimationRecord, title: &str) -> DocumentWrite {
    let (meta, document) = sample_document(title);
    DocumentWrite {
        id: record.id,
        expected_version: record.version,
        meta,
        document,
        at: OffsetDateTime::now_utc(),
    }
}

/// The keys of the animation's documents, as the test's prefix lists them.
async fn document_keys(stack: &Stack, record: &AnimationRecord) -> Vec<String> {
    let prefix = animation_prefix(stack.account, record.id).to_string();
    let keys = stack.storage.keys().await.unwrap();
    keys.into_iter()
        .filter(|key| key.starts_with(&prefix))
        .collect()
}

/// Runs the statements of `sql` on the test database.
async fn execute(stack: &Stack, sql: &'static str) {
    sqlx::raw_sql(sql)
        .execute(stack.database.pool())
        .await
        .unwrap();
}

/// The animation's current document.
async fn document(stack: &Stack, record: &AnimationRecord) -> (AnimationRecord, Bytes) {
    let owner = stack.owner();
    stack.store.read_document(&owner, record.id).await.unwrap()
}

async fn assert_usage_exact(stack: &Stack) {
    let used = stack.store.usage(&stack.owner()).await.unwrap();
    assert_eq!(i64::try_from(used).unwrap(), stack.referenced_bytes().await);
    assert_eq!(stack.stored_usage().await, stack.referenced_bytes().await);
}

#[tokio::test]
async fn racing_writes_on_one_version_let_one_win_and_leave_one_object() {
    let stack = Stack::new().await;
    let created = animation(&stack, "Cat").await;
    let tasks: Vec<_> = (0..RACING_WRITES)
        .map(|index| {
            let (store, owner) = (stack.store.clone(), stack.owner());
            let write = write(&created, &format!("Cat number {index}"));
            tokio::spawn(async move { store.write_document(&owner, write, None).await })
        })
        .collect();

    let mut outcomes = Vec::new();
    for task in tasks {
        outcomes.push(task.await.unwrap());
    }

    let (won, lost): (Vec<_>, Vec<_>) = outcomes.into_iter().partition(Result::is_ok);
    assert_eq!(won.len(), 1, "exactly one write wins");
    let winner = won[0].clone().unwrap();
    assert_eq!(winner.version, created.version + 1);
    let conflict = Err(StoreError::VersionConflict {
        current: winner.version,
    });
    assert!(lost.iter().all(|outcome| *outcome == conflict), "{lost:?}");
    assert_eq!(document_keys(&stack, &created).await.len(), 1);
    assert_usage_exact(&stack).await;
}

#[tokio::test]
async fn a_failure_after_the_object_write_leaves_usage_exact_and_the_orphan_swept() {
    let stack = Stack::new().await;
    let created = animation(&stack, "Cat").await;
    let (_, original) = document(&stack, &created).await;
    execute(&stack, INJECT_FAILURE).await;
    stack.objects.fail(true);

    let owner = stack.owner();
    let failed = stack
        .store
        .write_document(&owner, write(&created, "Black cat"), None);

    assert!(matches!(failed.await, Err(StoreError::Unavailable(_))));
    stack.objects.fail(false);
    execute(&stack, REMOVE_FAILURE).await;
    assert_usage_exact(&stack).await;
    let (record, bytes) = document(&stack, &created).await;
    assert_eq!((record.version, bytes), (created.version, original.clone()));
    assert_eq!(
        document_keys(&stack, &created).await.len(),
        2,
        "the orphan is left"
    );

    let sweeper = Sweeper::new(stack.database.pool().clone(), stack.storage.objects());
    assert_eq!(sweeper.sweep().await.unwrap(), 0, "a young orphan is kept");
    let sweeper = sweeper.with_min_age(Duration::ZERO);
    assert_eq!(sweeper.sweep().await.unwrap(), 1);

    assert_eq!(document_keys(&stack, &created).await.len(), 1);
    assert_eq!(document(&stack, &created).await.1, original);
    assert_usage_exact(&stack).await;
}

#[tokio::test]
async fn a_failed_or_refused_write_deletes_its_object() {
    let stack = Stack::new().await;
    let created = animation(&stack, "Cat").await;
    execute(&stack, INJECT_FAILURE).await;
    let owner = stack.owner();
    let failed = stack
        .store
        .write_document(&owner, write(&created, "Black cat"), None);
    assert!(failed.await.is_err());
    execute(&stack, REMOVE_FAILURE).await;

    let mut stale = write(&created, "White cat");
    stale.expected_version += 1;
    let conflict = stack
        .store
        .write_document(&stack.owner(), stale, None)
        .await;
    let over_quota = stack
        .store
        .write_document(
            &stack.owner(),
            write(&created, "A much longer title"),
            Some(1),
        )
        .await;

    assert!(matches!(conflict, Err(StoreError::VersionConflict { .. })));
    assert!(matches!(over_quota, Err(StoreError::QuotaExceeded { .. })));
    assert_eq!(document_keys(&stack, &created).await.len(), 1);
    assert_usage_exact(&stack).await;
}

#[tokio::test]
async fn a_write_replaces_its_object_and_deletions_free_theirs() {
    let stack = Stack::new().await;
    let kept = animation(&stack, "Dog").await;
    let deleted = animation(&stack, "Cat").await;
    let owner = stack.owner();
    let written = stack
        .store
        .write_document(&owner, write(&kept, "Big dog"), None);
    written.await.unwrap();
    assert_eq!(
        document_keys(&stack, &kept).await.len(),
        1,
        "the old object is deleted"
    );

    stack
        .store
        .delete_animation(&stack.owner(), deleted.id)
        .await
        .unwrap();
    assert_eq!(document_keys(&stack, &deleted).await, Vec::<String>::new());
    stack
        .store
        .delete_project(&stack.owner(), kept.project)
        .await
        .unwrap();
    assert_eq!(stack.storage.keys().await.unwrap(), Vec::<String>::new());
    assert_usage_exact(&stack).await;
    assert_eq!(stack.stored_usage().await, 0);
}

#[tokio::test]
async fn delete_everything_frees_every_object() {
    let stack = Stack::new().await;
    animation(&stack, "Dog").await;
    animation(&stack, "Cat").await;

    stack.store.delete_everything(&stack.owner()).await.unwrap();

    assert_eq!(stack.storage.keys().await.unwrap(), Vec::<String>::new());
    assert_eq!(stack.stored_usage().await, 0);
}

#[tokio::test]
async fn the_hosted_store_serves_accounts_only() {
    let stack = Stack::new().await;

    let usage = stack.store.usage(&Owner::Local).await;
    let project = stack
        .store
        .create_project(&Owner::Local, new_project("Local"))
        .await;

    assert!(matches!(usage, Err(StoreError::Unavailable(_))));
    assert!(matches!(project, Err(StoreError::Unavailable(_))));
}
