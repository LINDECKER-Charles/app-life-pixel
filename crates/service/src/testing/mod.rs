//! The contract suites every adapter of a port runs from its own crate's tests: feature
//! `testing`.
//!
//! [`library_store_contract!`] expands into one `#[tokio::test]` per case of the library store's
//! contract, each on a fresh [`StoreFixture`] made by the factory it is given:
//!
//! ```ignore
//! use life_pixel_service::testing::{StoreFixture, library_store_contract};
//!
//! async fn store() -> StoreFixture {
//!     StoreFixture::new(MyStore::connect().await)
//! }
//!
//! library_store_contract!(store);
//! ```
//!
//! The calling crate needs `life-pixel-service` with the `testing` feature and `tokio` with the
//! `macros` and `rt` features among its dev-dependencies. The cases make their own ids and act
//! for the fixture's owners only, so several cases may share one database.

// A contract case fails by panicking, as any test does; each assertion counts as a branch in
// cognitive complexity, though a case is a straight list of them.
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::cognitive_complexity)]

pub mod cases;
mod documents;
mod fixture;

pub use documents::{new_animation, new_project, sample_document};
pub use fixture::StoreFixture;

/// Expands into the library store's contract suite, one `#[tokio::test]` per case, in a module
/// `library_store_contract`. `$factory` is an async function, or a closure returning a future,
/// that makes a [`StoreFixture`](crate::testing::StoreFixture) — or any `LibraryStore`.
#[doc(hidden)]
#[macro_export]
macro_rules! __library_store_contract {
    ($factory:expr $(,)?) => {
        #[allow(unused_imports)]
        mod library_store_contract {
            use super::*;

            $crate::__library_store_contract!(@cases $factory;
                project_is_created_and_read_back,
                projects_are_listed,
                project_is_renamed,
                project_is_deleted,
                missing_project_is_not_found,
                project_is_invisible_to_another_owner,
                animation_is_created_from_a_document_and_read_back_byte_for_byte,
                animations_are_listed_by_project,
                animations_are_searched_case_insensitively,
                animations_page_in_a_stable_order_without_gap_or_duplicate,
                write_with_the_right_version_gives_a_new_version,
                write_with_a_wrong_version_is_a_conflict,
                concurrent_writes_on_one_version_let_exactly_one_win,
                create_beyond_the_quota_is_refused_with_exact_numbers,
                write_beyond_the_quota_is_refused_with_exact_numbers,
                shrinking_write_above_the_quota_is_accepted,
                usage_follows_create_write_and_delete,
                deleting_a_project_frees_its_animations_usage,
                animation_is_moved,
                animation_is_deleted,
                delete_everything_empties_the_owner,
            );
        }
    };
    (@cases $factory:expr; $($case:ident),+ $(,)?) => {
        $(
            #[::tokio::test]
            async fn $case() {
                let fixture = $crate::testing::StoreFixture::from(($factory)().await);
                $crate::testing::cases::$case(fixture).await;
            }
        )+
    };
}

#[doc(inline)]
pub use crate::__library_store_contract as library_store_contract;
