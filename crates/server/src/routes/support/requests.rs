//! `GET /support-requests` and `GET /support-requests/{id}`: the account's requests, and one
//! with its messages.

use axum::Json;
use axum::extract::{Query, State};

use super::responses::RequestNotFound;
use super::schema::{SupportQuery, SupportRequestSummary, SupportRequestThread};
use super::{OwnRequest, Requester};
use crate::http::problem::Problem;
use crate::routes::library::responses::{Malformed, RateLimited, Suspended, Unauthenticated};
use crate::routes::library::schema::Page;
use crate::state::AppState;

/// A page of the account's requests, from the most recently updated.
#[utoipa::path(
    get,
    path = "/support-requests",
    tag = "support",
    operation_id = "listSupportRequests",
    security(("session" = [])),
    params(SupportQuery),
    responses(
        (status = OK, description = "A page of the requests", body = Page<SupportRequestSummary>),
        Malformed, Unauthenticated, Suspended, RateLimited
    )
)]
pub(super) async fn list_support_requests(
    State(state): State<AppState>,
    Requester(account): Requester,
    Query(query): Query<SupportQuery>,
) -> Result<Json<Page<SupportRequestSummary>>, Problem> {
    let page = state.support.list(account, query.page()?).await?;
    Ok(Json(Page::of(page, SupportRequestSummary::from)))
}

/// The request `id`, with its messages: the team's internal notes never appear, and neither
/// does its screenshot.
#[utoipa::path(
    get,
    path = "/support-requests/{id}",
    tag = "support",
    operation_id = "getSupportRequest",
    security(("session" = [])),
    params(("id" = Uuid, Path, description = "The request's id")),
    responses(
        (status = OK, description = "The request and its messages", body = SupportRequestThread),
        Malformed, Unauthenticated, Suspended, RequestNotFound, RateLimited
    )
)]
pub(super) async fn get_support_request(
    State(state): State<AppState>,
    own: OwnRequest,
) -> Result<Json<SupportRequestThread>, Problem> {
    let thread = state.support.thread(own.account, own.id).await?;
    Ok(Json(thread.into()))
}
