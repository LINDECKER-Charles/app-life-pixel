//! The public router's outer layers: the security headers on every response, and the request
//! id, kept from a trusted proxy only.

mod common;

use std::net::SocketAddr;

use common::{TestServer, get_with, send};
use uuid::Uuid;

const CSP: &str = "default-src 'self'; script-src 'self' 'wasm-unsafe-eval'; \
style-src 'self' 'unsafe-inline'; img-src 'self' data: blob:; connect-src 'self' blob:; \
worker-src 'self'; font-src 'self'; object-src 'none'; base-uri 'self'; form-action 'self'; \
frame-ancestors 'none'";

/// An app page, an i18n file, the health check, an API error and an app error.
const PATHS: [&str; 5] = [
    "/",
    "/i18n/en.json",
    "/healthz",
    "/api/v1/nothing",
    "/i18n/../../Cargo.toml",
];

fn proxy_peer() -> SocketAddr {
    SocketAddr::from(([10, 0, 0, 2], 40_000))
}

#[tokio::test]
async fn every_response_carries_the_security_headers() {
    let server = TestServer::new();
    for path in PATHS {
        let answer = server.get(path).await;
        assert_eq!(answer.header("content-security-policy"), CSP, "{path}");
        assert_eq!(answer.header("x-content-type-options"), "nosniff");
        assert_eq!(answer.header("referrer-policy"), "no-referrer");
        let permissions = "camera=(), microphone=(), geolocation=(), payment=()";
        assert_eq!(answer.header("permissions-policy"), permissions);
        assert_eq!(answer.header("cross-origin-opener-policy"), "same-origin");
        assert!(!answer.headers.contains_key("strict-transport-security"));
    }
}

#[tokio::test]
async fn hsts_comes_with_an_https_origin() {
    let server = TestServer::with(
        |env| {
            env.insert(
                "LP_PUBLIC_URL".to_owned(),
                "https://life-pixel.app".to_owned(),
            );
        },
        true,
    );
    for path in PATHS {
        let answer = server.get(path).await;
        let hsts = answer.header("strict-transport-security");
        assert_eq!(hsts, "max-age=63072000; includeSubDomains", "{path}");
    }
}

#[tokio::test]
async fn a_request_without_an_id_gets_a_uuid_v7() {
    let answer = TestServer::new().get("/healthz").await;
    let id = Uuid::parse_str(answer.header("x-request-id")).unwrap();
    assert_eq!(id.get_version_num(), 7);
}

#[tokio::test]
async fn an_id_is_kept_from_a_trusted_proxy_only() {
    let server = TestServer::with(
        |env| {
            env.insert("LP_TRUSTED_PROXIES".to_owned(), "10.0.0.0/8".to_owned());
        },
        true,
    );
    let with_id = |id: &str| get_with("/healthz", "x-request-id", id);
    let trusted = send(server.public_router(proxy_peer()), with_id("edge-42")).await;
    assert_eq!(trusted.header("x-request-id"), "edge-42");
    let untrusted = server.send(with_id("edge-42")).await;
    assert_ne!(untrusted.header("x-request-id"), "edge-42");
    let forged = send(server.public_router(proxy_peer()), with_id("a b")).await;
    assert_ne!(forged.header("x-request-id"), "a b");
}
