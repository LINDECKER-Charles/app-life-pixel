//! `GET /audit-log`: the entries, newest first.

use axum::Json;
use axum::extract::{Query, State};

use super::responses::AdminUnauthenticated;
use crate::admin::schema::{AuditEntryBody, AuditQuery};
use crate::http::problem::Problem;
use crate::routes::library::responses::Malformed;
use crate::routes::library::schema::Page;
use crate::state::AppState;

/// A page of the audit log, newest first: the entries of `adminId`, of `action`, about
/// `targetId`.
#[utoipa::path(
    get,
    path = "/audit-log",
    tag = "audit",
    operation_id = "listAuditLog",
    params(AuditQuery),
    responses(
        (status = OK, description = "A page of the log", body = Page<AuditEntryBody>),
        Malformed, AdminUnauthenticated
    )
)]
pub(super) async fn list_audit_log(
    State(state): State<AppState>,
    Query(query): Query<AuditQuery>,
) -> Result<Json<Page<AuditEntryBody>>, Problem> {
    let (filter, page) = query.filter_and_page()?;
    let page = state.admin.audit_log(filter, page).await?;
    Ok(Json(Page::of(page, AuditEntryBody::from)))
}
