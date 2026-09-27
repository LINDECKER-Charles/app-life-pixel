//! The built app: its files with their caching rules, and `index.html` for every other `GET`,
//! never under `/api/`, `/i18n/` or `/mcp`.

mod common;

use axum::http::Method;
use common::{APP_FILES, TestServer, empty, get_with};

const IMMUTABLE: &str = "public, max-age=31536000, immutable";
const INDEX: &str = "<!doctype html><title>Life Pixel</title>";

#[tokio::test]
async fn hashed_bundles_are_immutable_and_the_rest_revalidated() {
    let server = TestServer::new();
    let expected = [
        ("/main-7QX4ZK2M.js", IMMUTABLE, "text/javascript"),
        ("/styles-3HB6WN8R.css", IMMUTABLE, "text/css"),
        (
            "/engine/life_pixel_wasm-7QX4ZK2M.js",
            "no-cache",
            "text/javascript",
        ),
        ("/life-pixel.js", "no-cache", "text/javascript"),
        ("/media/icon.svg", "no-cache", "image/svg+xml"),
        ("/index.html", "no-cache", "text/html"),
    ];
    for (path, cache_control, media_type) in expected {
        let answer = server.get(path).await;
        assert_eq!(answer.status, 200, "{path}");
        assert_eq!(answer.header("cache-control"), cache_control, "{path}");
        assert!(
            answer.header("content-type").starts_with(media_type),
            "{path}"
        );
        assert!(answer.header("etag").starts_with('"'), "{path}");
    }
    let (_, content) = APP_FILES[1];
    assert_eq!(server.get("/main-7QX4ZK2M.js").await.body, content);
}

#[tokio::test]
async fn an_app_route_gets_the_index() {
    let server = TestServer::new();
    for path in ["/", "/editor", "/library/projects/42", "/main-0000AAAA.js"] {
        let answer = server.get(path).await;
        assert_eq!(answer.status, 200, "{path}");
        assert_eq!(answer.body, INDEX, "{path}");
        assert_eq!(answer.header("cache-control"), "no-cache");
    }
}

#[tokio::test]
async fn the_reserved_prefixes_never_get_the_index() {
    let server = TestServer::new();
    for path in [
        "/api",
        "/api/v1/editor",
        "/i18n",
        "/i18n/xx/yy.json",
        "/mcp/tools",
    ] {
        server
            .get(path)
            .await
            .assert_problem(404, "request.not_found");
    }
}

#[tokio::test]
async fn an_etag_match_is_not_modified_and_head_has_no_body() {
    let server = TestServer::new();
    let etag = server.get("/").await.header("etag").to_owned();
    let answer = server.send(get_with("/", "if-none-match", &etag)).await;
    assert_eq!(answer.status, 304);
    assert!(answer.body.is_empty());
    assert_eq!(answer.header("etag"), etag);
    let head = server.send(empty(Method::HEAD, "/")).await;
    assert_eq!(head.status, 200);
    assert!(head.body.is_empty());
    assert_eq!(head.header("content-length"), INDEX.len().to_string());
}

#[tokio::test]
async fn a_write_to_the_app_is_not_allowed() {
    let server = TestServer::new();
    let post = empty(Method::POST, "/editor");
    let answer = server.send(post).await;
    answer.assert_problem(405, "request.method_not_allowed");
    assert_eq!(answer.header("allow"), "GET, HEAD");
}

#[tokio::test]
async fn without_a_built_app_only_the_api_answers() {
    let server = TestServer::with(
        |env| {
            env.insert(
                "LP_APP_DIR".to_owned(),
                "/nonexistent/life-pixel".to_owned(),
            );
        },
        true,
    );
    server
        .get("/")
        .await
        .assert_problem(404, "request.not_found");
    assert_eq!(server.get("/healthz").await.status, 200);
}
