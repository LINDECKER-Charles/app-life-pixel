//! `GET` and `PATCH /account`: the account, its usage against the quota, and its language.

use axum::Json;
use axum::extract::State;

use super::schema::AccountChange;
use crate::accounts::CurrentSession;
use crate::http::problem::{Problem, ProblemDocument};
use crate::routes::auth::schema::Account;
use crate::routes::library::responses::{
    Forbidden, Malformed, RateLimited, Suspended, Unauthenticated,
};
use crate::state::AppState;

/// The signed-in account, with its storage usage and quota.
#[utoipa::path(
    get,
    path = "/account",
    tag = "account",
    operation_id = "getAccount",
    security(("session" = [])),
    responses(
        (status = OK, description = "The account", body = Account),
        Unauthenticated, Suspended, RateLimited
    )
)]
pub(super) async fn get_account(
    State(state): State<AppState>,
    current: CurrentSession,
) -> Result<Json<Account>, Problem> {
    let account = state.accounts.account(current.session.account.id).await?;
    Ok(Json(Account::from(&account)))
}

/// Sets the language of the signed-in account's emails and interface.
#[utoipa::path(
    patch,
    path = "/account",
    tag = "account",
    operation_id = "updateAccount",
    security(("session" = [], "csrf" = [])),
    request_body = AccountChange,
    responses(
        (status = OK, description = "The account, changed", body = Account),
        Malformed, Unauthenticated, Forbidden,
        (
            status = UNPROCESSABLE_ENTITY,
            description = "`account.language` (`available`)",
            body = ProblemDocument,
            content_type = "application/problem+json"
        ),
        RateLimited
    )
)]
pub(super) async fn update_account(
    State(state): State<AppState>,
    current: CurrentSession,
    Json(change): Json<AccountChange>,
) -> Result<Json<Account>, Problem> {
    let id = current.session.account.id;
    state.accounts.change_language(id, &change.language).await?;
    let account = state.accounts.account(id).await?;
    Ok(Json(Account::from(&account)))
}
