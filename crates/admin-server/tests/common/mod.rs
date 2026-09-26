//! What the tests share: the local configuration, a state whose database never answers, a
//! signed-in admin for the routes behind the session, fake upstream servers, and one request at
//! a time.

#![allow(dead_code)] // Each test file uses its own share of the helpers.
#![allow(clippy::unwrap_used, clippy::expect_used)] // A helper fails its test by panicking.

use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::Arc;
use std::time::Duration;

use axum::Router;
use axum::body::Body;
use axum::extract::Request;
use axum::http::{HeaderMap, StatusCode};
use axum::middleware::{Next, from_fn};
use axum::response::Response;
use life_pixel_admin_server::admins::clock::SystemClock;
use life_pixel_admin_server::admins::sessions::SessionToken;
use life_pixel_admin_server::admins::store::AdminIdentity;
use life_pixel_admin_server::app::API_PREFIX;
use life_pixel_admin_server::config::Config;
use life_pixel_admin_server::http::problem::ensure_problem;
use life_pixel_admin_server::http::session::{CurrentAdmin, SessionState};
use life_pixel_admin_server::state::AppState;
use serde_json::Value;
use sqlx::PgPool;
use sqlx::postgres::PgPoolOptions;
use tokio::net::TcpListener;
use tower::ServiceExt;
use uuid::Uuid;

/// A key of the right shape, never a real one.
pub const SECRET: &str = "5f0c3a1e9b7d2468ace013579bdf2468ace013579bdf2468ace013579bdf2468";
/// Another one, for the TOTP secrets.
pub const TOTP_KEY: &str = "0b1d2f3a4c5e6f708192a3b4c5d6e7f8091a2b3c4d5e6f708192a3b4c5d6e7f8";
/// The internal admin API's secret the relay sends: a test value.
pub const RELAY_SECRET: &str = "relay-secret-of-the-tests";
/// The console's origin.
pub const CONSOLE_ORIGIN: &str = "http://localhost:8463";
/// The signed-in admin's address.
pub const ADMIN_EMAIL: &str = "ops@example.org";

/// The values of `.env.example`, the internal admin API and the sources at `upstream`.
const LOCAL_VALUES: [(&str, &str); 16] = [
    ("LPA_ENVIRONMENT", "local"),
    ("LPA_HTTP_ADDR", "127.0.0.1:8463"),
    ("LPA_METRICS_ADDR", "127.0.0.1:8464"),
    ("LPA_PUBLIC_URL", CONSOLE_ORIGIN),
    ("LPA_ALLOWED_ORIGINS", "http://localhost:4263"),
    (
        "LPA_DATABASE_URL",
        "postgres://life_pixel_admin:local@127.0.0.1:5460/life_pixel_admin",
    ),
    ("LPA_SESSION_SECRET", SECRET),
    ("LPA_TOTP_KEY", TOTP_KEY),
    ("LPA_SERVER_ADMIN_API_SECRET", RELAY_SECRET),
    ("LPA_ENVIRONMENTS", "staging,production"),
    ("LPA_METRICS_SELECTOR_STAGING", "{environment=\"staging\"}"),
    (
        "LPA_CONTAINERS_SELECTOR_STAGING",
        "container_label_com_docker_compose_project=\"life-pixel-staging\"",
    ),
    (
        "LPA_LOGS_SELECTOR_STAGING",
        "compose_project=\"life-pixel-staging\"",
    ),
    (
        "LPA_ALERTS_FILTER",
        "project=\"life-pixel\",environment=\"{ENV}\"",
    ),
    ("LPA_GRAFANA_LOGS_DATASOURCE", "vl"),
    ("RUST_LOG", "info"),
];

/// The local values, the internal admin API at `server` and every source at `sources` when
/// given, the repository's catalogues and no built console.
pub fn local_env(server: &str, sources: Option<&str>) -> HashMap<String, String> {
    let manifest = env!("CARGO_MANIFEST_DIR");
    let mut env: HashMap<String, String> = LOCAL_VALUES
        .into_iter()
        .map(|(name, value)| (name.to_owned(), value.to_owned()))
        .collect();
    env.insert("LPA_I18N_DIR".into(), format!("{manifest}/../../i18n"));
    env.insert("LPA_APP_DIR".into(), format!("{manifest}/tests/no-console"));
    env.insert("LPA_SERVER_ADMIN_API_URL".into(), server.to_owned());
    for (variable, path) in [
        ("LPA_VICTORIAMETRICS_URL", "/vm"),
        ("LPA_VICTORIALOGS_URL", "/vl"),
        ("LPA_ALERTMANAGER_URL", "/am"),
        ("LPA_GRAFANA_URL", "/grafana"),
    ] {
        if let Some(sources) = sources {
            env.insert(variable.to_owned(), format!("{sources}{path}"));
        }
    }
    env
}

/// The configuration of `env`.
pub fn config(env: &HashMap<String, String>) -> Config {
    Config::from_lookup(&|name| env.get(name).cloned()).unwrap()
}

/// A pool that never connects: nothing listens on its port.
pub fn unreachable_pool() -> PgPool {
    PgPoolOptions::new()
        .acquire_timeout(Duration::from_millis(300))
        .connect_lazy("postgres://nobody:nothing@127.0.0.1:9/none")
        .unwrap()
}

/// The state of `env`, its database unreachable.
pub fn state(env: &HashMap<String, String>) -> AppState {
    AppState::new(config(env), unreachable_pool(), Arc::new(SystemClock)).unwrap()
}

/// The signed-in admin of [`signed_in`].
pub fn admin() -> AdminIdentity {
    AdminIdentity {
        id: Uuid::from_u128(0x0190_f6a2_7c1e_7000_8000_0000_0000_0001),
        email: ADMIN_EMAIL.to_owned(),
    }
}

/// `routes` under `/api/admin/v1`, every request carrying a live session of [`admin`], every
/// error a problem.
pub fn signed_in(routes: Router<AppState>, state: AppState) -> Router {
    Router::new()
        .nest(API_PREFIX, routes.layer(from_fn(ensure_problem)))
        .layer(from_fn(live_session))
        .with_state(state)
}

async fn live_session(mut request: Request, next: Next) -> Response {
    let current = CurrentAdmin {
        token: SessionToken::generate(),
        admin: admin(),
    };
    request.extensions_mut().insert(SessionState::Live(current));
    next.run(request).await
}

/// Serves `router` on a free port of 127.0.0.1: its base URL.
pub async fn spawn(router: Router) -> String {
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let address: SocketAddr = listener.local_addr().unwrap();
    tokio::spawn(async move { axum::serve(listener, router).await });
    format!("http://{address}")
}

/// A base URL where nothing listens.
pub const NOTHING_LISTENS: &str = "http://127.0.0.1:9";

/// The answer of `router` to `request`: status, headers, body.
pub async fn send(router: &Router, request: Request<Body>) -> (StatusCode, HeaderMap, Vec<u8>) {
    let response = router.clone().oneshot(request).await.unwrap();
    let (parts, body) = response.into_parts();
    let bytes = axum::body::to_bytes(body, usize::MAX).await.unwrap();
    (parts.status, parts.headers, bytes.to_vec())
}

/// The body as JSON.
pub fn json(body: &[u8]) -> Value {
    serde_json::from_slice(body).unwrap()
}

/// A `GET` of `path`.
pub fn get(path: &str) -> Request<Body> {
    Request::get(path).body(Body::empty()).unwrap()
}
