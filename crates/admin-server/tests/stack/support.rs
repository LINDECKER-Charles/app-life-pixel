//! A router over a test database, on a clock the test moves, and its admins.

use std::net::SocketAddr;
use std::sync::{Arc, Mutex};

use axum::Router;
use axum::body::Body;
use axum::extract::Request;
use axum::extract::connect_info::MockConnectInfo;
use axum::http::StatusCode;
use axum::http::header::SET_COOKIE;
use life_pixel_admin_server::admins::clock::Clock;
use life_pixel_admin_server::admins::password::Password;
use life_pixel_admin_server::admins::store::AdminIdentity;
use life_pixel_admin_server::admins::totp::{TotpSecret, step_at};
use life_pixel_admin_server::app;
use life_pixel_admin_server::state::AppState;
use life_pixel_admin_server::testing::TestDatabase;
use serde_json::{Value, json};
use time::{Duration, OffsetDateTime};

use crate::common::{self, CONSOLE_ORIGIN, NOTHING_LISTENS};

/// The password of the admins [`Stack::admin`] creates: a test value.
pub const PASSWORD: &str = "correct horse battery staple";

/// A clock the test moves.
pub struct ManualClock(Mutex<OffsetDateTime>);

impl ManualClock {
    /// Moves the clock `by` forward.
    pub fn advance(&self, by: Duration) {
        *self.0.lock().unwrap() += by;
    }
}

impl Clock for ManualClock {
    fn now(&self) -> OffsetDateTime {
        *self.0.lock().unwrap()
    }
}

/// The admin server over a test database.
pub struct Stack {
    pub database: TestDatabase,
    pub state: AppState,
    pub router: Router,
    pub clock: Arc<ManualClock>,
}

/// An admin and its TOTP secret.
pub struct TestAdmin {
    pub identity: AdminIdentity,
    pub secret: TotpSecret,
}

/// A signed-in session: its cookie header and CSRF token.
pub struct Session {
    pub cookie: String,
    pub csrf_token: String,
}

impl Stack {
    /// A migrated test database, the router over it, the clock at now.
    pub async fn new() -> Self {
        let database = TestDatabase::create().await.unwrap();
        let clock = Arc::new(ManualClock(Mutex::new(OffsetDateTime::now_utc())));
        let env = common::local_env(NOTHING_LISTENS, None);
        let config = common::config(&env);
        let pool = database.pool().clone();
        let state = AppState::new(config, pool, Arc::clone(&clock) as Arc<dyn Clock>).unwrap();
        let peer = SocketAddr::from(([198, 51, 100, 4], 40_000));
        let router = app::router(state.clone()).layer(MockConnectInfo(peer));
        Self {
            database,
            state,
            router,
            clock,
        }
    }

    /// A new admin of `email`, with [`PASSWORD`].
    pub async fn admin(&self, email: &str) -> TestAdmin {
        let password = Password::new(PASSWORD.to_owned()).unwrap();
        let (identity, secret) = self.state.admins.create(email, &password).await.unwrap();
        TestAdmin { identity, secret }
    }

    /// The code of `admin`'s secret now, `steps` steps away.
    pub fn code(&self, admin: &TestAdmin, steps: i64) -> String {
        admin.secret.code_at(step_at(self.clock.now()) + steps)
    }

    /// Signs in with `password` and `code`: the status, the body, and the cookie set.
    pub async fn sign_in(
        &self,
        (email, password, code): (&str, &str, &str),
    ) -> (StatusCode, Value, Option<String>) {
        let body = json!({ "email": email, "password": password, "code": code });
        let request = Request::post("/api/admin/v1/auth/sign-in")
            .header("origin", CONSOLE_ORIGIN)
            .header("content-type", "application/json")
            .body(Body::from(body.to_string()))
            .unwrap();
        let (status, headers, body) = common::send(&self.router, request).await;
        let cookie = headers.get(SET_COOKIE).map(|value| {
            value
                .to_str()
                .unwrap()
                .split(';')
                .next()
                .unwrap()
                .to_owned()
        });
        (status, common::json(&body), cookie)
    }

    /// Signs `admin` in with its current code.
    pub async fn signed_in(&self, admin: &TestAdmin) -> Session {
        let code = self.code(admin, 0);
        let (status, body, cookie) = self.sign_in((&admin.identity.email, PASSWORD, &code)).await;
        assert_eq!(status, StatusCode::OK, "{body}");
        Session {
            cookie: cookie.unwrap(),
            csrf_token: body["csrfToken"].as_str().unwrap().to_owned(),
        }
    }

    /// `GET /auth/session` with `session`'s cookie: the status and the body.
    pub async fn session(&self, session: &Session) -> (StatusCode, Value, bool) {
        let request = Request::get("/api/admin/v1/auth/session")
            .header("cookie", &session.cookie)
            .body(Body::empty())
            .unwrap();
        let (status, headers, body) = common::send(&self.router, request).await;
        let body = if body.is_empty() {
            Value::Null
        } else {
            common::json(&body)
        };
        (status, body, headers.get(SET_COOKIE).is_some())
    }

    /// A `POST` of `path` from the console with `session`'s cookie and `csrf_token`, if any.
    pub async fn post(
        &self,
        path: &str,
        (session, csrf_token): (&Session, Option<&str>),
    ) -> (StatusCode, Value) {
        let mut request = Request::post(path)
            .header("origin", CONSOLE_ORIGIN)
            .header("cookie", &session.cookie);
        if let Some(token) = csrf_token {
            request = request.header("x-csrf-token", token);
        }
        let request = request.body(Body::empty()).unwrap();
        let (status, _, body) = common::send(&self.router, request).await;
        let body = if body.is_empty() {
            Value::Null
        } else {
            common::json(&body)
        };
        (status, body)
    }

    /// Drops the test database.
    pub async fn drop(self) {
        self.database.drop().await.unwrap();
    }
}
