//! `/animations` and `/animations/{id}`: the account's animations — listed and searched, read,
//! retitled or moved, and deleted.

use axum::Json;
use axum::extract::{Query, State};
use axum::http::{HeaderMap, StatusCode};
use life_pixel_service::AnimationId;
use life_pixel_service::ports::library_store::AnimationFilter;
use uuid::Uuid;

use super::projects::project_id;
use super::responses::{
    AnimationNotFound, AnimationOrProjectNotFound, Forbidden, InvalidName, Malformed,
    QuotaExceeded, RateLimited, Suspended, Unauthenticated, VersionConflict, VersionRequired,
};
use super::schema::{Animation, AnimationChange, AnimationQuery, Page};
use super::{AccountOwner, Item, refused, versions};
use crate::http::problem::Problem;
use crate::state::AppState;

/// A page of the account's animations, from the most recently updated: those of a project,
/// those whose title holds a text, or all.
#[utoipa::path(
    get,
    path = "/animations",
    tag = "library",
    operation_id = "listAnimations",
    security(("session" = [])),
    params(AnimationQuery),
    responses(
        (status = OK, description = "A page of the animations", body = Page<Animation>),
        Malformed, Unauthenticated, Suspended, RateLimited
    )
)]
pub(super) async fn list_animations(
    State(state): State<AppState>,
    AccountOwner(owner): AccountOwner,
    Query(query): Query<AnimationQuery>,
) -> Result<Json<Page<Animation>>, Problem> {
    let page = query.page()?;
    let filter = AnimationFilter {
        project: query.project.map(project_id),
        query: query.q,
    };
    let listed = state.library.list_animations(&owner, filter, page);
    let page = listed.await.map_err(refused)?;
    Ok(Json(Page::of(page, Animation::from)))
}

/// The animation `id`, without its document.
#[utoipa::path(
    get,
    path = "/animations/{id}",
    tag = "library",
    operation_id = "getAnimation",
    security(("session" = [])),
    params(("id" = Uuid, Path, description = "The animation's id")),
    responses(
        (status = OK, description = "The animation", body = Animation),
        Malformed, Unauthenticated, Suspended, AnimationNotFound, RateLimited
    )
)]
pub(super) async fn get_animation(item: Item) -> Result<Json<Animation>, Problem> {
    let found = item
        .library
        .get_animation(&item.owner, animation_id(item.id));
    Ok(Json(Animation::from(found.await.map_err(refused)?)))
}

/// Retitles the animation `id` — a save of its document, under `If-Match` —, moves it to
/// another project, or both.
#[utoipa::path(
    patch,
    path = "/animations/{id}",
    tag = "library",
    operation_id = "updateAnimation",
    security(("session" = [], "csrf" = [])),
    params(
        ("id" = Uuid, Path, description = "The animation's id"),
        ("If-Match" = Option<String>, Header, description = "`\"<version>\"`, with a title"),
    ),
    request_body = AnimationChange,
    responses(
        (status = OK, description = "The animation, changed", body = Animation),
        Malformed, Unauthenticated, Forbidden, AnimationOrProjectNotFound, QuotaExceeded,
        VersionConflict, InvalidName, VersionRequired, RateLimited
    )
)]
pub(super) async fn update_animation(
    item: Item,
    headers: HeaderMap,
    Json(change): Json<AnimationChange>,
) -> Result<Json<Animation>, Problem> {
    let (library, owner, id) = (&item.library, &item.owner, animation_id(item.id));
    let mut changed = None;
    if let Some(title) = change.title {
        let version = versions::required(&headers)?;
        let renamed = library.rename_animation(owner, id, version, &title);
        changed = Some(renamed.await.map_err(refused)?);
    }
    if let Some(project) = change.project_id {
        let moved = library.move_animation(owner, id, project_id(project));
        changed = Some(moved.await.map_err(refused)?);
    }
    let animation = match changed {
        Some(animation) => animation,
        None => library.get_animation(owner, id).await.map_err(refused)?,
    };
    Ok(Json(Animation::from(animation)))
}

/// Deletes the animation `id`, whatever the quota.
#[utoipa::path(
    delete,
    path = "/animations/{id}",
    tag = "library",
    operation_id = "deleteAnimation",
    security(("session" = [], "csrf" = [])),
    params(("id" = Uuid, Path, description = "The animation's id")),
    responses(
        (status = NO_CONTENT, description = "The animation is deleted"),
        Malformed, Unauthenticated, Forbidden, AnimationNotFound, RateLimited
    )
)]
pub(super) async fn delete_animation(item: Item) -> Result<StatusCode, Problem> {
    let deleted = item
        .library
        .delete_animation(&item.owner, animation_id(item.id));
    deleted.await.map_err(refused)?;
    Ok(StatusCode::NO_CONTENT)
}

/// The animation id of a path.
pub(super) fn animation_id(id: Uuid) -> AnimationId {
    AnimationId::from_uuid(id)
}
