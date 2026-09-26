//! `GET /tokens`, `POST /tokens` and `DELETE /tokens/{id}`.

use axum::Json;
use axum::extract::{Path, State};
use axum::http::StatusCode;
use metrics::counter;
use uuid::Uuid;

use super::responses::{InvalidToken, TokenLimit, TokenNotFound};
use super::schema::{AccessTokenSummary, CreatedAccessToken, McpServer, NewAccessToken};
use crate::accounts::CurrentSession;
use crate::accounts::metrics::AUTH_EVENTS_TOTAL;
use crate::http::problem::Problem;
use crate::mcp::MCP_PATH;
use crate::routes::library::responses::{
    Forbidden, Malformed, RateLimited, Suspended, Unauthenticated,
};
use crate::state::AppState;

/// The `event` of `auth_events_total` a created token counts under.
const TOKEN_CREATED: &str = "token_created";

/// The account's active tokens, the most recently created first; never their secret.
#[utoipa::path(
    get,
    path = "/tokens",
    tag = "tokens",
    operation_id = "listTokens",
    security(("session" = [])),
    responses(
        (status = OK, description = "The active tokens", body = Vec<AccessTokenSummary>),
        Unauthenticated, Suspended, RateLimited
    )
)]
pub(super) async fn list_tokens(
    State(state): State<AppState>,
    current: CurrentSession,
) -> Result<Json<Vec<AccessTokenSummary>>, Problem> {
    let tokens = state.tokens.list(current.session.account.id).await?;
    Ok(Json(tokens.into_iter().map(Into::into).collect()))
}

/// Creates a token: its secret is in this answer only, with the MCP server to register it on.
#[utoipa::path(
    post,
    path = "/tokens",
    tag = "tokens",
    operation_id = "createToken",
    security(("session" = [], "csrf" = [])),
    request_body = NewAccessToken,
    responses(
        (status = CREATED, description = "The token, its secret shown this once", body = CreatedAccessToken),
        Malformed, Unauthenticated, Forbidden, TokenLimit, InvalidToken, RateLimited
    )
)]
pub(super) async fn create_token(
    State(state): State<AppState>,
    current: CurrentSession,
    Json(request): Json<NewAccessToken>,
) -> Result<(StatusCode, Json<CreatedAccessToken>), Problem> {
    let account = current.session.account.id;
    let created = state.tokens.create(account, request.into()).await?;
    counter!(AUTH_EVENTS_TOTAL, "event" => TOKEN_CREATED).increment(1);
    let public_url = state.config.public_url.as_str().trim_end_matches('/');
    let mcp = McpServer {
        server_name: state.config.mcp_server_name.clone(),
        url: format!("{public_url}{MCP_PATH}"),
    };
    Ok((
        StatusCode::CREATED,
        Json(CreatedAccessToken::new(created, mcp)),
    ))
}

/// Revokes the token `id`: it stops working at once.
#[utoipa::path(
    delete,
    path = "/tokens/{id}",
    tag = "tokens",
    operation_id = "revokeToken",
    security(("session" = [], "csrf" = [])),
    params(("id" = Uuid, Path, description = "The token's id")),
    responses(
        (status = NO_CONTENT, description = "The token is revoked"),
        Malformed, Unauthenticated, Forbidden, TokenNotFound, RateLimited
    )
)]
pub(super) async fn revoke_token(
    State(state): State<AppState>,
    current: CurrentSession,
    Path(id): Path<Uuid>,
) -> Result<StatusCode, Problem> {
    state.tokens.revoke(current.session.account.id, id).await?;
    Ok(StatusCode::NO_CONTENT)
}
