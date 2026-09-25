//! The server's own codes, and the one table that gives every code its HTTP status: the
//! statuses of docs/v1/service.md's codes and the server's own. One line per code, sorted.

use axum::http::StatusCode;

/// The codes the server itself answers with, as constants.
pub mod codes {
    /// The client is older than `LP_MIN_CLIENT_VERSIONS` allows; params `minimum`.
    pub const CLIENT_UPDATE_REQUIRED: &str = "client.update_required";
    /// A write without the `If-Match` of the version it replaces.
    pub const DOCUMENT_VERSION_REQUIRED: &str = "document.version_required";
    /// An unexpected failure, logged with the request id; nothing else is returned.
    pub const INTERNAL_ERROR: &str = "internal.error";
    /// Too many requests; params `retryAfterSeconds`, and the `Retry-After` header.
    pub const RATE_LIMIT_EXCEEDED: &str = "rate_limit.exceeded";
    /// A request that does not parse; shared with `service`.
    pub const REQUEST_MALFORMED: &str = "request.malformed";
    /// A method the route does not accept.
    pub const REQUEST_METHOD_NOT_ALLOWED: &str = "request.method_not_allowed";
    /// A route that does not exist.
    pub const REQUEST_NOT_FOUND: &str = "request.not_found";
    /// A body above the route's limit; params `maxBytes`.
    pub const REQUEST_TOO_LARGE: &str = "request.too_large";
    /// A body in a media type the route does not read.
    pub const REQUEST_UNSUPPORTED_MEDIA_TYPE: &str = "request.unsupported_media_type";
    /// A dependency that does not answer; shared with `service`.
    pub const SERVICE_UNAVAILABLE: &str = "service.unavailable";
}

/// Every code of this crate, each with its key `errors.<code>` in every catalogue.
/// `request.malformed` and `service.unavailable` belong to `service::error::CODES`.
pub const CODES: &[&str] = &[
    codes::CLIENT_UPDATE_REQUIRED,
    codes::DOCUMENT_VERSION_REQUIRED,
    codes::INTERNAL_ERROR,
    codes::RATE_LIMIT_EXCEEDED,
    codes::REQUEST_METHOD_NOT_ALLOWED,
    codes::REQUEST_NOT_FOUND,
    codes::REQUEST_TOO_LARGE,
    codes::REQUEST_UNSUPPORTED_MEDIA_TYPE,
];

/// Code to status: one line per code, sorted. A task that creates a code adds its line.
const STATUSES: &[(&str, StatusCode)] = &[
    ("client.update_required", StatusCode::UPGRADE_REQUIRED),
    ("document.canvas_size", StatusCode::UNPROCESSABLE_ENTITY),
    ("document.cel", StatusCode::UNPROCESSABLE_ENTITY),
    ("document.frame_count", StatusCode::UNPROCESSABLE_ENTITY),
    ("document.frame_duration", StatusCode::UNPROCESSABLE_ENTITY),
    ("document.layer_count", StatusCode::UNPROCESSABLE_ENTITY),
    ("document.malformed", StatusCode::UNPROCESSABLE_ENTITY),
    ("document.name", StatusCode::UNPROCESSABLE_ENTITY),
    ("document.palette", StatusCode::UNPROCESSABLE_ENTITY),
    ("document.pixel_budget", StatusCode::UNPROCESSABLE_ENTITY),
    ("document.reference", StatusCode::UNPROCESSABLE_ENTITY),
    ("document.tag", StatusCode::UNPROCESSABLE_ENTITY),
    ("document.tag_count", StatusCode::UNPROCESSABLE_ENTITY),
    ("document.too_large", StatusCode::PAYLOAD_TOO_LARGE),
    (
        "document.unsupported_version",
        StatusCode::UNPROCESSABLE_ENTITY,
    ),
    ("document.version_conflict", StatusCode::PRECONDITION_FAILED),
    (
        "document.version_required",
        StatusCode::PRECONDITION_REQUIRED,
    ),
    ("draw.too_many_operations", StatusCode::UNPROCESSABLE_ENTITY),
    ("edit.frame_not_found", StatusCode::UNPROCESSABLE_ENTITY),
    ("edit.last_frame", StatusCode::UNPROCESSABLE_ENTITY),
    ("edit.last_layer", StatusCode::UNPROCESSABLE_ENTITY),
    ("edit.layer_not_found", StatusCode::UNPROCESSABLE_ENTITY),
    ("edit.out_of_canvas", StatusCode::UNPROCESSABLE_ENTITY),
    ("edit.palette_full", StatusCode::UNPROCESSABLE_ENTITY),
    ("edit.palette_in_use", StatusCode::UNPROCESSABLE_ENTITY),
    (
        "edit.position_out_of_range",
        StatusCode::UNPROCESSABLE_ENTITY,
    ),
    ("edit.stroke_too_long", StatusCode::UNPROCESSABLE_ENTITY),
    ("edit.tag_not_found", StatusCode::UNPROCESSABLE_ENTITY),
    ("export.scale", StatusCode::UNPROCESSABLE_ENTITY),
    ("export.tag_not_found", StatusCode::UNPROCESSABLE_ENTITY),
    ("export.too_large", StatusCode::UNPROCESSABLE_ENTITY),
    ("grid.character", StatusCode::UNPROCESSABLE_ENTITY),
    ("grid.index", StatusCode::UNPROCESSABLE_ENTITY),
    ("grid.size", StatusCode::UNPROCESSABLE_ENTITY),
    ("import.image_malformed", StatusCode::UNPROCESSABLE_ENTITY),
    ("import.image_too_large", StatusCode::UNPROCESSABLE_ENTITY),
    ("import.sheet_grid", StatusCode::UNPROCESSABLE_ENTITY),
    ("internal.error", StatusCode::INTERNAL_SERVER_ERROR),
    ("library.animation_not_found", StatusCode::NOT_FOUND),
    ("library.project_not_found", StatusCode::NOT_FOUND),
    ("preview.too_large", StatusCode::UNPROCESSABLE_ENTITY),
    ("quota.storage_exceeded", StatusCode::CONFLICT),
    ("rate_limit.exceeded", StatusCode::TOO_MANY_REQUESTS),
    ("request.malformed", StatusCode::BAD_REQUEST),
    ("request.method_not_allowed", StatusCode::METHOD_NOT_ALLOWED),
    ("request.not_found", StatusCode::NOT_FOUND),
    ("request.too_large", StatusCode::PAYLOAD_TOO_LARGE),
    (
        "request.unsupported_media_type",
        StatusCode::UNSUPPORTED_MEDIA_TYPE,
    ),
    ("service.unavailable", StatusCode::SERVICE_UNAVAILABLE),
];

/// The HTTP status of `code`, or `None` for a code the table does not know.
#[must_use]
pub fn status_of(code: &str) -> Option<StatusCode> {
    STATUSES
        .iter()
        .find(|(known, _)| *known == code)
        .map(|(_, status)| *status)
}
