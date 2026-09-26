//! The session cookie, `__Host-lpa_session`: bound to its host, `Secure`, `HttpOnly`,
//! `SameSite=Strict` and `Path=/`, and kept by the browser 8 hours at most — the server ends the
//! session sooner when it idles.

use axum::http::header::{COOKIE, SET_COOKIE};
use axum::http::{HeaderMap, HeaderValue};

use crate::admins::sessions::{SESSION_LIFETIME, SessionToken};

/// The session cookie's name.
pub const SESSION_COOKIE: &str = "__Host-lpa_session";
/// The attributes of every session cookie, after its `Max-Age`.
const ATTRIBUTES: &str = "Path=/; Secure; HttpOnly; SameSite=Strict";

/// The value of the session cookie a request carries, if any.
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
pub fn set(token: &SessionToken) -> HeaderValue {
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
    fn the_cookie_is_strict_bound_to_its_host_and_lives_eight_hours() {
        let token = SessionToken::generate();
        let expected = format!(
            "__Host-lpa_session={}; Max-Age=28800; Path=/; Secure; HttpOnly; SameSite=Strict",
            token.expose()
        );
        assert_eq!(set(&token).to_str().unwrap(), expected);
        let mut headers = HeaderMap::new();
        let pair = HeaderValue::from_static("a=1; __Host-lpa_session=abc; __Host-lp_session=x");
        headers.append(COOKIE, pair);
        assert_eq!(read(&headers).as_deref(), Some("abc"));
        headers.append(SET_COOKIE, cleared());
        assert!(is_set_in(&headers));
    }
}
