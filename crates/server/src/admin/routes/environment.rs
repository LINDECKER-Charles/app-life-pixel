//! `GET /environment`: which environment the server runs in, and its version.

use axum::Json;
use axum::extract::State;
use serde::Serialize;
use utoipa::ToSchema;

use super::responses::AdminUnauthenticated;
use crate::state::AppState;

/// The server's environment and version.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct AdminEnvironment {
    /// `local`, `staging` or `production`.
    pub environment: String,
    /// The server's version.
    pub version: String,
}

/// The environment the server runs in, and its version: the console shows them in its banner.
#[utoipa::path(
    get,
    path = "/environment",
    tag = "environment",
    operation_id = "getEnvironment",
    responses(
        (status = OK, description = "The environment", body = AdminEnvironment),
        AdminUnauthenticated
    )
)]
pub(super) async fn get_environment(State(state): State<AppState>) -> Json<AdminEnvironment> {
    Json(AdminEnvironment {
        environment: state.config.environment.as_str().to_owned(),
        version: env!("CARGO_PKG_VERSION").to_owned(),
    })
}
