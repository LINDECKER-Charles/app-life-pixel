//! What the library routes refuse: a request breaking a rule of the API or of the model, an
//! unknown id, and a request without a session or its token.

use axum::body::Bytes;
use axum::http::Method;
use life_pixel_core::limits::MAX_DOCUMENT_BYTES;
use serde_json::json;

use crate::router::auth::{from_app, with_json};
use crate::router::empty;
use crate::stack::{API, ApiStack, api, get, import, sample};
use crate::{UNKNOWN, json_call};

#[tokio::test]
async fn a_body_that_is_not_a_document_is_refused() {
    let stack = ApiStack::new().await;
    let ada = stack.sign_up("ada@example.org").await;
    let sprites = stack.project(&ada, "Sprites").await;
    let into = format!("/projects/{sprites}/animations");

    let as_json = with_json(api(&ada, Method::POST, &into), &json!({}));
    let unsupported = (415, "request.unsupported_media_type");
    stack.refused(as_json, unsupported).await;
    let huge = Bytes::from(vec![b' '; MAX_DOCUMENT_BYTES + 1]);
    let huge = import(&ada, &sprites, &huge);
    let params = stack.refused(huge, (413, "document.too_large")).await;
    assert_eq!(params["maxBytes"], MAX_DOCUMENT_BYTES);
    let empty_object = Bytes::from_static(b"{}");
    let invalid = stack.send(import(&ada, &sprites, &empty_object)).await;
    assert_eq!(invalid.status, 422, "{:?}", invalid.body);
    let code = invalid.json()["code"].as_str().unwrap().to_owned();
    assert!(code.starts_with("document."), "{code}");
}

#[tokio::test]
async fn a_malformed_request_or_an_unknown_id_is_refused() {
    let stack = ApiStack::new().await;
    let ada = stack.sign_up("ada@example.org").await;
    let malformed = [
        "/projects?cursor=nope",
        "/animations/not-an-id",
        "/animations?limit=many",
    ];

    for path in malformed {
        stack
            .refused(get(&ada, path), (400, "request.malformed"))
            .await;
    }
    let unnamed = json_call(&ada, (Method::POST, "/projects"), &json!({ "name": "" }));
    stack.refused(unnamed, (422, "document.name")).await;
    let missing = import(&ada, UNKNOWN, &sample("Walk"));
    stack
        .refused(missing, (404, "library.project_not_found"))
        .await;
    let missing = get(&ada, &format!("/animations/{UNKNOWN}/document"));
    stack
        .refused(missing, (404, "library.animation_not_found"))
        .await;
}

#[tokio::test]
async fn the_routes_need_a_session_and_a_change_its_token() {
    let stack = ApiStack::new().await;
    let ada = stack.sign_up("ada@example.org").await;
    let paths = ["/projects", "/animations", &format!("/projects/{UNKNOWN}")];
    let (projects, name) = (format!("{API}/projects"), json!({ "name": "Sprites" }));
    let unauthenticated = (401, "auth.unauthenticated");

    for path in paths {
        let anonymous = empty(Method::GET, &format!("{API}{path}"));
        stack.refused(anonymous, unauthenticated).await;
    }
    let tokenless = with_json(ada.cookie(from_app(Method::POST, &projects)), &name);
    stack.refused(tokenless, (403, "auth.csrf")).await;
    let anonymous = with_json(from_app(Method::POST, &projects), &name);
    stack.refused(anonymous, unauthenticated).await;
}
