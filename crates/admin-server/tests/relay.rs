//! The relay to the internal admin API, against a fake one: what it sends — method, path,
//! query, body, secret, admin identity, request id, and nothing else —, and what it passes back.

#![allow(clippy::unwrap_used, clippy::expect_used)]

mod common;

use std::sync::{Arc, Mutex};

use axum::Router;
use axum::body::{Body, Bytes};
use axum::extract::{Request, State};
use axum::http::header::{CONTENT_DISPOSITION, CONTENT_TYPE, SET_COOKIE};
use axum::http::{HeaderMap, Method, StatusCode};
use axum::response::{IntoResponse, Response};
use common::{ADMIN_EMAIL, NOTHING_LISTENS, RELAY_SECRET, admin, get, json, send, signed_in};
use futures_util::stream;
use life_pixel_admin_server::relay;

/// A request the fake internal admin API received.
#[derive(Clone, Debug)]
struct Received {
    method: Method,
    uri: String,
    headers: HeaderMap,
    body: Bytes,
}

type Log = Arc<Mutex<Vec<Received>>>;

/// The size of the export the fake API streams back.
const EXPORT_BYTES: usize = 3 * 1024 * 1024;

async fn fake_api(State(log): State<Log>, request: Request) -> Response {
    let (parts, body) = request.into_parts();
    let body = axum::body::to_bytes(body, usize::MAX).await.unwrap();
    let path = parts.uri.path().to_owned();
    log.lock().unwrap().push(Received {
        method: parts.method,
        uri: parts.uri.to_string(),
        headers: parts.headers,
        body,
    });
    answer(&path)
}

fn answer(path: &str) -> Response {
    match path {
        "/internal/admin/v1/users/missing" => not_found(),
        "/internal/admin/v1/users/refused" => StatusCode::UNAUTHORIZED.into_response(),
        "/internal/admin/v1/users/0190/export" => export(),
        _ => (
            [
                (CONTENT_TYPE, "application/json"),
                (SET_COOKIE, "upstream=1"),
            ],
            "{\"items\":[]}",
        )
            .into_response(),
    }
}

fn not_found() -> Response {
    let problem = r#"{"type":"urn:life-pixel:problem:admin.user_not_found","status":404,"code":"admin.user_not_found","params":{}}"#;
    (
        StatusCode::NOT_FOUND,
        [(CONTENT_TYPE, "application/problem+json")],
        problem,
    )
        .into_response()
}

fn export() -> Response {
    let chunks = (0..EXPORT_BYTES / 1024).map(|_| Ok::<_, std::io::Error>(vec![7_u8; 1024]));
    (
        [
            (CONTENT_TYPE, "application/zip"),
            (CONTENT_DISPOSITION, "attachment; filename=\"export.zip\""),
            (SET_COOKIE, "upstream=1"),
        ],
        Body::from_stream(stream::iter(chunks)),
    )
        .into_response()
}

/// The relay, signed in, towards a fake internal admin API; and what the API receives.
async fn relay_to_fake() -> (Router, Log) {
    let log = Log::default();
    let api = Router::new()
        .fallback(fake_api)
        .with_state(Arc::clone(&log));
    let base = common::spawn(api).await;
    let env = common::local_env(&format!("{base}/internal/admin/v1"), None);
    (signed_in(relay::router(), common::state(&env)), log)
}

fn only(log: &Log) -> Received {
    let received = log.lock().unwrap();
    assert_eq!(received.len(), 1, "{received:?}");
    received[0].clone()
}

#[tokio::test]
async fn a_request_goes_on_with_the_secret_and_the_admin_and_nothing_else() {
    let (router, log) = relay_to_fake().await;
    let request = Request::get("/api/admin/v1/audit-log?limit=10&cursor=a%20b")
        .header("accept", "application/json")
        .header("cookie", "__Host-lpa_session=secret-of-the-browser")
        .header("x-csrf-token", "token")
        .header("x-request-id", "req-0190")
        .header("authorization", "Bearer forged")
        .header("x-admin-id", "forged")
        .body(Body::empty())
        .unwrap();
    let (status, headers, body) = send(&router, request).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(json(&body)["items"], serde_json::json!([]));
    assert!(
        headers.get(SET_COOKIE).is_none(),
        "an upstream cookie never passes"
    );
    let received = only(&log);
    assert_eq!(received.method, Method::GET);
    assert_eq!(
        received.uri,
        "/internal/admin/v1/audit-log?limit=10&cursor=a%20b"
    );
    assert_relayed_headers(&received);
}

