//! `/api/v1/projects` and `/api/v1/animations` (H6): the signed-in account's library — lists by
//! cursor, search, duplicates, deletions, and documents saved under `If-Match` and the storage
//! quota. Every route needs a session, and sits under the `api` rate limit and the CSRF check.

mod animations;
mod body;
mod copies;
mod documents;
pub mod metrics;
mod projects;
pub mod responses;
pub mod schema;
mod versions;

use axum::extract::{FromRequestParts, Path};
use axum::http::request::Parts;
use life_pixel_service::Owner;
use life_pixel_service::library::{Library, LibraryError};
use utoipa_axum::router::OpenApiRouter;
use utoipa_axum::routes;
use uuid::Uuid;

pub use body::DOCUMENT_MEDIA_TYPE;

use crate::accounts::CurrentSession;
use crate::http::problem::{Problem, codes};
use crate::state::AppState;

/// The routes under the `api` rate limit and the CSRF check.
pub fn rate_limited() -> OpenApiRouter<AppState> {
    project_routes().merge(animation_routes())
}

/// The routes under `/projects`: one line per path.
fn project_routes() -> OpenApiRouter<AppState> {
    OpenApiRouter::new()
        .routes(routes!(projects::list_projects, projects::create_project))
        .routes(routes!(
            projects::get_project,
            projects::rename_project,
            projects::delete_project
        ))
        .routes(routes!(copies::duplicate_project))
        .routes(routes!(documents::create_animation))
}

/// The routes under `/animations`: one line per path.
fn animation_routes() -> OpenApiRouter<AppState> {
    OpenApiRouter::new()
        .routes(routes!(animations::list_animations))
        .routes(routes!(
            animations::get_animation,
            animations::update_animation,
            animations::delete_animation
        ))
        .routes(routes!(documents::get_document, documents::save_document))
        .routes(routes!(copies::duplicate_animation))
}

/// The owner a live session acts for, as the routes extract it: without one, they answer
/// `auth.unauthenticated`, or `auth.account_suspended`.
struct AccountOwner(Owner);

impl<S: Send + Sync> FromRequestParts<S> for AccountOwner {
    type Rejection = Problem;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Problem> {
        let current = CurrentSession::from_request_parts(parts, state).await?;
        Ok(Self(Owner::Account(current.session.account.id)))
    }
}

/// What a route on one project or animation acts on: the library, the owner a live session
/// acts for, and the `{id}` of its path. It answers the problems of [`AccountOwner`] first,
/// then `request.malformed` for an id that does not parse.
struct Item {
    library: Library,
    owner: Owner,
    id: Uuid,
}

impl FromRequestParts<AppState> for Item {
    type Rejection = Problem;

    async fn from_request_parts(parts: &mut Parts, state: &AppState) -> Result<Self, Problem> {
        let AccountOwner(owner) = AccountOwner::from_request_parts(parts, state).await?;
        let path = Path::<Uuid>::from_request_parts(parts, state).await;
        let Path(id) = path.map_err(|_| Problem::new(codes::REQUEST_MALFORMED))?;
        Ok(Self {
            library: state.library.clone(),
            owner,
            id,
        })
    }
}

/// The problem of `error`, counting a refusal of the storage quota.
fn refused(error: LibraryError) -> Problem {
    if matches!(error, LibraryError::QuotaExceeded { .. }) {
        metrics::count_storage_rejection();
    }
    Problem::from(error)
}
