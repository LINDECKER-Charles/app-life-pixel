//! CSRF, as the server's (H5): a `POST`, `PUT`, `PATCH` or `DELETE` carrying the session cookie
//! comes from an allowed origin — `LPA_PUBLIC_URL`'s or one of `LPA_ALLOWED_ORIGINS` — and, when
//! its session is live, sends the session's token in `X-CSRF-Token`:
//! base64url(HMAC-SHA256(`LPA_SESSION_SECRET`, the session's hash)). Signing in checks the origin
//! alone. Anything else is `admin.csrf`.

use axum::extract::{Request, State};
use axum::http::header::ORIGIN;
use axum::http::{HeaderMap, Method};
use axum::middleware::Next;
use axum::response::Response;
use base64::Engine as _;
use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use hmac::{Hmac, KeyInit, Mac};
use sha2::Sha256;

use super::{SessionState, cookie};
use crate::admins::sessions::TokenHash;
use crate::config::{Config, Key};
use crate::http::problem::{Problem, codes};
use crate::state::AppState;

/// The header carrying a session's CSRF token.
pub const CSRF_HEADER: &str = "X-CSRF-Token";

/// The CSRF token of the session of `token_hash`, under `key`.
#[must_use]
pub fn token(key: &Key, token_hash: &TokenHash) -> String {
    URL_SAFE_NO_PAD.encode(mac(key, token_hash).finalize().into_bytes())
}

/// Whether `sent` is the CSRF token of the session of `token_hash`, compared in constant time.
#[must_use]
pub fn is_token_of(key: &Key, token_hash: &TokenHash, sent: &str) -> bool {
    let Ok(bytes) = URL_SAFE_NO_PAD.decode(sent) else {
        return false;
    };
    mac(key, token_hash).verify_slice(&bytes).is_ok()
}

/// Middleware of the routes a session acts through: refuses an unsafe request that carries the
/// session cookie from another origin, or without its live session's token.
///
/// # Errors
///
/// `admin.csrf`.
pub async fn protect(
    State(state): State<AppState>,
    request: Request,
    next: Next,
) -> Result<Response, Problem> {
    if is_unsafe(request.method()) && cookie::read(request.headers()).is_some() {
        let headers = request.headers();
        if !is_allowed_origin(&state.config, headers) {
            return Err(refused());
        }
        if let Some(SessionState::Live(current)) = request.extensions().get::<SessionState>() {
            let sent = headers
                .get(CSRF_HEADER)
                .and_then(|value| value.to_str().ok());
            let key = &state.config.session_secret;
            let hash = current.token.hash();
            if !sent.is_some_and(|sent| is_token_of(key, &hash, sent)) {
                return Err(refused());
            }
        }
    }
    Ok(next.run(request).await)
}

/// Middleware of sign-in: refuses an unsafe request from another origin, with or without a
/// cookie.
///
/// # Errors
///
/// `admin.csrf`.
pub async fn check_origin(
    State(state): State<AppState>,
    request: Request,
    next: Next,
) -> Result<Response, Problem> {
    if is_unsafe(request.method()) && !is_allowed_origin(&state.config, request.headers()) {
        return Err(refused());
    }
    Ok(next.run(request).await)
}

fn is_unsafe(method: &Method) -> bool {
    [Method::POST, Method::PUT, Method::PATCH, Method::DELETE].contains(method)
}

/// Whether the request's `Origin` is `LPA_PUBLIC_URL`'s or one of `LPA_ALLOWED_ORIGINS`.
fn is_allowed_origin(config: &Config, headers: &HeaderMap) -> bool {
    let Some(origin) = headers.get(ORIGIN).and_then(|value| value.to_str().ok()) else {
        return false;
    };
    std::iter::once(&config.public_url)
        .chain(&config.allowed_origins)
        .any(|known| known.as_str().eq_ignore_ascii_case(origin))
}

fn mac(key: &Key, token_hash: &TokenHash) -> Hmac<Sha256> {
    let mut mac = <Hmac<Sha256> as KeyInit>::new_from_slice(key.as_bytes())
        .unwrap_or_else(|_| unreachable!("HMAC takes a key of any length"));
    mac.update(token_hash.as_bytes());
    mac
}

fn refused() -> Problem {
    Problem::new(codes::ADMIN_CSRF)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::admins::sessions::SessionToken;
    use crate::config::FromVariable;

    fn key(digit: char) -> Key {
        Key::from_variable(&digit.to_string().repeat(64)).unwrap()
    }

    #[test]
    fn a_token_is_the_base64url_hmac_of_the_session_hash() {
        let hash = SessionToken::generate().hash();
        let token = token(&key('1'), &hash);
        assert_eq!(token.len(), 43);
        assert!(is_token_of(&key('1'), &hash, &token));
        assert!(!is_token_of(&key('2'), &hash, &token));
        let other = SessionToken::generate().hash();
        assert!(!is_token_of(&key('1'), &other, &token));
        assert!(!is_token_of(&key('1'), &hash, "not base64url!"));
    }
}
