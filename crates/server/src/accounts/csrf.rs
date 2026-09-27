//! CSRF: a `POST`, `PUT`, `PATCH` or `DELETE` carrying the session cookie comes from an allowed
//! origin — `LP_PUBLIC_URL`'s or one of `LP_ALLOWED_ORIGINS` — and, when its session is live,
//! sends the session's token in `X-CSRF-Token`: base64url(HMAC-SHA256(`LP_SESSION_SECRET`, the
//! session's hash)). Signing up and in, which open a session, check the origin alone.

use axum::extract::{Request, State};
use axum::http::header::ORIGIN;
use axum::http::{HeaderMap, Method};
use axum::middleware::Next;
use axum::response::Response;
use base64::Engine as _;
use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use hmac::{Hmac, KeyInit, Mac};
use life_pixel_service::accounts::TokenHash;
use sha2::Sha256;

use super::cookie;
use super::session::SessionState;
use crate::config::{Config, HmacKey};
use crate::http::problem::{Problem, codes};
use crate::openapi::CSRF_HEADER;
use crate::state::AppState;

/// The CSRF token of the session of `token_hash`, under `key`.
#[must_use]
pub fn token(key: &HmacKey, token_hash: &TokenHash) -> String {
    URL_SAFE_NO_PAD.encode(mac(key, token_hash).finalize().into_bytes())
}

/// Whether `sent` is the CSRF token of the session of `token_hash`, compared in constant time.
#[must_use]
pub fn is_token_of(key: &HmacKey, token_hash: &TokenHash, sent: &str) -> bool {
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
/// `auth.csrf`.
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
        let live = request.extensions().get::<SessionState>();
        if let Some(SessionState::Live(current)) = live {
            let sent = headers
                .get(CSRF_HEADER)
                .and_then(|value| value.to_str().ok());
            let key = &state.config.secrets.session;
            let hash = &current.session.token_hash;
            if !sent.is_some_and(|sent| is_token_of(key, hash, sent)) {
                return Err(refused());
            }
        }
    }
    Ok(next.run(request).await)
}

/// Middleware of the routes that open a session: refuses an unsafe request from another origin,
/// with or without a cookie.
///
/// # Errors
///
/// `auth.csrf`.
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

/// Whether `method` changes state.
fn is_unsafe(method: &Method) -> bool {
    [Method::POST, Method::PUT, Method::PATCH, Method::DELETE].contains(method)
}

/// Whether the request's `Origin` is `LP_PUBLIC_URL`'s or one of `LP_ALLOWED_ORIGINS`.
fn is_allowed_origin(config: &Config, headers: &HeaderMap) -> bool {
    let Some(origin) = headers.get(ORIGIN).and_then(|value| value.to_str().ok()) else {
        return false;
    };
    let allowed = std::iter::once(&config.public_url).chain(&config.allowed_origins);
    allowed
        .into_iter()
        .any(|known| known.as_str().eq_ignore_ascii_case(origin))
}

fn mac(key: &HmacKey, token_hash: &TokenHash) -> Hmac<Sha256> {
    let mut mac = <Hmac<Sha256> as KeyInit>::new_from_slice(key.as_bytes())
        .unwrap_or_else(|_| unreachable!("HMAC takes a key of any length"));
    mac.update(token_hash.as_bytes());
    mac
}

fn refused() -> Problem {
    Problem::new(codes::AUTH_CSRF)
}

#[cfg(test)]
mod tests {
    use life_pixel_service::accounts::SecretToken;

    use super::*;
    use crate::config::{FromVariable, HMAC_KEY_BYTES};

    fn key(byte: u8) -> HmacKey {
        HmacKey::from_variable(&format!("{byte:02x}").repeat(HMAC_KEY_BYTES)).unwrap()
    }

    #[test]
    fn a_token_is_the_base64url_hmac_of_the_session_hash() {
        let hash = SecretToken::generate().hash();
        let token = token(&key(1), &hash);
        assert_eq!(token.len(), 43);
        assert!(is_token_of(&key(1), &hash, &token));
        assert!(!is_token_of(&key(2), &hash, &token));
        assert!(!is_token_of(
            &key(1),
            &SecretToken::generate().hash(),
            &token
        ));
        assert!(!is_token_of(&key(1), &hash, "not base64url!"));
        assert!(!is_token_of(&key(1), &hash, ""));
    }
}
