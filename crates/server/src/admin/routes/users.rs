//! `GET /users` and `GET /users/{id}`: the search, and one user in detail.

use axum::Json;
use axum::extract::{Query, State};

use super::OnUser;
use super::responses::{AdminUnauthenticated, UserNotFound};
use crate::admin::schema::{AdminUser, AdminUserDetail, UserQuery};
use crate::http::problem::Problem;
use crate::routes::library::responses::Malformed;
use crate::routes::library::schema::Page;
use crate::state::AppState;

/// A page of the users, from the most recent sign-up: those whose address contains `q`,
/// whatever its case, or whose id is `q`, and of `status`.
#[utoipa::path(
    get,
    path = "/users",
    tag = "users",
    operation_id = "listUsers",
    params(UserQuery),
    responses(
        (status = OK, description = "A page of the users", body = Page<AdminUser>),
        Malformed, AdminUnauthenticated
    )
)]
pub(super) async fn list_users(
    State(state): State<AppState>,
    Query(query): Query<UserQuery>,
) -> Result<Json<Page<AdminUser>>, Problem> {
    let (filter, page) = query.filter_and_page()?;
    let page = state.admin.search_users(filter, page).await?;
    Ok(Json(Page::of(page, AdminUser::from)))
}

/// The user `id`: as listed, plus the language, the counts of the library, the product events
/// of the last 30 days by name, the sessions, the support requests and the access tokens.
#[utoipa::path(
    get,
    path = "/users/{id}",
    tag = "users",
    operation_id = "getUser",
    params(("id" = Uuid, Path, description = "The account's id")),
    responses(
        (status = OK, description = "The user", body = AdminUserDetail),
        Malformed, AdminUnauthenticated, UserNotFound
    )
)]
pub(super) async fn get_user(
    State(state): State<AppState>,
    user: OnUser,
) -> Result<Json<AdminUserDetail>, Problem> {
    let detail = state.admin.user(user.id).await?;
    let tokens = state.tokens.list_all(user.id).await?;
    Ok(Json(AdminUserDetail::new(detail, tokens)))
}
