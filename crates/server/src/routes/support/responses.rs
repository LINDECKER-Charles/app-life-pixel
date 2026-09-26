//! The problems only the support routes answer with, as the description names them; the others
//! are the library's.

use utoipa::IntoResponses;

use crate::http::problem::ProblemDocument;

/// `404`: the request is not the account's.
#[derive(IntoResponses)]
#[response(
    status = NOT_FOUND,
    description = "`support.request_not_found`",
    content_type = "application/problem+json"
)]
pub struct RequestNotFound(pub ProblemDocument);

/// `409`: a closed request takes no reply.
#[derive(IntoResponses)]
#[response(
    status = CONFLICT,
    description = "`support.request_closed`",
    content_type = "application/problem+json"
)]
pub struct RequestClosed(pub ProblemDocument);

/// `422` on a new request: a value refused.
#[derive(IntoResponses)]
#[response(
    status = UNPROCESSABLE_ENTITY,
    description = "`support.category`; `support.message_length` with `min` and `max`; \
                   `support.screenshot` with `maxSide` and `maxBytes`",
    content_type = "application/problem+json"
)]
pub struct InvalidRequest(pub ProblemDocument);

/// `422` on a reply: its length.
#[derive(IntoResponses)]
#[response(
    status = UNPROCESSABLE_ENTITY,
    description = "`support.message_length`, with `min` and `max`",
    content_type = "application/problem+json"
)]
pub struct InvalidReply(pub ProblemDocument);

/// `413`: a form above the route's limit.
#[derive(IntoResponses)]
#[response(
    status = PAYLOAD_TOO_LARGE,
    description = "`request.too_large`, with `maxBytes`: 6 MiB",
    content_type = "application/problem+json"
)]
pub struct FormTooLarge(pub ProblemDocument);

/// `415`: not a `multipart/form-data` body.
#[derive(IntoResponses)]
#[response(
    status = UNSUPPORTED_MEDIA_TYPE,
    description = "`request.unsupported_media_type`",
    content_type = "application/problem+json"
)]
pub struct NotAForm(pub ProblemDocument);

/// `429`: the `support_create` rate limit.
#[derive(IntoResponses)]
#[response(
    status = TOO_MANY_REQUESTS,
    description = "`rate_limit.exceeded`: 10 a day per account",
    content_type = "application/problem+json"
)]
pub struct CreationRateLimited(pub ProblemDocument);
