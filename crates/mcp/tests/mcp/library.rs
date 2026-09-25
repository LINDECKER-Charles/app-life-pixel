//! `list_animations` and `create_animation`.

use life_pixel_core::limits::{CANVAS_MAX_SIDE, CANVAS_MIN_SIDE, MCP_PAGE_SIZE_DEFAULT};
use serde_json::{Value, json};

use super::session::Session;

/// The ids of a listing's animations.
fn ids(list: &Value) -> Vec<String> {
    let animations = list["animations"].as_array().unwrap();
    animations
        .iter()
        .map(|animation| animation["id"].as_str().unwrap().to_owned())
        .collect()
}

#[tokio::test]
async fn an_animation_is_created_blank_in_a_project_found_or_created_by_name() {
    let session = Session::start().await;

    let first = session.create(16, 8).await;
    let second = session.create(4, 4).await;

    assert_eq!(first["title"], "Mascot");
    assert_eq!(
        (first["width"].clone(), first["height"].clone()),
        (json!(16), json!(8))
    );
    assert_eq!(
        first["frames"],
        json!([{ "position": 0, "duration_ms": 100 }])
    );
    assert_eq!(first["layers"][0]["name"], "Layer 1");
    assert_eq!(first["project_id"], second["project_id"]);
    let arguments = json!({
        "title": "Ghost", "width": 2, "height": 2, "project_id": first["project_id"],
        "palette": ["#00000000", "#ff0000ff"], "frame_duration_ms": 250, "layer_name": "Ink",
    });
    let third = session.ok("create_animation", arguments).await;
    assert_eq!(third["project_id"], first["project_id"]);
    assert_eq!(third["palette"], json!(["#00000000", "#ff0000ff"]));
    assert_eq!(third["frames"][0]["duration_ms"], 250);
    assert_eq!(third["layers"][0]["name"], "Ink");
}

/// The arguments of a 4 × 4 "Mascot", with `extra`.
fn creation(extra: Value) -> Value {
    let mut arguments = json!({ "title": "Mascot", "width": 4, "height": 4 });
    let extra = extra.as_object().unwrap().clone();
    arguments.as_object_mut().unwrap().extend(extra);
    arguments
}

#[tokio::test]
async fn a_creation_needs_exactly_one_project() {
    let session = Session::start().await;
    let nil = uuid::Uuid::nil();

    let both = creation(json!({ "project_name": "Pets", "project_id": nil }));
    let neither = creation(json!({}));
    let missing = creation(json!({ "project_id": nil }));

    assert_eq!(
        session.code("create_animation", both).await,
        "request.malformed"
    );
    assert_eq!(
        session.code("create_animation", neither).await,
        "request.malformed"
    );
    let code = session.code("create_animation", missing).await;
    assert_eq!(code, "library.project_not_found");
}

#[tokio::test]
async fn a_creation_needs_a_valid_document() {
    let session = Session::start().await;

    let too_wide = creation(json!({ "project_name": "Pets", "width": CANVAS_MAX_SIDE + 1 }));
    let (code, params) = session.fails("create_animation", too_wide).await;
    let opaque_zero = creation(json!({ "project_name": "Pets", "palette": ["#ffffffff"] }));
    let unknown = creation(json!({ "project_name": "Pets", "colour": "red" }));

    assert_eq!(code, "document.canvas_size");
    assert_eq!(
        params,
        json!({ "min": CANVAS_MIN_SIDE, "max": CANVAS_MAX_SIDE })
    );
    assert_eq!(
        session.code("create_animation", opaque_zero).await,
        "document.palette"
    );
    assert_eq!(
        session.code("create_animation", unknown).await,
        "request.malformed"
    );
}

#[tokio::test]
async fn a_creation_beyond_the_quota_is_refused_with_its_numbers() {
    let session = Session::with_quota(10).await;

    let arguments = creation(json!({ "project_name": "Pets" }));
    let (code, params) = session.fails("create_animation", arguments).await;

    assert_eq!(code, "quota.storage_exceeded");
    assert_eq!(params["limit"], 10);
    assert_eq!(params["used"], 0);
}

#[tokio::test]
async fn a_listing_pages_by_cursor_without_gap_or_duplicate() {
    let session = Session::start().await;
    let mut created = Vec::new();
    for _ in 0..5 {
        created.push(
            session.create(2, 2).await["id"]
                .as_str()
                .unwrap()
                .to_owned(),
        );
    }

    let mut seen = Vec::new();
    let mut cursor = Value::Null;
    loop {
        let page = session
            .ok("list_animations", json!({ "limit": 2, "cursor": cursor }))
            .await;
        assert!(ids(&page).len() <= 2);
        seen.extend(ids(&page));
        cursor = page["next_cursor"].clone();
        if cursor.is_null() {
            break;
        }
    }

    created.reverse();
    assert_eq!(seen, created);
}

#[tokio::test]
async fn a_listing_holds_the_default_page_and_summaries() {
    let session = Session::start().await;
    for _ in 0..=MCP_PAGE_SIZE_DEFAULT {
        session.create(3, 2).await;
    }

    let page = session.ok("list_animations", json!({})).await;

    assert_eq!(ids(&page).len(), MCP_PAGE_SIZE_DEFAULT);
    assert!(page["next_cursor"].is_string());
    let summary = &page["animations"][0];
    let keys: Vec<&String> = summary.as_object().unwrap().keys().collect();
    let expected = [
        "frame_count",
        "height",
        "id",
        "project_id",
        "title",
        "updated_at",
        "width",
    ];
    assert_eq!(keys, expected);
    assert_eq!(summary["updated_at"], "2026-09-01T12:00:00Z");
}

#[tokio::test]
async fn a_listing_filters_by_title_and_project() {
    let session = Session::start().await;
    let mascot = session.create(2, 2).await;
    let arguments = json!({ "title": "Tree", "width": 2, "height": 2, "project_name": "Plants" });
    let tree = session.ok("create_animation", arguments).await;

    let found = session
        .ok("list_animations", json!({ "query": "TRE" }))
        .await;
    let in_pets = session
        .ok(
            "list_animations",
            json!({ "project_id": mascot["project_id"] }),
        )
        .await;

    assert_eq!(ids(&found), [tree["id"].as_str().unwrap()]);
    assert_eq!(ids(&in_pets), [mascot["id"].as_str().unwrap()]);
}

#[tokio::test]
async fn a_cursor_that_does_not_decode_is_malformed() {
    let session = Session::start().await;

    let code = session
        .code("list_animations", json!({ "cursor": "nope" }))
        .await;

    assert_eq!(code, "request.malformed");
}
