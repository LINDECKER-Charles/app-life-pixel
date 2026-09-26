//! Who may call the internal admin API: the admin server alone, with the secret as a bearer
//! token and the acting admin's identity; anything else is `admin.unauthenticated`, unknown
//! routes included. And `GET /environment`.

use axum::http::Method;
use axum::http::header::AUTHORIZATION;

use crate::router::SECRET;
use crate::router::auth::without_body;
use crate::stack::ApiStack;
use crate::{ADMIN, ADMIN_EMAIL, ADMIN_ID, acting, admin_get, call, expect};

/// Another secret of the right shape.
const OTHER_SECRET: &str = "6f0c3a1e9b7d2468ace013579bdf2468ace013579bdf2468ace013579bdf2468";

#[tokio::test]
async fn the_secret_and_the_identity_headers_are_required() {
    let stack = ApiStack::new().await;
    let path = format!("{ADMIN}/environment");
    let get = || crate::router::request(Method::GET, &path);
    let refused = [
        get(),
        get().header(AUTHORIZATION, format!("Bearer {OTHER_SECRET}")),
        get().header(AUTHORIZATION, format!("Basic {SECRET}")),
        get().header(AUTHORIZATION, SECRET),
        get()
            .header(AUTHORIZATION, format!("Bearer {SECRET}"))
            .header("X-Admin-Email", ADMIN_EMAIL),
        get()
            .header(AUTHORIZATION, format!("Bearer {SECRET}"))
            .header("X-Admin-Id", ADMIN_ID),
        acting(("", ADMIN_EMAIL), Method::GET, "/environment"),
        acting((ADMIN_ID, "not an address"), Method::GET, "/environment"),
        acting((&"a".repeat(101), ADMIN_EMAIL), Method::GET, "/environment"),
        get()
            .header(AUTHORIZATION, format!("Bearer {OTHER_SECRET}"))
            .header("X-Admin-Id", ADMIN_ID)
            .header("X-Admin-Email", ADMIN_EMAIL),
    ];
    for (n, builder) in refused.into_iter().enumerate() {
        let answer = call(&stack, without_body(builder)).await;
        let params = answer.assert_problem(401, "admin.unauthenticated");
        assert!(
            params.as_object().is_none_or(serde_json::Map::is_empty),
            "{n}"
        );
    }
}

#[tokio::test]
async fn the_environment_names_the_server_and_its_version() {
    let stack = ApiStack::new().await;
    let answer = expect(&stack, 200, admin_get("/environment")).await.json();
    assert_eq!(answer["environment"], "local");
    assert_eq!(answer["version"], env!("CARGO_PKG_VERSION"));
}

#[tokio::test]
async fn an_unknown_route_is_refused_without_the_secret_and_not_found_with_it() {
    let stack = ApiStack::new().await;
    let without = crate::router::request(Method::GET, &format!("{ADMIN}/nothing"));
    let without = call(&stack, without_body(without)).await;
    without.assert_problem(401, "admin.unauthenticated");
    let with = call(&stack, admin_get("/nothing")).await;
    with.assert_problem(404, "request.not_found");
    let outside = call(
        &stack,
        without_body(crate::router::request(Method::GET, "/users")),
    )
    .await;
    outside.assert_problem(404, "request.not_found");
}

#[tokio::test]
async fn the_public_listener_never_serves_the_admin_api() {
    let stack = ApiStack::new().await;
    let answer = stack.send(admin_get("/environment")).await;
    assert!(answer.header("content-type").starts_with("text/html"));
    assert!(!String::from_utf8_lossy(&answer.body).contains(env!("CARGO_PKG_VERSION")));
}
