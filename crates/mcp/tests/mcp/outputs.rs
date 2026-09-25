//! `export` through a delivery double, and `get_embed_snippet`.

use life_pixel_mcp::ExportFormat;
use life_pixel_service::AnimationId;
use serde_json::{Value, json};
use uuid::Uuid;

use super::session::{ACCOUNT, Received, Session};

/// An animation with a tag `blink`: its id.
async fn tagged(session: &Session) -> Value {
    let id = session.create(4, 4).await["id"].clone();
    let edits = json!({ "id": id, "edits": [{ "op": "add" }] });
    session.ok("edit_frames", edits).await;
    let blink = json!({ "name": "blink", "first": 1, "last": 1, "loop": "once" });
    session
        .ok("set_tags", json!({ "id": id, "tags": [blink] }))
        .await;
    id
}

#[tokio::test]
async fn a_wasm_export_hands_the_module_and_its_loader_to_the_delivery() {
    let session = Session::start().await;
    let id = tagged(&session).await;
    let version = session.ok("get_animation", json!({ "id": id })).await["version"].clone();

    let arguments = json!({ "id": id, "format": "wasm", "tag": "blink", "destination": "out" });
    let result = session.ok("export", arguments).await;

    assert_eq!(
        result,
        json!({ "delivered": ["mascot.wasm", "life-pixel.js"] })
    );
    let calls = session.delivery.calls.lock().unwrap();
    let [(call, files)] = calls.as_slice() else {
        panic!("one delivery expected");
    };
    let id = Uuid::parse_str(id.as_str().unwrap()).unwrap();
    assert_eq!(call.id, AnimationId::from_uuid(id));
    assert_eq!(json!(call.version), version);
    assert_eq!(
        (call.format, call.tag.as_deref(), call.scale),
        (ExportFormat::Wasm, Some("blink"), None)
    );
    assert_eq!(
        call.options,
        json!({ "destination": "out" }).as_object().unwrap().clone()
    );
    assert_eq!(files[0].media_type, "application/wasm");
    assert!(files.iter().all(|file: &Received| file.bytes > 0));
}

#[tokio::test]
async fn every_format_is_exported_and_recorded_for_the_account() {
    let session = Session::start().await;
    let id = session.create(4, 4).await["id"].clone();

    for format in ["wasm", "gif", "apng", "sprite_sheet", "png_frames"] {
        let arguments = json!({ "id": id, "format": format, "scale": 2, "destination": "." });
        session.ok("export", arguments).await;
    }

    assert_eq!(session.delivery.calls.lock().unwrap().len(), 5);
    let events = session.events.events();
    let exports: Vec<_> = events
        .iter()
        .filter(|e| e.name == "export_completed")
        .collect();
    assert_eq!(exports.len(), 5);
    assert!(
        exports
            .iter()
            .all(|event| event.account == ACCOUNT.account())
    );
    assert!(
        exports[0]
            .properties
            .contains(&("source", "mcp".to_owned()))
    );
}

#[tokio::test]
async fn a_failed_export_reaches_no_delivery() {
    let session = Session::start().await;
    let id = session.create(4, 4).await["id"].clone();

    let tag = json!({ "id": id, "format": "gif", "tag": "walk", "destination": "." });
    let (code, params) = session.fails("export", tag).await;
    let scale = json!({ "id": id, "format": "gif", "scale": 0, "destination": "." });
    let format = json!({ "id": id, "format": "bmp", "destination": "." });
    let missing = json!({ "id": Uuid::nil(), "format": "gif", "destination": "." });

    assert_eq!(
        (code.as_str(), params),
        ("export.tag_not_found", json!({ "name": "walk" }))
    );
    assert_eq!(session.code("export", scale).await, "export.scale");
    assert_eq!(session.code("export", format).await, "request.malformed");
    assert_eq!(
        session.code("export", missing).await,
        "library.animation_not_found"
    );
    assert!(session.delivery.calls.lock().unwrap().is_empty());
}

#[tokio::test]
async fn a_snippet_gives_the_code_and_where_each_file_goes() {
    let session = Session::start().await;
    let id = tagged(&session).await;

    let arguments = json!({ "id": id, "framework": "html", "tag": "blink" });
    let html = session.ok("get_embed_snippet", arguments).await;

    assert!(html["code"].as_str().unwrap().contains("<life-pixel"));
    let files = html["files"].as_array().unwrap();
    let names: Vec<&Value> = files.iter().map(|file| &file["name"]).collect();
    assert_eq!(names, [&json!("mascot.wasm"), &json!("life-pixel.js")]);
    for framework in ["angular", "react", "vue"] {
        let arguments = json!({ "id": id, "framework": framework });
        let snippet = session.ok("get_embed_snippet", arguments).await;
        assert!(!snippet["code"].as_str().unwrap().is_empty(), "{framework}");
        assert_eq!(snippet["files"].as_array().unwrap().len(), 2, "{framework}");
    }
}

#[tokio::test]
async fn a_snippet_needs_a_known_tag_and_framework() {
    let session = Session::start().await;
    let id = tagged(&session).await;

    let tag = json!({ "id": id, "framework": "html", "tag": "walk" });
    let framework = json!({ "id": id, "framework": "svelte" });

    assert_eq!(
        session.code("get_embed_snippet", tag).await,
        "export.tag_not_found"
    );
    assert_eq!(
        session.code("get_embed_snippet", framework).await,
        "request.malformed"
    );
}
