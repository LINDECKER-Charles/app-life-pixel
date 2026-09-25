//! The quota: creates and writes beyond it refused with the exact numbers, shrinking writes
//! accepted above it, usage kept exact.

use super::{create_animation, create_project, document_write};
use crate::ports::library_store::StoreError;
use crate::testing::{StoreFixture, new_animation};

/// A short title: a small document.
const SHORT_TITLE: &str = "Cat";
/// A long title: a larger document.
const LONG_TITLE: &str =
    "A very long title for a cat that walks slowly across the whole screen, then naps in the sun";

/// A create that would take usage beyond the quota is refused with the usage, the quota and the
/// document's size, and stores nothing.
pub async fn create_beyond_the_quota_is_refused_with_exact_numbers(fixture: StoreFixture) {
    let (store, owner) = (fixture.store(), fixture.owner());
    let project = create_project(&fixture, "Pets").await;
    let first = create_animation(&fixture, project.id, SHORT_TITLE).await;
    let second = new_animation(project.id, LONG_TITLE);
    let requested = u64::try_from(second.document.len()).unwrap();
    let limit = first.document_bytes + requested - 1;

    let refused = store.create_animation(&owner, second, Some(limit)).await;

    let used = first.document_bytes;
    let exceeded = StoreError::QuotaExceeded {
        used,
        limit,
        requested,
    };
    assert_eq!(refused, Err(exceeded));
    assert_eq!(store.usage(&owner).await, Ok(used));
    let accepted = new_animation(project.id, LONG_TITLE);
    let limit = Some(used + requested);
    assert!(
        store
            .create_animation(&owner, accepted, limit)
            .await
            .is_ok()
    );
}

/// A write that would take usage beyond the quota is refused with the usage, the quota and the
/// bytes it would add, and changes nothing.
pub async fn write_beyond_the_quota_is_refused_with_exact_numbers(fixture: StoreFixture) {
    let (store, owner) = (fixture.store(), fixture.owner());
    let project = create_project(&fixture, "Pets").await;
    let created = create_animation(&fixture, project.id, SHORT_TITLE).await;
    let (write, document) = document_write(created.id, created.version, LONG_TITLE);
    let used = created.document_bytes;
    let requested = u64::try_from(document.len()).unwrap() - used;

    let refused = store.write_document(&owner, write, Some(used)).await;

    let exceeded = StoreError::QuotaExceeded {
        used,
        limit: used,
        requested,
    };
    assert_eq!(refused, Err(exceeded));
    let record = store.get_animation(&owner, created.id).await.unwrap();
    assert_eq!(record.version, created.version);
    assert_eq!(store.usage(&owner).await, Ok(used));
}

/// A write that shrinks a document passes even when usage stays above the quota.
pub async fn shrinking_write_above_the_quota_is_accepted(fixture: StoreFixture) {
    let (store, owner) = (fixture.store(), fixture.owner());
    let project = create_project(&fixture, "Pets").await;
    let created = create_animation(&fixture, project.id, LONG_TITLE).await;
    let (write, document) = document_write(created.id, created.version, SHORT_TITLE);
    let quota = Some(1);

    let written = store.write_document(&owner, write, quota).await.unwrap();

    let size = u64::try_from(document.len()).unwrap();
    assert_eq!(written.document_bytes, size);
    assert_eq!(store.usage(&owner).await, Ok(size));
}

/// Usage is the bytes of the owner's documents after a create, a write and a delete.
pub async fn usage_follows_create_write_and_delete(fixture: StoreFixture) {
    let (store, owner) = (fixture.store(), fixture.owner());
    let project = create_project(&fixture, "Pets").await;
    assert_eq!(store.usage(&owner).await, Ok(0));
    let first = create_animation(&fixture, project.id, SHORT_TITLE).await;
    assert_eq!(store.usage(&owner).await, Ok(first.document_bytes));

    let (write, _) = document_write(first.id, first.version, LONG_TITLE);
    let written = store.write_document(&owner, write, None).await.unwrap();
    let second = create_animation(&fixture, project.id, SHORT_TITLE).await;
    let both = written.document_bytes + second.document_bytes;
    assert_eq!(store.usage(&owner).await, Ok(both));

    store.delete_animation(&owner, first.id).await.unwrap();
    assert_eq!(store.usage(&owner).await, Ok(second.document_bytes));
}

/// Deleting a project frees the bytes of its animations, and theirs only.
pub async fn deleting_a_project_frees_its_animations_usage(fixture: StoreFixture) {
    let (store, owner) = (fixture.store(), fixture.owner());
    let deleted = create_project(&fixture, "Old").await;
    let kept = create_project(&fixture, "New").await;
    let gone = create_animation(&fixture, deleted.id, LONG_TITLE).await;
    create_animation(&fixture, deleted.id, SHORT_TITLE).await;
    let remaining = create_animation(&fixture, kept.id, SHORT_TITLE).await;

    store.delete_project(&owner, deleted.id).await.unwrap();

    assert_eq!(store.usage(&owner).await, Ok(remaining.document_bytes));
    let read = store.get_animation(&owner, gone.id).await;
    assert_eq!(read, Err(StoreError::AnimationNotFound));
}
