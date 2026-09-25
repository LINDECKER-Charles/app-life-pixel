//! The cases of the library store's contract, one function each, run by
//! [`library_store_contract!`](crate::testing::library_store_contract).

mod animations;
mod projects;
mod quota;
mod removal;
mod writes;

pub use animations::{
    animation_is_created_from_a_document_and_read_back_byte_for_byte,
    animations_are_listed_by_project, animations_are_searched_case_insensitively,
    animations_page_in_a_stable_order_without_gap_or_duplicate,
};
pub use projects::{
    missing_project_is_not_found, project_is_created_and_read_back, project_is_deleted,
    project_is_invisible_to_another_owner, project_is_renamed, projects_are_listed,
};
pub use quota::{
    create_beyond_the_quota_is_refused_with_exact_numbers,
    deleting_a_project_frees_its_animations_usage, shrinking_write_above_the_quota_is_accepted,
    usage_follows_create_write_and_delete, write_beyond_the_quota_is_refused_with_exact_numbers,
};
pub use removal::{animation_is_deleted, animation_is_moved, delete_everything_empties_the_owner};
pub use writes::{
    concurrent_writes_on_one_version_let_exactly_one_win, write_with_a_wrong_version_is_a_conflict,
    write_with_the_right_version_gives_a_new_version,
};

use bytes::Bytes;
use time::OffsetDateTime;

use super::{StoreFixture, new_animation, new_project, sample_document};
use crate::ids::{AnimationId, ProjectId};
use crate::paging::PageRequest;
use crate::ports::library_store::{AnimationFilter, AnimationRecord, DocumentWrite, ProjectRecord};

/// A new project of the fixture's owner.
async fn create_project(fixture: &StoreFixture, name: &str) -> ProjectRecord {
    let project = new_project(name);
    let store = fixture.store();
    store
        .create_project(&fixture.owner(), project.clone())
        .await
        .unwrap();
    project
}

/// A new animation titled `title` in `project`, without quota.
async fn create_animation(
    fixture: &StoreFixture,
    project: ProjectId,
    title: &str,
) -> AnimationRecord {
    let new = new_animation(project, title);
    let store = fixture.store();
    store
        .create_animation(&fixture.owner(), new, None)
        .await
        .unwrap()
}

/// A write of the sample document titled `title` over `animation`'s `expected_version`, and its
/// bytes.
fn document_write(id: AnimationId, expected_version: u64, title: &str) -> (DocumentWrite, Bytes) {
    let (meta, document) = sample_document(title);
    let write = DocumentWrite {
        id,
        expected_version,
        meta,
        document: document.clone(),
        at: OffsetDateTime::now_utc(),
    };
    (write, document)
}

/// The ids of every animation of `project`, in list order.
async fn animation_ids(fixture: &StoreFixture, project: Option<ProjectId>) -> Vec<AnimationId> {
    let filter = AnimationFilter {
        project,
        query: None,
    };
    let page = fixture
        .store()
        .list_animations(&fixture.owner(), filter, PageRequest::default())
        .await
        .unwrap();
    page.items.iter().map(|animation| animation.id).collect()
}

/// The ids of the owner's projects, in list order.
async fn project_ids(fixture: &StoreFixture) -> Vec<ProjectId> {
    let store = fixture.store();
    let page = store
        .list_projects(&fixture.owner(), PageRequest::default())
        .await;
    page.unwrap()
        .items
        .iter()
        .map(|project| project.id)
        .collect()
}
