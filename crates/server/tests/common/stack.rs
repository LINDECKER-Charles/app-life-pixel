//! What the library and account API tests share (H6): the server over a test database, a test
//! bucket prefix, recorded emails and product events, and the requests of a signed-in browser.

#![allow(dead_code)] // Each test file uses its own share of the helpers.

use std::collections::HashMap;
use std::sync::Arc;

use axum::body::{Body, Bytes};
use axum::extract::connect_info::MockConnectInfo;
use axum::http::header::{CONTENT_TYPE, IF_MATCH};
use axum::http::request::Builder;
use axum::http::{Method, Request};
use life_pixel_server::accounts;
use life_pixel_server::app;
use life_pixel_server::routes::library::DOCUMENT_MEDIA_TYPE;
use life_pixel_server::state::{AppState, Backends};
use life_pixel_server::storage::HostedLibraryStore;
use life_pixel_server::testing::{TestDatabase, TestStorage, load_test_env};
use life_pixel_service::accounts::memory::RecordingMailer;
use life_pixel_service::memory::RecordingEvents;
use life_pixel_service::testing::sample_document;
use serde_json::{Value, json};
use tempfile::TempDir;

use crate::router::auth::{Browser, from_app, sign_up, with_json, without_body};
use crate::router::{Answer, Database, built_app, client_peer, local_env, read_config, send};

/// The prefix of the API's paths.
pub const API: &str = "/api/v1";

/// The server's state over a test database and bucket prefix.
pub struct ApiStack {
    pub database: TestDatabase,
    pub storage: TestStorage,
    pub events: Arc<RecordingEvents>,
    pub state: AppState,
    _app_dir: TempDir,
}

impl ApiStack {
    /// The local configuration over a new database and bucket prefix.
    pub async fn new() -> Self {
        Self::with(|_| {}).await
    }

    /// The local configuration that `change` adjusts, over a new database and bucket prefix.
    pub async fn with(change: impl FnOnce(&mut HashMap<String, String>)) -> Self {
        load_test_env();
        let database = TestDatabase::create().await.unwrap();
        let storage = TestStorage::create().unwrap();
        let app_dir = built_app();
        let mut env = local_env(app_dir.path());
        change(&mut env);
        let config = read_config(&env).unwrap();
        let events = Arc::new(RecordingEvents::new());
        let mut ports = accounts::hosted_ports(database.pool(), Arc::new(RecordingMailer::new()));
        ports.events = events.clone();
        let store = HostedLibraryStore::new(database.pool().clone(), storage.objects());
        let backends = Backends {
            readiness: Arc::new(Database { answers: true }),
            library_store: Arc::new(store),
            accounts: ports,
        };
        let state = AppState::new(config, backends).unwrap();
        Self {
            database,
            storage,
            events,
            state,
            _app_dir: app_dir,
        }
    }

    /// The answer to `request`, from the usual peer.
    pub async fn send(&self, request: Request<Body>) -> Answer {
        let router = app::public_router(self.state.clone()).layer(MockConnectInfo(client_peer()));
        send(router, request).await
    }

    /// Signs `email` up in English: its session.
    pub async fn sign_up(&self, email: &str) -> Browser {
        let answer = self.send(sign_up(email, "en")).await;
        assert_eq!(answer.status, 201, "{:?}", answer.body);
        Browser::of(&answer)
    }

    /// The answer to `request`, checked to have `status`.
    pub async fn expect(&self, status: u16, request: Request<Body>) -> Answer {
        let answer = self.send(request).await;
        assert_eq!(answer.status.as_u16(), status, "{:?}", answer.body);
        answer
    }

    /// Creates the project `name` for `browser`: its id.
    pub async fn project(&self, browser: &Browser, name: &str) -> String {
        let body = json!({ "name": name });
        let created = with_json(api(browser, Method::POST, "/projects"), &body);
        self.created(created).await["id"]
            .as_str()
            .unwrap()
            .to_owned()
    }

    /// The params of the answer to `request`, checked to be the problem `code` with `status`.
    pub async fn refused(&self, request: Request<Body>, (status, code): (u16, &str)) -> Value {
        self.send(request).await.assert_problem(status, code)
    }

    /// The body of the answer to `request`, checked to be `201`.
    pub async fn created(&self, request: Request<Body>) -> Value {
        self.expect(201, request).await.json()
    }

    /// The number `query` counts, with `value` as its `$1`.
    pub async fn count(&self, query: &'static str, value: &str) -> i64 {
        let query = sqlx::query_scalar(query).bind(value);
        query.fetch_one(self.database.pool()).await.unwrap()
    }
}

/// `GET /api/v1<path>` with `browser`'s session.
pub fn get(browser: &Browser, path: &str) -> Request<Body> {
    browser.get(&format!("{API}{path}"))
}

/// `method /api/v1<path>` from the app, with `browser`'s session and its token.
pub fn api(browser: &Browser, method: Method, path: &str) -> Builder {
    browser.signed(from_app(method, &format!("{API}{path}")))
}

/// `DELETE /api/v1<path>` from the app, with `browser`'s session and its token.
pub fn delete(browser: &Browser, path: &str) -> Request<Body> {
    without_body(api(browser, Method::DELETE, path))
}

/// The import of `document` into `project`, for `browser`.
pub fn import(browser: &Browser, project: &str, document: &Bytes) -> Request<Body> {
    let path = format!("/projects/{project}/animations");
    with_document(api(browser, Method::POST, &path), document)
}

/// The sample document titled `title`.
pub fn sample(title: &str) -> Bytes {
    sample_document(title).1
}

/// `builder` with `If-Match: "<version>"`.
pub fn if_match(builder: Builder, version: u64) -> Builder {
    builder.header(IF_MATCH, format!("\"{version}\""))
}

/// The request of `builder`, with `document` as a document.
pub fn with_document(builder: Builder, document: &Bytes) -> Request<Body> {
    let builder = builder.header(CONTENT_TYPE, DOCUMENT_MEDIA_TYPE);
    builder.body(Body::from(document.clone())).unwrap()
}
