//! The session cookie, `__Host-lp_session`: read from a request, set and cleared on a response.
//! Bound to its host, it is `Secure`, `HttpOnly`, `SameSite=Lax` and `Path=/`, and lives as long
//! as a session unseen.

use axum::http::header::{COOKIE, SET_COOKIE};
use axum::http::{HeaderMap, HeaderValue};
use life_pixel_service::accounts::{SESSION_LIFETIME, SecretToken};

use crate::openapi::SESSION_COOKIE;

/// The attributes of every session cookie, after its `Max-Age`.
const ATTRIBUTES: &str = "Path=/; Secure; HttpOnly; SameSite=Lax";

/// The value of the session cookie a request carries, if any: the first one, as a browser sends
/// the most specific first.
#[must_use]
pub fn read(headers: &HeaderMap) -> Option<String> {
    headers
        .get_all(COOKIE)
        .iter()
        .filter_map(|value| value.to_str().ok())
        .flat_map(|value| value.split(';'))
        .find_map(|pair| {
            let (name, value) = pair.trim().split_once('=')?;
            (name == SESSION_COOKIE).then(|| value.trim_matches('"').to_owned())
        })
}

/// The `Set-Cookie` that stores `token` for a session's lifetime.
#[must_use]
pub fn set(token: &SecretToken) -> HeaderValue {
    let max_age = SESSION_LIFETIME.whole_seconds();
    let cookie = format!(
        "{SESSION_COOKIE}={}; Max-Age={max_age}; {ATTRIBUTES}",
        token.expose()
    );
    HeaderValue::try_from(cookie).unwrap_or_else(|_| cleared())
}

/// The `Set-Cookie` that removes the session cookie.
#[must_use]
pub fn cleared() -> HeaderValue {
    let cookie = format!("{SESSION_COOKIE}=; Max-Age=0; {ATTRIBUTES}");
    HeaderValue::try_from(cookie).unwrap_or(HeaderValue::from_static(""))
}

/// Whether `headers` already set or clear the session cookie.
#[must_use]
pub fn is_set_in(headers: &HeaderMap) -> bool {
    let prefix = format!("{SESSION_COOKIE}=");
    headers
        .get_all(SET_COOKIE)
        .iter()
        .any(|value| value.as_bytes().starts_with(prefix.as_bytes()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_session_cookie_is_found_among_others() {
        let mut headers = HeaderMap::new();
        headers.append(COOKIE, HeaderValue::from_static("theme=dark"));
        let pair = HeaderValue::from_static("a=1; __Host-lp_session=abc; b=2");
        headers.append(COOKIE, pair);
        assert_eq!(read(&headers).as_deref(), Some("abc"));
        assert_eq!(read(&HeaderMap::new()), None);
    }

    #[test]
    fn a_session_cookie_is_bound_to_its_host_for_thirty_days() {
        let token = SecretToken::generate();
        let set = set(&token);
        let expected = format!(
            "__Host-lp_session={}; Max-Age=2592000; Path=/; Secure; HttpOnly; SameSite=Lax",
            token.expose()
        );
        assert_eq!(set.to_str().unwrap(), expected);
        let cleared = "__Host-lp_session=; Max-Age=0; Path=/; Secure; HttpOnly; SameSite=Lax";
        assert_eq!(super::cleared().to_str().unwrap(), cleared);
        let mut headers = HeaderMap::new();
        assert!(!is_set_in(&headers));
        headers.append(SET_COOKIE, super::cleared());
        assert!(is_set_in(&headers));
    }
}
