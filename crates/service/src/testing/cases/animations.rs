//! Animations: create from a document, read it back byte for byte, list by project, search
//! case-insensitively, page in a stable order.

use std::collections::BTreeSet;

use super::{animation_ids, create_animation, create_project};
use crate::ids::AnimationId;
use crate::paging::{Cursor, PageRequest};
use crate::ports::library_store::{AnimationFilter, AnimationRecord};
use crate::testing::{StoreFixture, new_animation};

/// How many animations the paging case creates.
const PAGED_ANIMATIONS: usize = 120;
/// The page size of the paging case.
const PAGE_LIMIT: u16 = 50;

/// A created animation answers with its record, and reads back byte for byte.
pub async fn animation_is_created_from_a_document_and_read_back_byte_for_byte(
    fixture: StoreFixture,
) {
    let (store, owner) = (fixture.store(), fixture.owner());
    let project = create_project(&fixture, "Pets").await;
    let new = new_animation(project.id, "Cat");
    let (id, meta, document) = (new.id, new.meta.clone(), new.document.clone());

    let created = store.create_animation(&owner, new, None).await.unwrap();

    assert_eq!((created.id, created.project), (id, project.id));
    assert_eq!(created.meta, meta);
    assert_eq!(
        created.document_bytes,
        u64::try_from(document.len()).unwrap()
    );
    let (record, bytes) = store.read_document(&owner, id).await.unwrap();
    assert_eq!(bytes, document);
    assert_eq!(record, created);
    assert_eq!(store.get_animation(&owner, id).await.unwrap(), created);
}

/// A project's list holds its animations only, and its record counts them.
pub async fn animations_are_listed_by_project(fixture: StoreFixture) {
    let pets = create_project(&fixture, "Pets").await;
    let plants = create_project(&fixture, "Plants").await;
    let cat = create_animation(&fixture, pets.id, "Cat").await.id;
    let dog = create_animation(&fixture, pets.id, "Dog").await.id;
    let fern = create_animation(&fixture, plants.id, "Fern").await.id;

    assert_eq!(
        sorted(animation_ids(&fixture, Some(pets.id)).await),
        sorted(vec![cat, dog])
    );
    assert_eq!(animation_ids(&fixture, Some(plants.id)).await, [fern]);
    assert_eq!(
        sorted(animation_ids(&fixture, None).await),
        sorted(vec![cat, dog, fern])
    );
    let pets = fixture.store().get_project(&fixture.owner(), pets.id).await;
    assert_eq!(pets.unwrap().animation_count, 2);
}

/// A query keeps the titles that hold it, whatever their case.
pub async fn animations_are_searched_case_insensitively(fixture: StoreFixture) {
    let project = create_project(&fixture, "Animals").await;
    let walking = create_animation(&fixture, project.id, "Walking Cat")
        .await
        .id;
    let napping = create_animation(&fixture, project.id, "cat nap").await.id;
    create_animation(&fixture, project.id, "Dog").await;

    assert_eq!(
        search(&fixture, "CAT").await,
        sorted(vec![walking, napping])
    );
    assert_eq!(search(&fixture, "wALk").await, [walking]);
    assert_eq!(search(&fixture, "zebra").await, []);
}

/// 120 animations paged by 50: 50, 50, then 20, from the most recently updated, each once, in
/// the same order every time.
pub async fn animations_page_in_a_stable_order_without_gap_or_duplicate(fixture: StoreFixture) {
    let project = create_project(&fixture, "Crowd").await;
    let mut created = Vec::new();
    for index in 0..PAGED_ANIMATIONS {
        let title = format!("Walker {index}");
        created.push(create_animation(&fixture, project.id, &title).await.id);
    }

    let first = all_pages(&fixture).await;
    let second = all_pages(&fixture).await;

    let sizes: Vec<usize> = first.iter().map(Vec::len).collect();
    assert_eq!(sizes, [50, 50, 20]);
    let listed: Vec<AnimationRecord> = first.into_iter().flatten().collect();
    let ids: Vec<AnimationId> = listed.iter().map(|animation| animation.id).collect();
    assert_eq!(sorted(ids.clone()), sorted(created));
    let cursors: Vec<Cursor> = listed.iter().map(cursor).collect();
    assert!(
        cursors.windows(2).all(|pair| pair[0] > pair[1]),
        "not in list order"
    );
    let again: Vec<AnimationId> = second.into_iter().flatten().map(|a| a.id).collect();
    assert_eq!(again, ids);
}

/// Every page of the owner's animations, by [`PAGE_LIMIT`].
async fn all_pages(fixture: &StoreFixture) -> Vec<Vec<AnimationRecord>> {
    let mut request = PageRequest::new(None, Some(PAGE_LIMIT));
    let mut pages = Vec::new();
    loop {
        let page = fixture
            .store()
            .list_animations(
                &fixture.owner(),
                AnimationFilter::default(),
                request.clone(),
            )
            .await
            .unwrap();
        pages.push(page.items);
        let Some(next) = page.next_cursor else {
            return pages;
        };
        request.cursor = Some(next);
    }
}

/// The ids of the animations whose title holds `query`, sorted.
async fn search(fixture: &StoreFixture, query: &str) -> Vec<AnimationId> {
    let filter = AnimationFilter {
        project: None,
        query: Some(query.to_owned()),
    };
    let page = fixture
        .store()
        .list_animations(&fixture.owner(), filter, PageRequest::default())
        .await
        .unwrap();
    sorted(page.items.iter().map(|animation| animation.id).collect())
}

fn sorted(ids: Vec<AnimationId>) -> Vec<AnimationId> {
    let unique: BTreeSet<AnimationId> = ids.iter().copied().collect();
    assert_eq!(unique.len(), ids.len(), "an animation is listed twice");
    unique.into_iter().collect()
}

fn cursor(animation: &AnimationRecord) -> Cursor {
    Cursor {
        updated_at: animation.updated_at,
        id: animation.id.uuid(),
    }
}
