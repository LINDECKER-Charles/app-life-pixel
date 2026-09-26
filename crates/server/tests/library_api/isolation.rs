//! One account seeing and touching nothing of another's library: each route answers as if the
//! project or the animation did not exist.

use axum::body::Body;
use axum::http::{Method, Request};
use serde_json::json;

use crate::router::auth::{Browser, with_json};
use crate::stack::{ApiStack, api, delete, get, if_match, import, sample, with_document};
use crate::{id, json_call, titles};

/// Every request of `bob` on the project `project`, which is not theirs.
fn on_project(bob: &Browser, project: &str) -> Vec<Request<Body>> {
    let path = format!("/projects/{project}");
    let name = json!({ "name": "Mine" });
    let duplicate = (Method::POST, format!("{path}/duplicate"));
    vec![
        get(bob, &path),
        json_call(bob, (Method::PATCH, &path), &name),
        json_call(bob, (duplicate.0, &duplicate.1), &name),
        import(bob, project, &sample("Walk")),
        delete(bob, &path),
    ]
}

/// Every request of `bob` on the animation `animation`, which is not theirs.
fn on_animation(bob: &Browser, animation: &str) -> Vec<Request<Body>> {
    let path = format!("/animations/{animation}");
    let (document, duplicate) = (format!("{path}/document"), format!("{path}/duplicate"));
    let (title, retitle) = (
        json!({ "title": "Mine" }),
        if_match(api(bob, Method::PATCH, &path), 1),
    );
    let save = if_match(api(bob, Method::PUT, &document), 1);
    vec![
        get(bob, &path),
        with_json(retitle, &title),
        json_call(bob, (Method::POST, &duplicate), &title),
        get(bob, &document),
        with_document(save, &sample("Run")),
        delete(bob, &path),
    ]
}

#[tokio::test]
async fn an_account_sees_and_touches_nothing_of_another() {
    let stack = ApiStack::new().await;
    let ada = stack.sign_up("ada@example.org").await;
    let bob = stack.sign_up("bob@example.org").await;
    let sprites = stack.project(&ada, "Sprites").await;
    let walk = id(&stack.created(import(&ada, &sprites, &sample("Walk"))).await);

    for request in on_project(&bob, &sprites) {
        stack
            .refused(request, (404, "library.project_not_found"))
            .await;
    }
    for request in on_animation(&bob, &walk) {
        stack
            .refused(request, (404, "library.animation_not_found"))
            .await;
    }

    let listed = stack.expect(200, get(&bob, "/projects")).await.json();
    assert_eq!(listed["items"], json!([]));
    assert_eq!(titles(&stack, &bob, "").await, Vec::<String>::new());
    assert_eq!(titles(&stack, &ada, "").await, ["Walk"]);
    let path = format!("/animations/{walk}/document");
    assert_eq!(
        stack.expect(200, get(&ada, &path)).await.body,
        sample("Walk")
    );
}

#[tokio::test]
async fn an_animation_goes_to_no_project_of_another_account() {
    let stack = ApiStack::new().await;
    let ada = stack.sign_up("ada@example.org").await;
    let bob = stack.sign_up("bob@example.org").await;
    let sprites = stack.project(&ada, "Sprites").await;
    let tiles = stack.project(&bob, "Tiles").await;
    let run = id(&stack.created(import(&bob, &tiles, &sample("Run"))).await);
    let (path, into_ada) = (
        format!("/animations/{run}"),
        json!({ "projectId": sprites }),
    );
    let copy = json!({ "projectId": sprites, "title": "Mine" });

    let moved = json_call(&bob, (Method::PATCH, &path), &into_ada);
    let copied = json_call(&bob, (Method::POST, &format!("{path}/duplicate")), &copy);

    let missing = (404, "library.project_not_found");
    stack.refused(moved, missing).await;
    stack.refused(copied, missing).await;
    assert_eq!(titles(&stack, &ada, "").await, Vec::<String>::new());
}
