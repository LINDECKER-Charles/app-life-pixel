//! The version of a document: read as its ETag, and required by `If-Match` on each save.

use axum::body::Bytes;
use axum::http::Method;
use axum::http::request::Builder;
use life_pixel_server::routes::library::DOCUMENT_MEDIA_TYPE;
use serde_json::json;

use crate::router::auth::{Browser, with_json};
use crate::stack::{ApiStack, api, get, if_match, import, sample, with_document};
use crate::{id, json_call};

/// The path of the document of a new animation of `browser`, imported from `document`.
async fn document_path(stack: &ApiStack, browser: &Browser, document: &Bytes) -> String {
    let sprites = stack.project(browser, "Sprites").await;
    let animation = stack.created(import(browser, &sprites, document)).await;
    format!("/animations/{}/document", id(&animation))
}

/// `PUT path` from the app, for `browser`.
fn put(browser: &Browser, path: &str) -> Builder {
    api(browser, Method::PUT, path)
}

#[tokio::test]
async fn a_document_is_read_and_saved_under_its_version() {
    let stack = ApiStack::new().await;
    let ada = stack.sign_up("ada@example.org").await;
    let (walk, run) = (sample("Walk"), sample("Run"));
    let path = document_path(&stack, &ada, &walk).await;

    let read = stack.expect(200, get(&ada, &path)).await;
    assert_eq!(read.header("content-type"), DOCUMENT_MEDIA_TYPE);
    assert_eq!((read.header("etag"), &read.body), ("\"1\"", &walk));
    let saved = with_document(if_match(put(&ada, &path), 1), &run);
    let saved = stack.expect(200, saved).await;

    assert_eq!(saved.header("etag"), "\"2\"");
    let (title, version) = (
        saved.json()["title"].clone(),
        saved.json()["version"].clone(),
    );
    assert_eq!((title, version), (json!("Run"), json!(2)));
    let read = stack.expect(200, get(&ada, &path)).await;
    assert_eq!((read.header("etag"), &read.body), ("\"2\"", &run));
}

#[tokio::test]
async fn a_save_without_the_current_version_is_refused() {
    let stack = ApiStack::new().await;
    let ada = stack.sign_up("ada@example.org").await;
    let (walk, run) = (sample("Walk"), sample("Run"));
    let path = document_path(&stack, &ada, &walk).await;
    stack
        .expect(200, with_document(if_match(put(&ada, &path), 1), &run))
        .await;

    let stale = stack
        .send(with_document(if_match(put(&ada, &path), 1), &walk))
        .await;
    let params = stale.assert_problem(412, "document.version_conflict");
    assert_eq!(params, json!({ "current": 2 }));
    let blind = stack.send(with_document(put(&ada, &path), &walk)).await;
    blind.assert_problem(428, "document.version_required");
    let weak = put(&ada, &path).header("if-match", "W/\"2\"");
    let weak = stack.send(with_document(weak, &walk)).await;
    weak.assert_problem(400, "request.malformed");

    assert_eq!(stack.expect(200, get(&ada, &path)).await.body, run);
}

#[tokio::test]
async fn a_title_changes_under_the_version_it_replaces() {
    let stack = ApiStack::new().await;
    let ada = stack.sign_up("ada@example.org").await;
    let sprites = stack.project(&ada, "Sprites").await;
    let walk = stack.created(import(&ada, &sprites, &sample("Walk"))).await;
    let path = format!("/animations/{}", id(&walk));
    let title = json!({ "title": "Jump" });

    let blind = stack
        .send(json_call(&ada, (Method::PATCH, &path), &title))
        .await;
    blind.assert_problem(428, "document.version_required");
    let stale = with_json(if_match(api(&ada, Method::PATCH, &path), 7), &title);
    stack
        .send(stale)
        .await
        .assert_problem(412, "document.version_conflict");
    let renamed = with_json(if_match(api(&ada, Method::PATCH, &path), 1), &title);
    let renamed = stack.expect(200, renamed).await.json();
    assert_eq!(
        (renamed["title"].clone(), renamed["version"].clone()),
        (json!("Jump"), json!(2))
    );
    let long = with_json(
        if_match(api(&ada, Method::PATCH, &path), 2),
        &json!({ "title": "x".repeat(200) }),
    );
    stack.send(long).await.assert_problem(422, "document.name");
}
