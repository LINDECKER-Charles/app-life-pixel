//! `GET /metrics/product`: H13's aggregates over a period, and the support queue's measures.

use axum::Json;
use axum::extract::{Query, State};

use super::responses::AdminUnauthenticated;
use crate::admin::schema::{MetricsQuery, ProductMetricsBody};
use crate::http::problem::Problem;
use crate::routes::library::responses::Malformed;
use crate::state::AppState;

/// The product metrics from `from` to `to`: sign-ups, activation, active accounts, exports, MCP
/// calls and storage, then the open requests by status and the medians of their age, of the time
/// to a first answer and of the time to resolution.
#[utoipa::path(
    get,
    path = "/metrics/product",
    tag = "metrics",
    operation_id = "getProductMetrics",
    params(MetricsQuery),
    responses(
        (status = OK, description = "The metrics", body = ProductMetricsBody),
        Malformed, AdminUnauthenticated
    )
)]
pub(super) async fn product_metrics(
    State(state): State<AppState>,
    Query(query): Query<MetricsQuery>,
) -> Result<Json<ProductMetricsBody>, Problem> {
    let metrics = state.admin.product_metrics(query.bounds()?).await?;
    Ok(Json(metrics.into()))
}
