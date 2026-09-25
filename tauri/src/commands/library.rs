//! The library commands: the HTTP API's library routes, in-process on the local library.

use bytes::Bytes;
use life_pixel_service::Owner;
use life_pixel_service::ports::AnimationFilter;
use tauri::State;

use super::arguments;
use super::shapes::{Animation, ListPage, OpenedDocument, Project, StorageUsage};
use crate::errors::CommandError;
use crate::state::DesktopState;

/// The one owner of a local library.
const LOCAL: Owner = Owner::Local;

/// A page of the projects, from the most recently updated.
#[tauri::command]
pub async fn library_list_projects(
    state: State<'_, DesktopState>,
    cursor: Option<String>,
    limit: Option<u32>,
) -> Result<ListPage<Project>, CommandError> {
    let page = arguments::page(cursor, limit)?;
    let projects = state.library().await?.list_projects(&LOCAL, page).await?;
    Ok(ListPage::of(projects))
}

/// Creates an empty project.
#[tauri::command]
pub async fn library_create_project(
    state: State<'_, DesktopState>,
    name: String,
) -> Result<Project, CommandError> {
    let project = state.library().await?.create_project(&LOCAL, &name).await?;
    Ok(project.into())
}

/// Renames a project.
#[tauri::command]
pub async fn library_rename_project(
    state: State<'_, DesktopState>,
    id: String,
    name: String,
) -> Result<Project, CommandError> {
    let id = arguments::id("id", &id)?;
    let library = state.library().await?;
    Ok(library.rename_project(&LOCAL, id, &name).await?.into())
}

/// Copies a project and its animations into a new project.
#[tauri::command]
pub async fn library_duplicate_project(
    state: State<'_, DesktopState>,
    id: String,
    name: String,
) -> Result<Project, CommandError> {
    let id = arguments::id("id", &id)?;
    let library = state.library().await?;
    Ok(library.duplicate_project(&LOCAL, id, &name).await?.into())
}

/// Deletes a project and its animations.
#[tauri::command]
pub async fn library_delete_project(
    state: State<'_, DesktopState>,
    id: String,
) -> Result<(), CommandError> {
    let id = arguments::id("id", &id)?;
    Ok(state.library().await?.delete_project(&LOCAL, id).await?)
}

/// A page of the animations — of a project, whose title holds `query`, or all —, from the most
/// recently updated.
#[tauri::command]
#[allow(clippy::too_many_arguments)] // The arguments of the HTTP API's query, flat as it has them.
pub async fn library_list_animations(
    state: State<'_, DesktopState>,
    project_id: Option<String>,
    query: Option<String>,
    cursor: Option<String>,
    limit: Option<u32>,
) -> Result<ListPage<Animation>, CommandError> {
    let project = project_id
        .map(|text| arguments::id("projectId", &text))
        .transpose()?;
    let filter = AnimationFilter { project, query };
    let page = arguments::page(cursor, limit)?;
    let library = state.library().await?;
    let animations = library.list_animations(&LOCAL, filter, page).await?;
    Ok(ListPage::of(animations))
}

/// Creates an animation in a project from a document: the first save of new work.
#[tauri::command]
pub async fn library_create_animation(
    state: State<'_, DesktopState>,
    project_id: String,
    document: String,
) -> Result<Animation, CommandError> {
    let project = arguments::id("projectId", &project_id)?;
    let document = Bytes::from(arguments::bytes("document", &document)?);
    let library = state.library().await?;
    Ok(library
        .import_animation(&LOCAL, project, document)
        .await?
        .into())
}

/// An animation and its document.
#[tauri::command]
pub async fn library_open_document(
    state: State<'_, DesktopState>,
    id: String,
) -> Result<OpenedDocument, CommandError> {
    let id = arguments::id("id", &id)?;
    let (summary, document) = state.library().await?.open_document(&LOCAL, id).await?;
    Ok(OpenedDocument {
        summary: summary.into(),
        document: arguments::base64(&document),
    })
}

/// Replaces a document when its version is still `version`.
#[tauri::command]
#[allow(clippy::too_many_arguments)] // The HTTP API's route, its `If-Match` and its body.
pub async fn library_save_document(
    state: State<'_, DesktopState>,
    id: String,
    version: u64,
    document: String,
) -> Result<Animation, CommandError> {
    let id = arguments::id("id", &id)?;
    let document = Bytes::from(arguments::bytes("document", &document)?);
    let library = state.library().await?;
    Ok(library
        .save_document(&LOCAL, id, version, document)
        .await?
        .into())
}

/// Retitles an animation when its version is still `version`.
#[tauri::command]
#[allow(clippy::too_many_arguments)] // The HTTP API's route, its `If-Match` and its body.
pub async fn library_rename_animation(
    state: State<'_, DesktopState>,
    id: String,
    version: u64,
    title: String,
) -> Result<Animation, CommandError> {
    let id = arguments::id("id", &id)?;
    let library = state.library().await?;
    let renamed = library
        .rename_animation(&LOCAL, id, version, &title)
        .await?;
    Ok(renamed.into())
}

/// Moves an animation to another project.
#[tauri::command]
pub async fn library_move_animation(
    state: State<'_, DesktopState>,
    id: String,
    project_id: String,
) -> Result<Animation, CommandError> {
    let id = arguments::id("id", &id)?;
    let project = arguments::id("projectId", &project_id)?;
    let library = state.library().await?;
    Ok(library.move_animation(&LOCAL, id, project).await?.into())
}

/// Copies an animation under a new title, into `projectId` or beside the original.
#[tauri::command]
#[allow(clippy::too_many_arguments)] // The HTTP API's route and its body.
pub async fn library_duplicate_animation(
    state: State<'_, DesktopState>,
    id: String,
    title: String,
    project_id: Option<String>,
) -> Result<Animation, CommandError> {
    let id = arguments::id("id", &id)?;
    let project = project_id
        .map(|text| arguments::id("projectId", &text))
        .transpose()?;
    let library = state.library().await?;
    let copy = library
        .duplicate_animation(&LOCAL, id, &title, project)
        .await?;
    Ok(copy.into())
}

/// Deletes an animation.
#[tauri::command]
pub async fn library_delete_animation(
    state: State<'_, DesktopState>,
    id: String,
) -> Result<(), CommandError> {
    let id = arguments::id("id", &id)?;
    Ok(state.library().await?.delete_animation(&LOCAL, id).await?)
}

/// The bytes the library's documents take; no quota.
#[tauri::command]
pub async fn library_usage(state: State<'_, DesktopState>) -> Result<StorageUsage, CommandError> {
    Ok(state.library().await?.usage(&LOCAL).await?.into())
}
