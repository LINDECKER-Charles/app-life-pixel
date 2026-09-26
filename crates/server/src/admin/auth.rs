//! Who may call the internal admin API: the admin server alone, with `Authorization: Bearer
//! <LP_ADMIN_API_SECRET>` compared in constant time, on behalf of the admin that `X-Admin-Id`
//! and `X-Admin-Email` name. Anything else is `admin.unauthenticated`.

use axum::extract::{FromRequestParts, Request, State};
use axum::http::HeaderMap;
use axum::http::header::AUTHORIZATION;
use axum::http::request::Parts;
use axum::middleware::Next;
use axum::response::{IntoResponse, Response};
use hmac::{Hmac, KeyInit, Mac};
use life_pixel_service::admin::AdminIdentity;
use sha2::Sha256;

use crate::config::{FromVariable, HmacKey};
use crate::http::problem::{Problem, codes};
use crate::state::AppState;

/// The header naming the acting admin's id.
pub const ADMIN_ID_HEADER: &str = "X-Admin-Id";
/// The header naming the acting admin's address.
pub const ADMIN_EMAIL_HEADER: &str = "X-Admin-Email";
/// The authentication scheme of `Authorization`.
const BEARER_SCHEME: &str = "bearer";

/// The admin a request acts for, once [`authenticate`] let it through.
#[derive(Clone, Debug)]
pub struct ActingAdmin(pub AdminIdentity);

/// Middleware on the whole internal admin API: lets a request through with its [`ActingAdmin`]
/// only when it carries the secret and the admin's identity.
pub async fn authenticate(
    State(state): State<AppState>,
    mut request: Request,
    next: Next,
) -> Response {
    let Some(admin) = identify(&state.config.secrets.admin_api, request.headers()) else {
        return Problem::new(codes::ADMIN_UNAUTHENTICATED).into_response();
    };
    request.extensions_mut().insert(ActingAdmin(admin));
    next.run(request).await
}

impl<S: Send + Sync> FromRequestParts<S> for ActingAdmin {
    type Rejection = Problem;

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Problem> {
        let admin = parts.extensions.get::<Self>().cloned();
        admin.ok_or_else(|| Problem::new(codes::ADMIN_UNAUTHENTICATED))
    }
}

/// The admin `headers` name, when they also carry `secret`.
fn identify(secret: &HmacKey, headers: &HeaderMap) -> Option<AdminIdentity> {
    let authorization = header(headers, AUTHORIZATION.as_str())?;
    let (scheme, token) = authorization.split_once(' ')?;
    let is_authorized = scheme.eq_ignore_ascii_case(BEARER_SCHEME) && is_secret(secret, token);
    if !is_authorized {
        return None;
    }
    let id = header(headers, ADMIN_ID_HEADER)?;
    let email = header(headers, ADMIN_EMAIL_HEADER)?;
    AdminIdentity::parse(id, email)
}

fn header<'a>(headers: &'a HeaderMap, name: &str) -> Option<&'a str> {
    headers.get(name)?.to_str().ok()
}

/// Whether `token` is `secret`'s 64 hexadecimal characters, whatever their case. The bytes are
/// compared through their HMAC under the secret, whose check takes the same time wherever they
/// differ.
fn is_secret(secret: &HmacKey, token: &str) -> bool {
    let Some(sent) = HmacKey::from_variable(token.trim()) else {
        return false;
    };
    let expected = mac(secret, secret).finalize().into_bytes();
    mac(secret, &sent).verify_slice(&expected).is_ok()
}

fn mac(key: &HmacKey, message: &HmacKey) -> Hmac<Sha256> {
    let mut mac = <Hmac<Sha256> as KeyInit>::new_from_slice(key.as_bytes())
        .unwrap_or_else(|_| unreachable!("HMAC takes a key of any length"));
    mac.update(message.as_bytes());
    mac
}

#[cfg(test)]
mod tests {
    use axum::http::HeaderValue;

    use super::*;

    const SECRET: &str = "5f0c3a1e9b7d2468ace013579bdf2468ace013579bdf2468ace013579bdf2468";

    fn secret() -> HmacKey {
        HmacKey::from_variable(SECRET).unwrap_or_else(|| unreachable!("a valid key"))
    }

    fn headers(pairs: &[(&'static str, &str)]) -> HeaderMap {
        let mut headers = HeaderMap::new();
        for (name, value) in pairs {
            let value = HeaderValue::from_str(value).unwrap_or_else(|_| unreachable!());
            headers.insert(*name, value);
        }
        headers
    }

    #[test]
    fn the_secret_and_the_identity_let_a_request_through() {
        let bearer = format!("Bearer {}", SECRET.to_uppercase());
        let sent = headers(&[
            ("authorization", &bearer),
            ("x-admin-id", "0190f6a2"),
            ("x-admin-email", "ada@example.org"),
        ]);
        let admin = identify(&secret(), &sent);
        assert_eq!(
            admin.map(|admin| admin.id().to_owned()),
            Some("0190f6a2".into())
        );
    }

    #[test]
    fn another_secret_or_scheme_is_refused() {
        let other = SECRET.replace('5', "6");
        for authorization in [
            format!("Bearer {other}"),
            format!("Basic {SECRET}"),
            SECRET.to_owned(),
            "Bearer short".to_owned(),
        ] {
            let sent = headers(&[
                ("authorization", &authorization),
                ("x-admin-id", "0190f6a2"),
                ("x-admin-email", "ada@example.org"),
            ]);
            assert!(identify(&secret(), &sent).is_none(), "{authorization}");
        }
    }
}
