//! `/projects` and `/projects/{id}`: the account's projects — listed, created, read, renamed and
//! deleted with their animations.

use axum::Json;
use axum::extract::{Query, State};
use axum::http::StatusCode;
use life_pixel_service::ProjectId;
use uuid::Uuid;

use super::responses::{
    Forbidden, InvalidName, Malformed, ProjectNotFound, RateLimited, Suspended, Unauthenticated,
};
use super::schema::{Page, Project, ProjectName, ProjectQuery};
use super::{AccountOwner, Item, refused};
use crate::http::problem::Problem;
use crate::state::AppState;

/// A page of the account's projects, from the most recently updated.
#[utoipa::path(
    get,
    path = "/projects",
    tag = "library",
    operation_id = "listProjects",
    security(("session" = [])),
    params(ProjectQuery),
    responses(
        (status = OK, description = "A page of the projects", body = Page<Project>),
        Malformed, Unauthenticated, Suspended, RateLimited
    )
)]
pub(super) async fn list_projects(
    State(state): State<AppState>,
    AccountOwner(owner): AccountOwner,
    Query(query): Query<ProjectQuery>,
) -> Result<Json<Page<Project>>, Problem> {
    let listed = state.library.list_projects(&owner, query.page()?);
    let page = listed.await.map_err(refused)?;
    Ok(Json(Page::of(page, Project::from)))
}

/// Creates an empty project.
#[utoipa::path(
    post,
    path = "/projects",
    tag = "library",
    operation_id = "createProject",
    security(("session" = [], "csrf" = [])),
    request_body = ProjectName,
    responses(
        (status = CREATED, description = "The project", body = Project),
        Malformed, Unauthenticated, Forbidden, InvalidName, RateLimited
    )
)]
pub(super) async fn create_project(
    State(state): State<AppState>,
    AccountOwner(owner): AccountOwner,
    Json(request): Json<ProjectName>,
) -> Result<(StatusCode, Json<Project>), Problem> {
    let created = state.library.create_project(&owner, &request.name);
    let project = created.await.map_err(refused)?;
    Ok((StatusCode::CREATED, Json(Project::from(project))))
}

/// The project `id`.
#[utoipa::path(
    get,
    path = "/projects/{id}",
    tag = "library",
    operation_id = "getProject",
    security(("session" = [])),
    params(("id" = Uuid, Path, description = "The project's id")),
    responses(
        (status = OK, description = "The project", body = Project),
        Malformed, Unauthenticated, Suspended, ProjectNotFound, RateLimited
    )
)]
pub(super) async fn get_project(item: Item) -> Result<Json<Project>, Problem> {
    let found = item.library.get_project(&item.owner, project_id(item.id));
    Ok(Json(Project::from(found.await.map_err(refused)?)))
}

/// Renames the project `id`.
#[utoipa::path(
    patch,
    path = "/projects/{id}",
    tag = "library",
    operation_id = "renameProject",
    security(("session" = [], "csrf" = [])),
    params(("id" = Uuid, Path, description = "The project's id")),
    request_body = ProjectName,
    responses(
        (status = OK, description = "The project, renamed", body = Project),
        Malformed, Unauthenticated, Forbidden, ProjectNotFound, InvalidName, RateLimited
    )
)]
pub(super) async fn rename_project(
    item: Item,
    Json(request): Json<ProjectName>,
) -> Result<Json<Project>, Problem> {
    let renamed = item
        .library
        .rename_project(&item.owner, project_id(item.id), &request.name);
    Ok(Json(Project::from(renamed.await.map_err(refused)?)))
}

/// Deletes the project `id` and its animations, whatever the quota.
#[utoipa::path(
    delete,
    path = "/projects/{id}",
    tag = "library",
    operation_id = "deleteProject",
    security(("session" = [], "csrf" = [])),
    params(("id" = Uuid, Path, description = "The project's id")),
    responses(
        (status = NO_CONTENT, description = "The project and its animations are deleted"),
        Malformed, Unauthenticated, Forbidden, ProjectNotFound, RateLimited
    )
)]
pub(super) async fn delete_project(item: Item) -> Result<StatusCode, Problem> {
    let deleted = item
        .library
        .delete_project(&item.owner, project_id(item.id));
    deleted.await.map_err(refused)?;
    Ok(StatusCode::NO_CONTENT)
}

/// The project id of a path.
pub(super) fn project_id(id: Uuid) -> ProjectId {
    ProjectId::from_uuid(id)
}
