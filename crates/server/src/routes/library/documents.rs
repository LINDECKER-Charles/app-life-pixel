//! Documents: the first save of an animation into a project, then reads and saves under
//! `If-Match`. The `ETag` of a document is its animation's version.

use axum::Json;
use axum::body::Body;
use axum::http::header::CONTENT_TYPE;
use axum::http::{HeaderMap, HeaderValue, StatusCode};
use axum::response::{IntoResponse, Response};

use super::animations::animation_id;
use super::body::{DOCUMENT_MEDIA_TYPE, DocumentBody};
use super::projects::project_id;
use super::responses::{
    AnimationNotFound, DocumentTooLarge, Forbidden, InvalidDocument, Malformed, NotADocument,
    ProjectNotFound, QuotaExceeded, RateLimited, Suspended, Unauthenticated, VersionConflict,
    VersionRequired,
};
use super::schema::{Animation, AnimationDocument};
use super::{Item, refused, versions};
use crate::http::problem::Problem;

/// Creates an animation in the project `id` from its document: the app's first save.
#[utoipa::path(
    post,
    path = "/projects/{id}/animations",
    tag = "library",
    operation_id = "createAnimation",
    security(("session" = [], "csrf" = [])),
    params(("id" = Uuid, Path, description = "The project's id")),
    request_body(
        content = AnimationDocument,
        content_type = "application/vnd.life-pixel.animation+json"
    ),
    responses(
        (
            status = CREATED,
            description = "The animation; `ETag` is its version",
            body = Animation,
            headers(("ETag" = String, description = "`\"<version>\"`"))
        ),
        Malformed, Unauthenticated, Forbidden, ProjectNotFound, QuotaExceeded, DocumentTooLarge,
        NotADocument, InvalidDocument, RateLimited
    )
)]
pub(super) async fn create_animation(
    item: Item,
    DocumentBody(document): DocumentBody,
) -> Result<Response, Problem> {
    let imported = item
        .library
        .import_animation(&item.owner, project_id(item.id), document);
    let animation = imported.await.map_err(refused)?;
    let tag = versions::etag(animation.version);
    Ok((StatusCode::CREATED, tag, Json(Animation::from(animation))).into_response())
}

/// The document of the animation `id`.
#[utoipa::path(
    get,
    path = "/animations/{id}/document",
    tag = "library",
    operation_id = "getDocument",
    security(("session" = [])),
    params(("id" = Uuid, Path, description = "The animation's id")),
    responses(
        (
            status = OK,
            description = "The document; `ETag` is its version",
            body = AnimationDocument,
            content_type = "application/vnd.life-pixel.animation+json",
            headers(("ETag" = String, description = "`\"<version>\"`"))
        ),
        Malformed, Unauthenticated, Suspended, AnimationNotFound, RateLimited
    )
)]
pub(super) async fn get_document(item: Item) -> Result<Response, Problem> {
    let opened = item
        .library
        .open_document(&item.owner, animation_id(item.id));
    let (animation, document) = opened.await.map_err(refused)?;
    let media_type = [(CONTENT_TYPE, HeaderValue::from_static(DOCUMENT_MEDIA_TYPE))];
    let tag = versions::etag(animation.version);
    Ok((media_type, tag, Body::from(document)).into_response())
}

/// Replaces the document of the animation `id`, when `If-Match` names its current version.
#[utoipa::path(
    put,
    path = "/animations/{id}/document",
    tag = "library",
    operation_id = "saveDocument",
    security(("session" = [], "csrf" = [])),
    params(
        ("id" = Uuid, Path, description = "The animation's id"),
        ("If-Match" = String, Header, description = "`\"<version>\"`: the version replaced"),
    ),
    request_body(
        content = AnimationDocument,
        content_type = "application/vnd.life-pixel.animation+json"
    ),
    responses(
        (
            status = OK,
            description = "The animation, saved; `ETag` is its new version",
            body = Animation,
            headers(("ETag" = String, description = "`\"<version>\"`"))
        ),
        Malformed, Unauthenticated, Forbidden, AnimationNotFound, QuotaExceeded,
        VersionConflict, DocumentTooLarge, NotADocument, InvalidDocument, VersionRequired,
        RateLimited
    )
)]
pub(super) async fn save_document(
    item: Item,
    headers: HeaderMap,
    DocumentBody(document): DocumentBody,
) -> Result<Response, Problem> {
    let version = versions::required(&headers)?;
    let saved = item
        .library
        .save_document(&item.owner, animation_id(item.id), version, document);
    let animation = saved.await.map_err(refused)?;
    let tag = versions::etag(animation.version);
    Ok((tag, Json(Animation::from(animation))).into_response())
}
