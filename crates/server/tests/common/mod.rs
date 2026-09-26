//! What the router tests share: the local configuration over a temporary built app and the
//! repository's catalogues, a database that answers or not, an in-memory library and accounts,
//! and one request at a time.

#![allow(dead_code)] // Each test file uses its own share of the helpers.
#![allow(clippy::unwrap_used)] // A helper fails its test by panicking, as the test would.

pub mod accounts;
pub mod auth;

use std::collections::HashMap;
use std::net::SocketAddr;
use std::path::Path;
use std::sync::Arc;

use async_trait::async_trait;
use axum::Router;
use axum::body::{Body, Bytes};
use axum::extract::connect_info::MockConnectInfo;
use axum::http::{HeaderMap, Method, Request, StatusCode};
use axum::response::Response;
use life_pixel_server::app;
use life_pixel_server::config::{Config, ConfigError};
use life_pixel_server::readiness::Readiness;
use life_pixel_server::state::{AppState, Backends};
use life_pixel_service::admin::memory::unavailable_stores;
use life_pixel_service::mcp::memory::in_memory_stores as mcp_stores;
use life_pixel_service::memory::{InMemoryLibraryStore, RecordingEvents};
use life_pixel_service::support::memory::in_memory_stores;
use life_pixel_service::tokens::memory::InMemoryTokenStore;
use serde_json::Value;
use tempfile::TempDir;
use tower::ServiceExt;

use self::accounts::TestAccounts;

/// A secret of the right shape, never a real one.
pub const SECRET: &str = "5f0c3a1e9b7d2468ace013579bdf2468ace013579bdf2468ace013579bdf2468";

/// The files of the built app: its index, hashed bundles, the engine, the player.
pub const APP_FILES: [(&str, &str); 6] = [
    ("index.html", "<!doctype html><title>Life Pixel</title>"),
    ("main-7QX4ZK2M.js", "console.log('main');"),
    ("styles-3HB6WN8R.css", "body { margin: 0; }"),
    ("engine/life_pixel_wasm-7QX4ZK2M.js", "export {};"),
    ("life-pixel.js", "export {};"),
    (
        "media/icon.svg",
        "<svg xmlns=\"http://www.w3.org/2000/svg\"/>",
    ),
];

/// The values of `.env.example`, but the folders.
const LOCAL_VALUES: [(&str, &str); 31] = [
    ("LP_ENVIRONMENT", "local"),
    ("LP_HTTP_ADDR", "127.0.0.1:8460"),
    ("LP_METRICS_ADDR", "127.0.0.1:8461"),
    ("LP_ADMIN_API_ADDR", "127.0.0.1:8462"),
    ("LP_PUBLIC_URL", "http://localhost:8460"),
    ("LP_ALLOWED_ORIGINS", "http://localhost:4260"),
    (
        "LP_DATABASE_URL",
        "postgres://life_pixel:local@127.0.0.1:5460/life_pixel",
    ),
    ("LP_DATABASE_MAX_CONNECTIONS", "10"),
    ("LP_STORAGE_URL", "s3://life-pixel-local"),
    ("LP_S3_ENDPOINT", "http://127.0.0.1:5461"),
    ("LP_S3_REGION", "us-east-1"),
    ("LP_S3_ACCESS_KEY_ID", "local"),
    ("LP_S3_SECRET_ACCESS_KEY", "local"),
    ("LP_S3_PATH_STYLE", "true"),
    ("LP_SMTP_URL", "smtp://127.0.0.1:5462"),
    ("LP_MAIL_FROM", "Life Pixel <no-reply@localhost>"),
    ("LP_SESSION_SECRET", SECRET),
    ("LP_EVENTS_SECRET", SECRET),
    ("LP_EXPORT_LINK_SECRET", SECRET),
    ("LP_ADMIN_API_SECRET", SECRET),
    ("LP_PLAN_FREE_STORAGE_BYTES", "100000000"),
    ("LP_PLAN_FREE_MCP_CALLS_PER_DAY", "1000"),
    ("LP_MIN_CLIENT_VERSIONS", "web=0.0.0,desktop=0.0.0"),
    ("LP_TRUSTED_PROXIES", ""),
    ("LP_MCP_SERVER_NAME", "life-pixel-dev"),
    ("LP_LEGAL_PUBLISHER", "Publisher"),
    ("LP_LEGAL_ADDRESS", "Address"),
    ("LP_LEGAL_CONTACT", "contact@localhost"),
    ("LP_LEGAL_DIRECTOR", "Director"),
    ("LP_LEGAL_HOST", "Host"),
    ("RUST_LOG", "info"),
];

/// The values of `.env.example`, with the built app in `app_dir` and the repository's
/// catalogues.
pub fn local_env(app_dir: &Path) -> HashMap<String, String> {
    let i18n_dir = format!("{}/../../i18n", env!("CARGO_MANIFEST_DIR"));
    let folders = [
        ("LP_APP_DIR", app_dir.display().to_string()),
        ("LP_I18N_DIR", i18n_dir),
    ];
    let values = LOCAL_VALUES
        .into_iter()
        .map(|(name, value)| (name, value.to_owned()));
    values
        .chain(folders)
        .map(|(name, value)| (name.to_owned(), value))
        .collect()
}

