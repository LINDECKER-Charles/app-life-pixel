//! The problems the internal admin API answers with, as its description names them; a
//! malformed request is the library's `Malformed`.

use utoipa::IntoResponses;

use crate::http::problem::ProblemDocument;

/// `401`: not the admin server, or no admin named.
#[derive(IntoResponses)]
#[response(
    status = UNAUTHORIZED,
    description = "`admin.unauthenticated`: no `Authorization: Bearer` with the admin API's \
                   secret, or no valid `X-Admin-Id` and `X-Admin-Email`",
    content_type = "application/problem+json"
)]
pub struct AdminUnauthenticated(pub ProblemDocument);

/// `404`: no such account.
#[derive(IntoResponses)]
#[response(
    status = NOT_FOUND,
    description = "`admin.user_not_found`",
    content_type = "application/problem+json"
)]
pub struct UserNotFound(pub ProblemDocument);

/// `404`: no such support request.
#[derive(IntoResponses)]
#[response(
    status = NOT_FOUND,
    description = "`support.request_not_found`",
    content_type = "application/problem+json"
)]
pub struct RequestNotFound(pub ProblemDocument);

/// `404`: no such support request, or it came without a screenshot.
#[derive(IntoResponses)]
#[response(
    status = NOT_FOUND,
    description = "`support.request_not_found`, `admin.screenshot_not_found`",
    content_type = "application/problem+json"
)]
pub struct ScreenshotNotFound(pub ProblemDocument);

/// `422`: the reason of an action on an account.
#[derive(IntoResponses)]
#[response(
    status = UNPROCESSABLE_ENTITY,
    description = "`admin.reason_length`, with `min` and `max`",
    content_type = "application/problem+json"
)]
pub struct InvalidReason(pub ProblemDocument);

/// `422`: a message's length.
#[derive(IntoResponses)]
#[response(
    status = UNPROCESSABLE_ENTITY,
    description = "`support.message_length`, with `min` and `max`",
    content_type = "application/problem+json"
)]
pub struct InvalidMessage(pub ProblemDocument);
