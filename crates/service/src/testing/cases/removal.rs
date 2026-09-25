//! Move, delete, and `delete_everything`.

use time::OffsetDateTime;

use super::{animation_ids, create_animation, create_project, project_ids};
use crate::ports::library_store::StoreError;
use crate::testing::StoreFixture;

/// A moved animation belongs to its new project, and leaves the old one.
pub async fn animation_is_moved(fixture: StoreFixture) {
    let (store, owner) = (fixture.store(), fixture.owner());
    let from = create_project(&fixture, "Drafts").await.id;
    let to = create_project(&fixture, "Final").await.id;
    let animation = create_animation(&fixture, from, "Cat").await.id;

    let moved = store.move_animation(&owner, animation, to, OffsetDateTime::now_utc());

    assert_eq!(moved.await.unwrap().project, to);
    let read = store.get_animation(&owner, animation).await.unwrap();
    assert_eq!(read.project, to);
    assert_eq!(animation_ids(&fixture, Some(from)).await, []);
    assert_eq!(animation_ids(&fixture, Some(to)).await, [animation]);
    let from = store.get_project(&owner, from).await.unwrap();
    let to = store.get_project(&owner, to).await.unwrap();
    assert_eq!((from.animation_count, to.animation_count), (0, 1));
}

/// A deleted animation is gone from reads and lists; deleting it again is not found.
pub async fn animation_is_deleted(fixture: StoreFixture) {
    let (store, owner) = (fixture.store(), fixture.owner());
    let project = create_project(&fixture, "Pets").await;
    let kept = create_animation(&fixture, project.id, "Dog").await;
    let deleted = create_animation(&fixture, project.id, "Cat").await;

    store.delete_animation(&owner, deleted.id).await.unwrap();

    let not_found = StoreError::AnimationNotFound;
    assert_eq!(
        store.get_animation(&owner, deleted.id).await.unwrap_err(),
        not_found
    );
    assert_eq!(
        store.read_document(&owner, deleted.id).await.unwrap_err(),
        not_found
    );
    assert_eq!(
        store
            .delete_animation(&owner, deleted.id)
            .await
            .unwrap_err(),
        not_found
    );
    assert_eq!(animation_ids(&fixture, Some(project.id)).await, [kept.id]);
}

/// `delete_everything` leaves the owner without project, animation or usage.
pub async fn delete_everything_empties_the_owner(fixture: StoreFixture) {
    let (store, owner) = (fixture.store(), fixture.owner());
    for name in ["Pets", "Plants"] {
        let project = create_project(&fixture, name).await;
        create_animation(&fixture, project.id, "Cat").await;
    }

    store.delete_everything(&owner).await.unwrap();

    assert_eq!(project_ids(&fixture).await, []);
    assert_eq!(animation_ids(&fixture, None).await, []);
    assert_eq!(store.usage(&owner).await, Ok(0));
}
