//! Writes: the right version gives a new one, a wrong one conflicts, and of two concurrent writes
//! on one version exactly one wins.

use super::{create_animation, create_project, document_write};
use crate::ports::library_store::StoreError;
use crate::testing::StoreFixture;

/// The versions a JavaScript number carries exactly: below 2^53.
const VERSION_BOUND: u64 = 1 << 53;

/// A write expecting the current version replaces the document and gives a new version.
pub async fn write_with_the_right_version_gives_a_new_version(fixture: StoreFixture) {
    let (store, owner) = (fixture.store(), fixture.owner());
    let project = create_project(&fixture, "Pets").await;
    let created = create_animation(&fixture, project.id, "Cat").await;
    let (write, document) = document_write(created.id, created.version, "Black cat");
    let meta = write.meta.clone();

    let written = store.write_document(&owner, write, None).await.unwrap();

    assert_ne!(written.version, created.version);
    assert!(created.version < VERSION_BOUND && written.version < VERSION_BOUND);
    assert_eq!(written.meta, meta);
    assert_eq!(
        written.document_bytes,
        u64::try_from(document.len()).unwrap()
    );
    let (record, bytes) = store.read_document(&owner, created.id).await.unwrap();
    assert_eq!((record.version, bytes), (written.version, document));
}

/// A write expecting a version that is no longer current is refused with the current one, and
/// changes nothing.
pub async fn write_with_a_wrong_version_is_a_conflict(fixture: StoreFixture) {
    let (store, owner) = (fixture.store(), fixture.owner());
    let project = create_project(&fixture, "Pets").await;
    let created = create_animation(&fixture, project.id, "Cat").await;
    let (write, document) = document_write(created.id, created.version, "Black cat");
    let current = store
        .write_document(&owner, write, None)
        .await
        .unwrap()
        .version;

    let (stale, _) = document_write(created.id, created.version, "White cat");
    let refused = store.write_document(&owner, stale, None).await;

    assert_eq!(refused, Err(StoreError::VersionConflict { current }));
    let (record, bytes) = store.read_document(&owner, created.id).await.unwrap();
    assert_eq!((record.version, bytes), (current, document));
}

/// Two writes expecting the same version at the same time: one wins, the other conflicts with
/// the winner's version, and the winner's document is kept.
pub async fn concurrent_writes_on_one_version_let_exactly_one_win(fixture: StoreFixture) {
    let owner = fixture.owner();
    let project = create_project(&fixture, "Pets").await;
    let created = create_animation(&fixture, project.id, "Cat").await;
    let writes = ["Black cat", "White cat"].map(|title| {
        let (write, document) = document_write(created.id, created.version, title);
        let store = fixture.store().clone();
        let task = tokio::spawn(async move { store.write_document(&owner, write, None).await });
        (task, document)
    });

    let mut outcomes = Vec::new();
    for (task, document) in writes {
        outcomes.push((task.await.unwrap(), document));
    }

    let (won, lost): (Vec<_>, Vec<_>) =
        outcomes.into_iter().partition(|(result, _)| result.is_ok());
    assert_eq!((won.len(), lost.len()), (1, 1), "exactly one write wins");
    let (winner, document) = (won[0].0.clone().unwrap(), won[0].1.clone());
    let current = winner.version;
    assert_eq!(lost[0].0, Err(StoreError::VersionConflict { current }));
    let (_, bytes) = fixture
        .store()
        .read_document(&owner, created.id)
        .await
        .unwrap();
    assert_eq!(bytes, document);
}
