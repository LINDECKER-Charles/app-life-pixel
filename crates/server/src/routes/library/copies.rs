//! Duplicates: a project with its animations, or one animation, copied under new ids and within
//! the storage quota.

use axum::Json;
use axum::http::StatusCode;

use super::animations::animation_id;
use super::projects::project_id;
use super::responses::{
    AnimationOrProjectNotFound, Forbidden, InvalidName, Malformed, ProjectNotFound, QuotaExceeded,
    RateLimited, Unauthenticated,
};
use super::schema::{Animation, AnimationCopy, Project, ProjectName};
use super::{Item, refused};
use crate::http::problem::Problem;

/// Copies the project `id` and its animations into a new project; the whole copy is checked
/// against the quota first.
#[utoipa::path(
    post,
    path = "/projects/{id}/duplicate",
    tag = "library",
    operation_id = "duplicateProject",
    security(("session" = [], "csrf" = [])),
    params(("id" = Uuid, Path, description = "The project's id")),
    request_body = ProjectName,
    responses(
        (status = CREATED, description = "The copy", body = Project),
        Malformed, Unauthenticated, Forbidden, ProjectNotFound, QuotaExceeded, InvalidName,
        RateLimited
    )
)]
pub(super) async fn duplicate_project(
    item: Item,
    Json(request): Json<ProjectName>,
) -> Result<(StatusCode, Json<Project>), Problem> {
    let copied = item
        .library
        .duplicate_project(&item.owner, project_id(item.id), &request.name);
    let copy = copied.await.map_err(refused)?;
    Ok((StatusCode::CREATED, Json(Project::from(copy))))
}

/// Copies the animation `id` under a new title, into another project or beside the original.
#[utoipa::path(
    post,
    path = "/animations/{id}/duplicate",
    tag = "library",
    operation_id = "duplicateAnimation",
    security(("session" = [], "csrf" = [])),
    params(("id" = Uuid, Path, description = "The animation's id")),
    request_body = AnimationCopy,
    responses(
        (status = CREATED, description = "The copy", body = Animation),
        Malformed, Unauthenticated, Forbidden, AnimationOrProjectNotFound, QuotaExceeded,
        InvalidName, RateLimited
    )
)]
pub(super) async fn duplicate_animation(
    item: Item,
    Json(request): Json<AnimationCopy>,
) -> Result<(StatusCode, Json<Animation>), Problem> {
    let project = request.project_id.map(project_id);
    let copied = item.library.duplicate_animation(
        &item.owner,
        animation_id(item.id),
        &request.title,
        project,
    );
    let copy = copied.await.map_err(refused)?;
    Ok((StatusCode::CREATED, Json(Animation::from(copy))))
}
