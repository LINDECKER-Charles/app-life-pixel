//! The internal admin API's routes, one module per area: users and what the admin does to them,
//! the support queue and the team's answers, the metrics, the audit log and the environment.

mod audit;
mod environment;
mod export;
mod metrics;
pub mod responses;
mod support;
mod support_changes;
mod user_actions;
mod users;

use axum::extract::{FromRequestParts, Path};
use axum::http::request::Parts;
use life_pixel_service::AccountId;
use life_pixel_service::admin::AdminIdentity;
use life_pixel_service::support::SupportRequestId;
use utoipa_axum::router::OpenApiRouter;
use utoipa_axum::routes;
use uuid::Uuid;

use crate::admin::auth::ActingAdmin;
use crate::http::problem::{Problem, codes};
use crate::state::AppState;

/// Every route of the internal admin API: one line per area.
pub fn routes() -> OpenApiRouter<AppState> {
    OpenApiRouter::new()
        .merge(user_routes())
        .merge(support_routes())
        .merge(overview_routes())
}

/// The users and what the admin does to them: one line per path.
fn user_routes() -> OpenApiRouter<AppState> {
    OpenApiRouter::new()
        .routes(routes!(users::list_users))
        .routes(routes!(users::get_user, user_actions::delete_user))
        .routes(routes!(user_actions::suspend_user))
        .routes(routes!(user_actions::reactivate_user))
        .routes(routes!(export::export_user))
}

/// The support queue and the team's answers: one line per path.
fn support_routes() -> OpenApiRouter<AppState> {
    let request = routes!(
        support::get_support_request,
        support_changes::update_support_request
    );
    OpenApiRouter::new()
        .routes(routes!(support::list_support_requests))
        .routes(request)
        .routes(routes!(support::get_support_screenshot))
        .routes(routes!(support_changes::post_support_message))
}

/// The metrics, the audit log and the environment: one line per path.
fn overview_routes() -> OpenApiRouter<AppState> {
    OpenApiRouter::new()
        .routes(routes!(metrics::product_metrics))
        .routes(routes!(audit::list_audit_log))
        .routes(routes!(environment::get_environment))
}

/// The `{id}` of a path, as a UUID: `request.malformed` when it does not parse.
async fn path_id(parts: &mut Parts) -> Result<Uuid, Problem> {
    let path = Path::<Uuid>::from_request_parts(parts, &()).await;
    let Path(id) = path.map_err(|_| Problem::new(codes::REQUEST_MALFORMED))?;
    Ok(id)
}

/// A route on `/users/{id}`: the acting admin, then the account of the path.
struct OnUser {
    admin: AdminIdentity,
    id: AccountId,
}

impl<S: Send + Sync> FromRequestParts<S> for OnUser {
    type Rejection = Problem;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Problem> {
        let ActingAdmin(admin) = ActingAdmin::from_request_parts(parts, state).await?;
        let id = AccountId::from_uuid(path_id(parts).await?);
        Ok(Self { admin, id })
    }
}

/// A route on `/support-requests/{id}`: the acting admin, then the request of the path.
struct OnRequest {
    admin: AdminIdentity,
    id: SupportRequestId,
}

impl<S: Send + Sync> FromRequestParts<S> for OnRequest {
    type Rejection = Problem;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Problem> {
        let ActingAdmin(admin) = ActingAdmin::from_request_parts(parts, state).await?;
        let id = SupportRequestId::from_uuid(path_id(parts).await?);
        Ok(Self { admin, id })
    }
}
