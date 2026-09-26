//! The signed links of `export`: each file compiled again from the version the link names and
//! sent as an attachment, until the link expires or the animation changes; never with a link
//! tampered with.

use axum::body::Body;
use axum::http::{Method, Request};
use serde_json::{Value, json};
use time::Duration;

use crate::router::Answer;
use crate::router::auth::{Browser, with_json};
use crate::stack::{ApiStack, api, delete, if_match};
use crate::tokens::{secret, tool_call, tool_output};
use crate::{animation, clock_at, stack_at};

/// The origin of the links: `LP_PUBLIC_URL`.
const PUBLIC_URL: &str = "http://localhost:8460";

/// The files `export` answers for `animation` in `format`, with the token `secret`.
async fn exported(stack: &ApiStack, secret: &str, (animation, format): (&Value, &str)) -> Value {
    let arguments = json!({ "id": animation["id"], "format": format });
    let answer = stack.send(tool_call(secret, ("export", arguments))).await;
    tool_output(&answer)
}

/// The download of the link `url`, without a session.
async fn download(stack: &ApiStack, url: &str) -> Answer {
    let path = url.strip_prefix(PUBLIC_URL).unwrap();
    let request = Request::get(path).body(Body::empty()).unwrap();
    stack.send(request).await
}

/// A user with an animation and a token granting `read` and `export`.
async fn setup(stack: &ApiStack) -> (Browser, Value, String) {
    let ada = stack.sign_up("ada@example.org").await;
    let mascot = animation(stack, &ada, "Mascot").await;
    let secret = secret(stack, &ada, &["read", "export"]).await;
    (ada, mascot, secret)
}

/// The URL of the first file of `export`.
fn first_url(export: &Value) -> String {
    export["files"][0]["url"].as_str().unwrap().to_owned()
}

#[tokio::test]
async fn a_link_downloads_its_file_as_an_attachment_until_it_expires() {
    let clock = clock_at(time::OffsetDateTime::now_utc());
    let stack = stack_at(&clock, |_| {}).await;
    let (_, mascot, secret) = setup(&stack).await;

    let export = exported(&stack, &secret, (&mascot, "gif")).await;

    let file = &export["files"][0];
    assert_eq!(file["name"], "mascot.gif");
    assert_eq!(file["media_type"], "image/gif");
    let url = first_url(&export);
    assert!(
        url.starts_with("http://localhost:8460/api/v1/exports/"),
        "{url}"
    );
    assert!(export["expires_at"].is_string());
    for _ in 0..2 {
        let answer = download(&stack, &url).await;
        assert_eq!(answer.status.as_u16(), 200, "{:?}", answer.body);
        assert_eq!(answer.header("content-type"), "image/gif");
        let disposition = answer.header("content-disposition");
        assert_eq!(disposition, "attachment; filename=\"mascot.gif\"");
        assert_eq!(answer.header("cache-control"), "no-store");
        assert_eq!(answer.body.len() as u64, file["bytes"].as_u64().unwrap());
        assert!(answer.body.starts_with(b"GIF89a"));
    }
    clock.advance(Duration::minutes(15));
    let expired = download(&stack, &url).await;
    expired.assert_problem(410, "export.link_invalid");
}

#[tokio::test]
async fn every_file_of_a_web_assembly_export_has_its_link() {
    let stack = ApiStack::new().await;
    let (_, mascot, secret) = setup(&stack).await;

    let export = exported(&stack, &secret, (&mascot, "wasm")).await;

    let files = export["files"].as_array().unwrap();
    let names: Vec<&str> = files
        .iter()
        .map(|file| file["name"].as_str().unwrap())
        .collect();
    assert_eq!(names, ["mascot.wasm", "life-pixel.js"]);
    for file in files {
        let answer = download(&stack, file["url"].as_str().unwrap()).await;
        assert_eq!(answer.status.as_u16(), 200);
        assert_eq!(answer.header("content-type"), file["media_type"]);
    }
}

#[tokio::test]
async fn a_link_tampered_with_is_invalid() {
    let stack = ApiStack::new().await;
    let (_, mascot, secret) = setup(&stack).await;
    let url = first_url(&exported(&stack, &secret, (&mascot, "gif")).await);
    let (payload, signature) = url.rsplit_once('.').unwrap();
    let other = first_url(&exported(&stack, &secret, (&mascot, "apng")).await);
    let (other_payload, _) = other.rsplit_once('.').unwrap();
    let flipped = if signature.starts_with('A') { "B" } else { "A" };
    let tampered = [
        format!("{payload}.{flipped}{}", &signature[1..]),
        format!("{other_payload}.{signature}"),
        format!("{payload}{signature}"),
        format!("{PUBLIC_URL}/api/v1/exports/not-a-link"),
    ];

    for url in tampered {
        let answer = download(&stack, &url).await;
        answer.assert_problem(410, "export.link_invalid");
    }
}

#[tokio::test]
async fn a_link_to_an_animation_changed_or_deleted_since_is_invalid() {
    let stack = ApiStack::new().await;
    let (ada, mascot, secret) = setup(&stack).await;
    let id = mascot["id"].as_str().unwrap();
    let before = first_url(&exported(&stack, &secret, (&mascot, "gif")).await);
    let path = format!("/animations/{id}");
    let version = mascot["version"].as_u64().unwrap();
    let rename = if_match(api(&ada, Method::PATCH, &path), version);
    stack
        .expect(200, with_json(rename, &json!({ "title": "Hero" })))
        .await;

    download(&stack, &before)
        .await
        .assert_problem(410, "export.link_invalid");
    let after = first_url(&exported(&stack, &secret, (&mascot, "gif")).await);
    assert_eq!(download(&stack, &after).await.status.as_u16(), 200);
    stack.expect(204, delete(&ada, &path)).await;
    download(&stack, &after)
        .await
        .assert_problem(410, "export.link_invalid");
}
