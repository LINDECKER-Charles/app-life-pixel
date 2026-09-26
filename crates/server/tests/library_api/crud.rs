//! Each library route doing its work: projects listed by page, renamed and deleted; animations
//! imported, searched, moved and deleted; duplicates.

use axum::http::Method;
use serde_json::{Value, json};

use crate::stack::{ApiStack, delete, get, import, sample};
use crate::{id, json_call, titles};

#[tokio::test]
async fn projects_are_created_listed_by_page_renamed_and_deleted() {
    let stack = ApiStack::new().await;
    let ada = stack.sign_up("ada@example.org").await;
    let sprites = stack.project(&ada, "Sprites").await;
    stack.project(&ada, "Tiles").await;

    let first = stack
        .expect(200, get(&ada, "/projects?limit=1"))
        .await
        .json();
    assert_eq!(first["items"][0]["name"], "Tiles");
    let cursor = first["nextCursor"].as_str().unwrap();
    let second = stack
        .expect(
            200,
            get(&ada, &format!("/projects?limit=1&cursor={cursor}")),
        )
        .await;
    assert_eq!(second.json()["items"][0]["name"], "Sprites");
    assert_eq!(second.json()["nextCursor"], Value::Null);
    let path = format!("/projects/{sprites}");
    let read = stack.expect(200, get(&ada, &path)).await.json();
    assert_eq!(
        (read["name"].clone(), read["animationCount"].clone()),
        (json!("Sprites"), json!(0))
    );
    let renamed = json_call(&ada, (Method::PATCH, &path), &json!({ "name": "Walks" }));
    assert_eq!(stack.expect(200, renamed).await.json()["name"], "Walks");
    stack.expect(204, delete(&ada, &path)).await;

    let gone = stack.send(get(&ada, &path)).await;
    gone.assert_problem(404, "library.project_not_found");
}

#[tokio::test]
async fn animations_are_imported_listed_and_searched() {
    let stack = ApiStack::new().await;
    let ada = stack.sign_up("ada@example.org").await;
    let sprites = stack.project(&ada, "Sprites").await;
    let tiles = stack.project(&ada, "Tiles").await;

    let walk = stack.created(import(&ada, &sprites, &sample("Walk"))).await;
    stack.created(import(&ada, &sprites, &sample("Run"))).await;

    assert_eq!(walk["projectId"], sprites.as_str());
    assert_eq!(walk["version"], 1);
    assert_eq!(titles(&stack, &ada, "").await, ["Run", "Walk"]);
    assert_eq!(titles(&stack, &ada, "?q=wal").await, ["Walk"]);
    let in_tiles = titles(&stack, &ada, &format!("?project={tiles}")).await;
    assert_eq!(in_tiles, Vec::<String>::new());
}

#[tokio::test]
async fn an_animation_is_moved_read_and_deleted() {
    let stack = ApiStack::new().await;
    let ada = stack.sign_up("ada@example.org").await;
    let sprites = stack.project(&ada, "Sprites").await;
    let tiles = stack.project(&ada, "Tiles").await;
    let walk = stack.created(import(&ada, &sprites, &sample("Walk"))).await;
    let path = format!("/animations/{}", id(&walk));

    let moved = json_call(&ada, (Method::PATCH, &path), &json!({ "projectId": tiles }));
    let moved = stack.expect(200, moved).await.json();
    assert_eq!(moved["projectId"], tiles.as_str());
    let in_tiles = titles(&stack, &ada, &format!("?project={tiles}")).await;
    assert_eq!(in_tiles, ["Walk"]);
    let unchanged = json_call(&ada, (Method::PATCH, &path), &json!({}));
    assert_eq!(stack.expect(200, unchanged).await.json(), moved);
    assert_eq!(stack.expect(200, get(&ada, &path)).await.json(), moved);
    stack.expect(204, delete(&ada, &path)).await;

    let gone = stack.send(get(&ada, &path)).await;
    gone.assert_problem(404, "library.animation_not_found");
    assert_eq!(titles(&stack, &ada, "").await, Vec::<String>::new());
}

#[tokio::test]
async fn duplicates_copy_a_project_with_its_animations_or_one_animation() {
    let stack = ApiStack::new().await;
    let ada = stack.sign_up("ada@example.org").await;
    let sprites = stack.project(&ada, "Sprites").await;
    let walk = id(&stack.created(import(&ada, &sprites, &sample("Walk"))).await);

    let copy = json_call(
        &ada,
        (Method::POST, &format!("/projects/{sprites}/duplicate")),
        &json!({ "name": "Copy" }),
    );
    let copy = stack.created(copy).await;
    assert_eq!(
        (copy["name"].clone(), copy["animationCount"].clone()),
        (json!("Copy"), json!(1))
    );
    let path = format!("/animations/{walk}/duplicate");
    let beside = stack
        .created(json_call(
            &ada,
            (Method::POST, &path),
            &json!({ "title": "Walk 2" }),
        ))
        .await;
    assert_eq!(beside["projectId"], sprites.as_str());
    assert_ne!(id(&beside), walk);
    let moved = json!({ "title": "Walk 3", "projectId": id(&copy) });
    let into = stack
        .created(json_call(&ada, (Method::POST, &path), &moved))
        .await;
    assert_eq!(into["projectId"], copy["id"]);

    assert_eq!(titles(&stack, &ada, "?q=walk").await.len(), 4);
}
