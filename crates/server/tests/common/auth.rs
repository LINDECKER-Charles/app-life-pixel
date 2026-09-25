//! What the auth tests share: requests from the app's origin, and a browser keeping a session's
//! cookie and CSRF token.

use std::net::SocketAddr;

use axum::body::Body;
use axum::http::header::{CONTENT_TYPE, COOKIE, ORIGIN};
use axum::http::request::Builder;
use axum::http::{Method, Request};
use life_pixel_server::openapi::{CSRF_HEADER, SESSION_COOKIE};
use serde_json::{Value, json};

use super::{Answer, request};

/// `LP_PUBLIC_URL` of the local configuration: the app's origin.
pub const APP_ORIGIN: &str = "http://localhost:8460";
/// One of `LP_ALLOWED_ORIGINS` of the local configuration.
pub const DEV_ORIGIN: &str = "http://localhost:4260";
/// An origin the configuration does not know.
pub const OTHER_ORIGIN: &str = "https://attacker.example";
/// A password of an allowed length.
pub const PASSWORD: &str = "correct horse battery";
/// Another password of an allowed length.
pub const NEW_PASSWORD: &str = "staple battery horse correct";
/// The attributes of the session cookie, after its value.
pub const COOKIE_ATTRIBUTES: &str = "Max-Age=2592000; Path=/; Secure; HttpOnly; SameSite=Lax";
/// The session cookie of a response that clears it.
pub const CLEARED_COOKIE: &str = "__Host-lp_session=; Max-Age=0; Path=/; Secure; HttpOnly; \
                                  SameSite=Lax";

/// The `n`th of the peers the tests of one route spread their requests over, so that its rate
/// limit is not reached.
pub fn peer(n: u8) -> SocketAddr {
    SocketAddr::from(([198, 51, 100, n], 50_000))
}

/// A request builder for `method` and `path`, from the app's origin.
pub fn from_app(method: Method, path: &str) -> Builder {
    request(method, path).header(ORIGIN, APP_ORIGIN)
}

/// The request of `builder`, with `body` as JSON.
pub fn with_json(builder: Builder, body: &Value) -> Request<Body> {
    let builder = builder.header(CONTENT_TYPE, "application/json");
    builder.body(Body::from(body.to_string())).unwrap()
}

/// The request of `builder`, without a body.
pub fn without_body(builder: Builder) -> Request<Body> {
    builder.body(Body::empty()).unwrap()
}

/// A sign-up of `email` with [`PASSWORD`] in `language`, from the app's origin.
pub fn sign_up(email: &str, language: &str) -> Request<Body> {
    let body = json!({ "email": email, "password": PASSWORD, "language": language });
    with_json(from_app(Method::POST, "/api/v1/auth/sign-up"), &body)
}

/// A sign-in of `email` with `password`, from the app's origin.
pub fn sign_in(email: &str, password: &str) -> Request<Body> {
    let body = json!({ "email": email, "password": password });
    with_json(from_app(Method::POST, "/api/v1/auth/sign-in"), &body)
}

/// The value of the session cookie `answer` sets, and its attributes, checked.
pub fn session_cookie(answer: &Answer) -> String {
    let set = answer.header("set-cookie");
    let (pair, attributes) = set.split_once("; ").unwrap();
    let value = pair.strip_prefix(&format!("{SESSION_COOKIE}=")).unwrap();
    assert_eq!(attributes, COOKIE_ATTRIBUTES);
    assert_eq!(value.len(), 43, "32 bytes in base64url: {value}");
    value.to_owned()
}

/// A browser holding a session: its cookie and its CSRF token.
#[derive(Clone, Debug)]
pub struct Browser {
    pub cookie: String,
    pub csrf: String,
}

impl Browser {
    /// The session `answer` opened: its cookie, checked, and the token of its body.
    pub fn of(answer: &Answer) -> Self {
        let csrf = answer.json()["csrfToken"].as_str().unwrap().to_owned();
        Self {
            cookie: session_cookie(answer),
            csrf,
        }
    }

    /// `builder` with the session cookie, and no token.
    pub fn cookie(&self, builder: Builder) -> Builder {
        builder.header(COOKIE, format!("{SESSION_COOKIE}={}", self.cookie))
    }

    /// `builder` with the session cookie and its token.
    pub fn signed(&self, builder: Builder) -> Builder {
        self.cookie(builder).header(CSRF_HEADER, &self.csrf)
    }

    /// `GET path` with the session cookie.
    pub fn get(&self, path: &str) -> Request<Body> {
        without_body(self.cookie(request(Method::GET, path)))
    }

    /// `POST path` from the app, with the cookie, the token and `body`.
    pub fn post(&self, path: &str, body: &Value) -> Request<Body> {
        with_json(self.signed(from_app(Method::POST, path)), body)
    }

    /// `PUT /api/v1/auth/password` from the app, with the cookie and the token.
    pub fn change_password(&self, current: &str, new: &str) -> Request<Body> {
        let body = json!({ "currentPassword": current, "newPassword": new });
        with_json(
            self.signed(from_app(Method::PUT, "/api/v1/auth/password")),
            &body,
        )
    }
}