/// A request without a body.
pub fn empty(method: Method, path: &str) -> Request<Body> {
    request(method, path).body(Body::empty()).unwrap()
}

/// A `GET` of `path` with the header `name` set to `value`.
pub fn get_with(path: &str, name: &str, value: &str) -> Request<Body> {
    let builder = request(Method::GET, path).header(name, value);
    builder.body(Body::empty()).unwrap()
}

/// The configuration of `env`.
pub fn read_config(env: &HashMap<String, String>) -> Result<Config, ConfigError> {
    Config::from_lookup(&|name| env.get(name).cloned())
}

/// A temporary folder holding [`APP_FILES`].
pub fn built_app() -> TempDir {
    let dir = tempfile::tempdir().unwrap();
    for (path, content) in APP_FILES {
        let path = dir.path().join(path);
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        std::fs::write(path, content).unwrap();
    }
    dir
}

/// A database that answers, or not.
pub struct Database {
    pub answers: bool,
}

#[async_trait]
impl Readiness for Database {
    async fn is_ready(&self) -> bool {
        self.answers
    }
}

/// A peer outside every trusted network.
pub fn client_peer() -> SocketAddr {
    SocketAddr::from(([192, 0, 2, 10], 50_000))
}

/// The server's state over a temporary built app, and the requests it answers.
pub struct TestServer {
    pub state: AppState,
    pub accounts: TestAccounts,
    _app_dir: TempDir,
}

impl TestServer {
    /// A server with the local configuration and a database that answers.
    pub fn new() -> Self {
        Self::with(|_| {}, true)
    }

    /// A server whose configuration `change` adjusts.
    pub fn with(change: impl FnOnce(&mut HashMap<String, String>), database_answers: bool) -> Self {
        let app_dir = built_app();
        let mut env = local_env(app_dir.path());
        change(&mut env);
        let config = read_config(&env).unwrap();
        let accounts = TestAccounts::new();
        let backends = Backends {
            readiness: Arc::new(Database {
                answers: database_answers,
            }),
            library_store: Arc::new(InMemoryLibraryStore::new()),
            accounts: accounts.ports(),
            events: Arc::new(RecordingEvents::new()),
            support: in_memory_stores(),
            admin: unavailable_stores(),
            tokens: Arc::new(InMemoryTokenStore::new()),
            mcp: mcp_stores(),
        };
        let state = AppState::new(config, backends).unwrap();
        Self {
            state,
            accounts,
            _app_dir: app_dir,
        }
    }

    /// The public router, as a connection from `peer` sees it.
    pub fn public_router(&self, peer: SocketAddr) -> Router {
        app::public_router(self.state.clone()).layer(MockConnectInfo(peer))
    }

    /// The answer to `GET path`.
    pub async fn get(&self, path: &str) -> Answer {
        self.send(request(Method::GET, path).body(Body::empty()).unwrap())
            .await
    }

    /// The answer of the public router to `request`, from [`client_peer`].
    pub async fn send(&self, request: Request<Body>) -> Answer {
        send(self.public_router(client_peer()), request).await
    }
}

/// A request builder for `method` and `path`.
pub fn request(method: Method, path: &str) -> axum::http::request::Builder {
    Request::builder().method(method).uri(path)
}

/// The answer of `router` to `request`.
pub async fn send(router: Router, request: Request<Body>) -> Answer {
    Answer::read(router.oneshot(request).await.unwrap()).await
}

/// A response, read whole.
#[derive(Debug)]
pub struct Answer {
    pub status: StatusCode,
    pub headers: HeaderMap,
    pub body: Bytes,
}

impl Answer {
    /// Reads `response` whole.
    pub async fn read(response: Response) -> Self {
        let (parts, body) = response.into_parts();
        let body = axum::body::to_bytes(body, usize::MAX).await.unwrap();
        Self {
            status: parts.status,
            headers: parts.headers,
            body,
        }
    }

    /// The header `name`, which must be present.
    pub fn header(&self, name: &str) -> &str {
        let value = self.headers.get(name);
        let value = value.unwrap_or_else(|| panic!("no {name} in {:?}", self.headers));
        value.to_str().unwrap()
    }

    /// The body, as JSON.
    pub fn json(&self) -> Value {
        serde_json::from_slice(&self.body).unwrap()
    }

    /// Checks that the answer is the problem `code`, with `status`, and returns its params.
    pub fn assert_problem(&self, status: u16, code: &str) -> Value {
        assert_eq!(self.status.as_u16(), status, "{code}: {:?}", self.body);
        assert_eq!(self.header("content-type"), "application/problem+json");
        let problem = self.json();
        assert_eq!(problem["code"], code);
        assert_eq!(problem["status"], status);
        assert_eq!(problem["type"], format!("urn:life-pixel:problem:{code}"));
        problem["params"].clone()
    }
}
