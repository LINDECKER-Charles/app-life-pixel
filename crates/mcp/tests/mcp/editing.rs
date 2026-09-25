//! `set_palette`, `write_frame`, `draw`, `edit_frames` and `set_tags`.

use life_pixel_core::limits::DRAW_MAX_OPERATIONS;
use serde_json::{Value, json};

use super::session::Session;

/// The composite of frame `frame` of the animation `id`, as grid rows.
async fn rows(session: &Session, id: &Value, frame: u16) -> Value {
    let arguments = json!({ "id": id, "include_pixels": true, "frames": [frame] });
    session.ok("get_animation", arguments).await["grids"][0]["rows"].clone()
}

#[tokio::test]
async fn a_palette_is_replaced_and_a_colour_still_painted_stays() {
    let session = Session::start().await;
    let id = session.create(2, 1).await["id"].clone();
    let palette = json!(["#00000000", "#ff0000ff", "#00ff00ff"]);

    let view = session
        .ok("set_palette", json!({ "id": id, "palette": palette }))
        .await;
    session
        .ok(
            "write_frame",
            json!({ "id": id, "frame": 0, "grid": ["2."] }),
        )
        .await;
    let shrunk = json!({ "id": id, "palette": ["#00000000", "#ff0000ff"] });
    let (code, params) = session.fails("set_palette", shrunk).await;
    let opaque = json!({ "id": id, "palette": ["#ffffffff"] });

    assert_eq!(view["palette"], palette);
    assert_eq!(
        (code.as_str(), params),
        ("edit.palette_in_use", json!({ "index": 2 }))
    );
    assert_eq!(
        session.code("set_palette", opaque).await,
        "document.palette"
    );
    let not_a_colour = json!({ "id": id, "palette": ["#00000000", "red"] });
    assert_eq!(
        session.code("set_palette", not_a_colour).await,
        "document.palette"
    );
}

#[tokio::test]
async fn a_grid_is_written_then_read_back() {
    let session = Session::start().await;
    let id = session.create(3, 2).await["id"].clone();

    let view = session
        .ok(
            "write_frame",
            json!({ "id": id, "frame": 0, "grid": ["12.", ".3."] }),
        )
        .await;

    assert_eq!(view["id"], id);
    assert_eq!(rows(&session, &id, 0).await, json!(["12.", ".3."]));
}

#[tokio::test]
async fn a_grid_that_does_not_fit_fails_with_its_code() {
    let session = Session::start().await;
    let created = session.create(3, 2).await;
    let id = &created["id"];
    let layer = created["layers"][0]["id"].as_u64().unwrap();

    let short = json!({ "id": id, "frame": 0, "grid": ["..."] });
    let (code, params) = session.fails("write_frame", short).await;
    assert_eq!(
        (code.as_str(), params),
        ("grid.size", json!({ "width": 3, "height": 2 }))
    );
    let odd = json!({ "id": id, "frame": 0, "grid": ["..?", "..."] });
    let (code, params) = session.fails("write_frame", odd).await;
    assert_eq!(
        (code.as_str(), params),
        ("grid.character", json!({ "row": 0, "column": 2 }))
    );
    let frame = json!({ "id": id, "frame": 4, "grid": ["...", "..."] });
    assert_eq!(
        session.code("write_frame", frame).await,
        "edit.frame_not_found"
    );
    let layer = json!({ "id": id, "frame": 0, "layer_id": layer + 7, "grid": ["...", "..."] });
    assert_eq!(
        session.code("write_frame", layer).await,
        "edit.layer_not_found"
    );
}