/// The headers the internal admin API receives: its own, the admin's, and a few of the browser's.
fn assert_relayed_headers(received: &Received) {
    let header = |name: &str| {
        received
            .headers
            .get(name)
            .map(|v| v.to_str().unwrap().to_owned())
    };
    assert_eq!(
        header("authorization"),
        Some(format!("Bearer {RELAY_SECRET}"))
    );
    assert_eq!(
        header("x-admin-id"),
        Some(admin().id.hyphenated().to_string())
    );
    assert_eq!(header("x-admin-email").as_deref(), Some(ADMIN_EMAIL));
    assert_eq!(header("x-request-id").as_deref(), Some("req-0190"));
    assert_eq!(header("accept").as_deref(), Some("application/json"));
    for absent in ["cookie", "x-csrf-token"] {
        assert_eq!(header(absent), None, "{absent} is not relayed");
    }
    assert_eq!(received.headers.get_all("x-admin-id").iter().count(), 1);
}

#[tokio::test]
async fn a_body_is_streamed_to_the_server() {
    let (router, log) = relay_to_fake().await;
    let chunks = ["{\"body\":", "\"thanks, ", "fixed\"}"]
        .map(|chunk| Ok::<_, std::io::Error>(Bytes::from_static(chunk.as_bytes())));
    let request = Request::post("/api/admin/v1/support-requests/0190/messages")
        .header("content-type", "application/json")
        .body(Body::from_stream(stream::iter(chunks)))
        .unwrap();
    let (status, _, _) = send(&router, request).await;
    assert_eq!(status, StatusCode::OK);
    let received = only(&log);
    assert_eq!(received.method, Method::POST);
    assert_eq!(
        received.uri,
        "/internal/admin/v1/support-requests/0190/messages"
    );
    assert_eq!(&received.body[..], b"{\"body\":\"thanks, fixed\"}");
    let content_type = received.headers.get(CONTENT_TYPE).unwrap();
    assert_eq!(content_type, "application/json");
}

#[tokio::test]
async fn a_download_streams_back_with_its_headers() {
    let (router, _) = relay_to_fake().await;
    let (status, headers, body) = send(&router, get("/api/admin/v1/users/0190/export")).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(headers.get(CONTENT_TYPE).unwrap(), "application/zip");
    let disposition = headers.get(CONTENT_DISPOSITION).unwrap();
    assert_eq!(disposition, "attachment; filename=\"export.zip\"");
    assert!(headers.get(SET_COOKIE).is_none());
    assert_eq!(body.len(), EXPORT_BYTES);
}

#[tokio::test]
async fn the_server_s_problems_pass_through() {
    let (router, _) = relay_to_fake().await;
    let (status, headers, body) = send(&router, get("/api/admin/v1/users/missing")).await;
    assert_eq!(status, StatusCode::NOT_FOUND);
    assert_eq!(
        headers.get(CONTENT_TYPE).unwrap(),
        "application/problem+json"
    );
    assert_eq!(json(&body)["code"], "admin.user_not_found");
}

#[tokio::test]
async fn a_refused_secret_is_an_internal_error_never_the_admin_s() {
    let (router, _) = relay_to_fake().await;
    let (status, _, body) = send(&router, get("/api/admin/v1/users/refused")).await;
    assert_eq!(status, StatusCode::INTERNAL_SERVER_ERROR);
    assert_eq!(json(&body)["code"], "internal.error");
}

#[tokio::test]
async fn a_server_that_does_not_answer_is_unavailable() {
    let env = common::local_env(&format!("{NOTHING_LISTENS}/internal/admin/v1"), None);
    let router = signed_in(relay::router(), common::state(&env));
    let (status, _, body) = send(&router, get("/api/admin/v1/users")).await;
    assert_eq!(status, StatusCode::SERVICE_UNAVAILABLE);
    assert_eq!(json(&body)["code"], "service.unavailable");
}

#[tokio::test]
async fn only_the_relayed_paths_reach_the_server() {
    let (router, log) = relay_to_fake().await;
    for path in [
        "/api/admin/v1/environment",
        "/api/admin/v1/users/..%2Fenvironment/../..",
        "/api/admin/v1/users/%2e%2e/environment",
        "/api/admin/v1/usersx",
    ] {
        let (status, _, _) = send(&router, get(path)).await;
        assert_eq!(status, StatusCode::NOT_FOUND, "{path}");
    }
    assert!(log.lock().unwrap().is_empty());
}
