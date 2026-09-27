//! The layer in front of `/mcp`: a personal access token as `Authorization: Bearer lp_pat_…`,
//! then the `mcp` rate limit of that token.

use axum::extract::{Request, State};
use axum::http::header::{AUTHORIZATION, WWW_AUTHENTICATE};
use axum::http::{HeaderMap, HeaderValue};
use axum::middleware::Next;
use axum::response::{IntoResponse, Response};
use life_pixel_service::tokens::TokensError;

use crate::http::problem::Problem;
use crate::http::rate_limit::{Policy, RateKey};
use crate::state::AppState;

/// The scheme of `Authorization`, compared without case.
const BEARER: &str = "bearer";

/// Middleware of `/mcp`: puts the request's [`AccessToken`](life_pixel_service::tokens::AccessToken)
/// in its extensions, where the caller policy reads it.
///
/// Answers `401 token.invalid` with `WWW-Authenticate: Bearer` without a token, or with one
/// unknown, revoked or expired; `403 auth.account_suspended`; `429 rate_limit.exceeded` beyond
/// the `mcp` limit of the token; `503 service.unavailable`.
pub async fn require_token(
    State(state): State<AppState>,
    mut request: Request,
    next: Next,
) -> Response {
    let Some(text) = bearer(request.headers()) else {
        return invalid();
    };
    let token = match state.tokens.authenticate(text).await {
        Ok(token) => token,
        Err(TokensError::Invalid) => return invalid(),
        Err(error) => return Problem::from(error).into_response(),
    };
    let key = RateKey::Token(token.id.to_string());
    if let Err(problem) = state.rate_limits.check(Policy::Mcp, &key) {
        return problem.into_response();
    }
    request.extensions_mut().insert(token);
    next.run(request).await
}

/// The token of `Authorization: Bearer <token>`, if the header has that form.
fn bearer(headers: &HeaderMap) -> Option<&str> {
    let value = headers.get(AUTHORIZATION)?.to_str().ok()?;
    let (scheme, token) = value.trim().split_once(' ')?;
    scheme.eq_ignore_ascii_case(BEARER).then(|| token.trim())
}

/// `401 token.invalid`, asking for a bearer token.
fn invalid() -> Response {
    let mut response = Problem::from(TokensError::Invalid).into_response();
    let challenge = HeaderValue::from_static("Bearer");
    response.headers_mut().insert(WWW_AUTHENTICATE, challenge);
    response
}

#[cfg(test)]
mod tests {
    use super::*;

    fn headers(value: &str) -> HeaderMap {
        let mut headers = HeaderMap::new();
        headers.insert(AUTHORIZATION, HeaderValue::from_str(value).unwrap());
        headers
    }

    #[test]
    fn only_a_bearer_authorization_carries_a_token() {
        assert_eq!(bearer(&headers("Bearer lp_pat_abc")), Some("lp_pat_abc"));
        assert_eq!(bearer(&headers("bearer  lp_pat_abc ")), Some("lp_pat_abc"));
        assert_eq!(bearer(&headers("Basic bHA6cGF0")), None);
        assert_eq!(bearer(&headers("lp_pat_abc")), None);
        assert_eq!(bearer(&HeaderMap::new()), None);
    }
}
