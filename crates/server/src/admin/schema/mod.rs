//! The internal admin API's bodies: what the routes answer and read, in camelCase. One line per
//! module.

mod audit;
mod metrics;
mod support;
mod users;

pub use audit::{AuditEntryBody, AuditQuery};
pub use metrics::{MetricsQuery, ProductMetricsBody};
pub use support::{
    AdminSupportContext, AdminSupportMessage, AdminSupportRequest, AdminSupportThread, MessagePost,
    QueueQuery, RequestPatch, Screenshot,
};
pub use users::{
    AdminSession, AdminUser, AdminUserDetail, AdminUserStatus, EventCountBody, ReasonBody,
    UserQuery,
};

use life_pixel_service::paging::{Cursor, PageRequest};
use time::OffsetDateTime;
use time::format_description::well_known::Rfc3339;

use crate::http::problem::Problem;

/// `at` in RFC 3339.
fn timestamp(at: OffsetDateTime) -> String {
    at.format(&Rfc3339).unwrap_or_default()
}

/// The page of `cursor` and `limit`.
///
/// # Errors
///
/// `request.malformed` for a cursor that does not decode.
fn page(cursor: Option<&str>, limit: Option<u16>) -> Result<PageRequest, Problem> {
    let cursor = cursor.map(str::parse::<Cursor>).transpose()?;
    Ok(PageRequest::new(cursor, limit))
}

/// `text` when it holds something besides whitespace.
fn non_blank(text: Option<&String>) -> Option<String> {
    text.map(|text| text.trim())
        .filter(|text| !text.is_empty())
        .map(str::to_owned)
}
