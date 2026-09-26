//! The problems only the token routes answer with, as the description names them; the others
//! are the library's.

use utoipa::IntoResponses;

use crate::http::problem::ProblemDocument;

/// `404`: the account has no such active token.
#[derive(IntoResponses)]
#[response(
    status = NOT_FOUND,
    description = "`token.not_found`",
    content_type = "application/problem+json"
)]
pub struct TokenNotFound(pub ProblemDocument);

/// `409`: the account already has its most active tokens.
#[derive(IntoResponses)]
#[response(
    status = CONFLICT,
    description = "`token.limit`, with `max`",
    content_type = "application/problem+json"
)]
pub struct TokenLimit(pub ProblemDocument);

/// `422`: a value refused.
#[derive(IntoResponses)]
#[response(
    status = UNPROCESSABLE_ENTITY,
    description = "`token.name` with `max`; `token.expiry` with `allowed`",
    content_type = "application/problem+json"
)]
pub struct InvalidToken(pub ProblemDocument);