#[tokio::test]
async fn drawing_operations_apply_in_order() {
    let session = Session::start().await;
    let id = session.create(4, 3).await["id"].clone();
    let operations = json!([
        { "op": "rectangle", "from": [0, 0], "to": [3, 2], "index": 1 },
        { "op": "fill", "x": 1, "y": 1, "index": 2 },
        { "op": "line", "from": [0, 2], "to": [3, 2], "index": 3 },
        { "op": "pixel", "x": 3, "y": 0, "index": 0 },
    ]);

    session
        .ok(
            "draw",
            json!({ "id": id, "frame": 0, "operations": operations }),
        )
        .await;

    assert_eq!(
        rows(&session, &id, 0).await,
        json!(["111.", "1221", "3333"])
    );
}

#[tokio::test]
async fn too_many_or_refused_operations_change_nothing() {
    let session = Session::start().await;
    let id = session.create(2, 2).await["id"].clone();
    let pixel = json!({ "op": "pixel", "x": 0, "y": 0, "index": 1 });
    let many = vec![pixel.clone(); DRAW_MAX_OPERATIONS + 1];

    let (code, params) = session
        .fails("draw", json!({ "id": id, "frame": 0, "operations": many }))
        .await;
    let unknown_index = json!({ "op": "pixel", "x": 1, "y": 1, "index": 200 });
    let refused = json!({ "id": id, "frame": 0, "operations": [pixel, unknown_index] });
    let refused_code = session.code("draw", refused).await;
    let unknown_op = json!({ "id": id, "frame": 0, "operations": [{ "op": "spray" }] });

    assert_eq!(code, "draw.too_many_operations");
    assert_eq!(params, json!({ "max": DRAW_MAX_OPERATIONS }));
    assert_eq!(refused_code, "document.palette");
    assert_eq!(session.code("draw", unknown_op).await, "request.malformed");
    assert_eq!(rows(&session, &id, 0).await, json!(["..", ".."]));
}

#[tokio::test]
async fn frame_edits_return_the_frames_and_the_tags() {
    let session = Session::start().await;
    let id = session.create(2, 2).await["id"].clone();
    let edits = json!([
        { "op": "add", "duration_ms": 200 },
        { "op": "duplicate", "frame": 1 },
        { "op": "set_duration", "frame": 0, "duration_ms": 50 },
        { "op": "move", "frame": 0, "position": 2 },
        { "op": "delete", "frame": 1 },
    ]);

    let result = session
        .ok("edit_frames", json!({ "id": id, "edits": edits }))
        .await;

    let frames = json!([
        { "position": 0, "duration_ms": 200 },
        { "position": 1, "duration_ms": 50 },
    ]);
    assert_eq!(result, json!({ "frames": frames, "tags": [] }));
}

#[tokio::test]
async fn deleting_the_last_frame_fails_and_keeps_every_edit_out() {
    let session = Session::start().await;
    let id = session.create(2, 2).await["id"].clone();
    let edits =
        json!([{ "op": "add" }, { "op": "delete", "frame": 0 }, { "op": "delete", "frame": 0 }]);

    let code = session
        .code("edit_frames", json!({ "id": id, "edits": edits }))
        .await;
    let view = session.ok("get_animation", json!({ "id": id })).await;

    assert_eq!(code, "edit.last_frame");
    assert_eq!(view["frames"].as_array().unwrap().len(), 1);
}

#[tokio::test]
async fn tags_are_replaced_and_checked() {
    let session = Session::start().await;
    let id = session.create(2, 2).await["id"].clone();
    session
        .ok(
            "edit_frames",
            json!({ "id": id, "edits": [{ "op": "add" }] }),
        )
        .await;
    let blink = json!({ "name": "blink", "first": 1, "last": 1, "loop": "once" });

    let result = session
        .ok("set_tags", json!({ "id": id, "tags": [blink] }))
        .await;
    let beyond = json!({ "name": "far", "first": 0, "last": 5, "loop": "loop" });
    let (code, params) = session
        .fails("set_tags", json!({ "id": id, "tags": [beyond] }))
        .await;

    assert_eq!(result, json!({ "tags": [blink] }));
    assert_eq!(
        (code.as_str(), params),
        ("document.tag", json!({ "name": "far" }))
    );
}
