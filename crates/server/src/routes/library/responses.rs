//! The problems the library and account routes answer with, as the description names them: one
//! type per status and set of codes, listed in each route's `responses`.

use utoipa::IntoResponses;

use crate::http::problem::ProblemDocument;

/// `400`: a cursor, an id, a query or a body that does not parse.
#[derive(IntoResponses)]
#[response(
    status = BAD_REQUEST,
    description = "`request.malformed`: a cursor, an id, a query or a body that does not parse",
    content_type = "application/problem+json"
)]
pub struct Malformed(pub ProblemDocument);

/// `401`: no session.
#[derive(IntoResponses)]
#[response(
    status = UNAUTHORIZED,
    description = "`auth.unauthenticated`: no session, or one that ended",
    content_type = "application/problem+json"
)]
pub struct Unauthenticated(pub ProblemDocument);

/// `403` on a read: the account is suspended.
#[derive(IntoResponses)]
#[response(
    status = FORBIDDEN,
    description = "`auth.account_suspended`",
    content_type = "application/problem+json"
)]
pub struct Suspended(pub ProblemDocument);

/// `403` on a change: the account is suspended, or the CSRF check failed.
#[derive(IntoResponses)]
#[response(
    status = FORBIDDEN,
    description = "`auth.account_suspended`, `auth.csrf`",
    content_type = "application/problem+json"
)]
pub struct Forbidden(pub ProblemDocument);

/// `404`: the project is not the account's.
#[derive(IntoResponses)]
#[response(
    status = NOT_FOUND,
    description = "`library.project_not_found`",
    content_type = "application/problem+json"
)]
pub struct ProjectNotFound(pub ProblemDocument);

/// `404`: the animation is not the account's.
#[derive(IntoResponses)]
#[response(
    status = NOT_FOUND,
    description = "`library.animation_not_found`",
    content_type = "application/problem+json"
)]
pub struct AnimationNotFound(pub ProblemDocument);

/// `404`: the animation, or the project it goes to, is not the account's.
#[derive(IntoResponses)]
#[response(
    status = NOT_FOUND,
    description = "`library.animation_not_found`, `library.project_not_found`",
    content_type = "application/problem+json"
)]
pub struct AnimationOrProjectNotFound(pub ProblemDocument);

/// `409`: the change would take the account beyond its quota.
#[derive(IntoResponses)]
#[response(
    status = CONFLICT,
    description = "`quota.storage_exceeded` (`used`, `limit`, `requested`); deleting always works",
    content_type = "application/problem+json"
)]
pub struct QuotaExceeded(pub ProblemDocument);

/// `412`: the document changed since the version of `If-Match`.
#[derive(IntoResponses)]
#[response(
    status = PRECONDITION_FAILED,
    description = "`document.version_conflict` (`current`): the document changed since",
    content_type = "application/problem+json"
)]
pub struct VersionConflict(pub ProblemDocument);

/// `413`: the document is larger than `MAX_DOCUMENT_BYTES`.
#[derive(IntoResponses)]
#[response(
    status = PAYLOAD_TOO_LARGE,
    description = "`document.too_large` (`maxBytes`)",
    content_type = "application/problem+json"
)]
pub struct DocumentTooLarge(pub ProblemDocument);

/// `415`: the body is not a document.
#[derive(IntoResponses)]
#[response(
    status = UNSUPPORTED_MEDIA_TYPE,
    description = "`request.unsupported_media_type`: not `application/vnd.life-pixel.animation+json`",
    content_type = "application/problem+json"
)]
pub struct NotADocument(pub ProblemDocument);

/// `422`: the document breaks a rule of the model.
#[derive(IntoResponses)]
#[response(
    status = UNPROCESSABLE_ENTITY,
    description = "A `document.*` code of the model, with its parameters",
    content_type = "application/problem+json"
)]
pub struct InvalidDocument(pub ProblemDocument);

/// `422`: a name or a title breaks the model's rule.
#[derive(IntoResponses)]
#[response(
    status = UNPROCESSABLE_ENTITY,
    description = "`document.name` (`max`)",
    content_type = "application/problem+json"
)]
pub struct InvalidName(pub ProblemDocument);

/// `428`: a write without `If-Match`.
#[derive(IntoResponses)]
#[response(
    status = PRECONDITION_REQUIRED,
    description = "`document.version_required`: `If-Match` names the version replaced",
    content_type = "application/problem+json"
)]
pub struct VersionRequired(pub ProblemDocument);

/// `429`: the `api` rate limit.
#[derive(IntoResponses)]
#[response(
    status = TOO_MANY_REQUESTS,
    description = "`rate_limit.exceeded`: 600 a minute per account",
    content_type = "application/problem+json"
)]
pub struct RateLimited(pub ProblemDocument);
