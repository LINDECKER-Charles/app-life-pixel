//! `GET /healthz`: whether the server can answer, for Compose, the load balancer and
//! `life-pixel-server healthcheck`.

use std::time::Duration;

use axum::Json;
use axum::extract::State;
use serde::Serialize;
use utoipa::ToSchema;
use utoipa_axum::router::OpenApiRouter;
use utoipa_axum::routes;

use crate::http::problem::{Problem, ProblemDocument, codes};
use crate::state::AppState;

/// How long the database has to answer.
const READINESS_TIMEOUT: Duration = Duration::from_secs(1);

/// The server's health.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, ToSchema)]
pub struct Health {
    /// Always `ok`: a server that cannot answer sends a problem.
    pub status: HealthStatus,
}

/// The status of a healthy server.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub enum HealthStatus {
    /// The database answers.
    Ok,
}

/// The routes of this module.
pub fn router() -> OpenApiRouter<AppState> {
    OpenApiRouter::new().routes(routes!(healthz))
}

/// Whether the database answers within a second.
#[utoipa::path(
    get,
    path = "/healthz",
    tag = "health",
    operation_id = "healthz",
    responses(
        (status = OK, description = "The database answers", body = Health),
        (
            status = SERVICE_UNAVAILABLE,
            description = "The database does not answer: `service.unavailable`",
            body = ProblemDocument,
            content_type = "application/problem+json"
        ),
    )
)]
async fn healthz(State(state): State<AppState>) -> Result<Json<Health>, Problem> {
    let ready = tokio::time::timeout(READINESS_TIMEOUT, state.readiness.is_ready()).await;
    if ready != Ok(true) {
        return Err(Problem::new(codes::SERVICE_UNAVAILABLE));
    }
    Ok(Json(Health {
        status: HealthStatus::Ok,
    }))
}
