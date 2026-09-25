//! `render_preview`: an image content and its layout, within the caps.

use base64::Engine as _;
use base64::engine::general_purpose::STANDARD;
use life_pixel_core::limits::{CANVAS_MAX_SIDE, EXPORT_MAX_SCALE};
use rmcp::model::{CallToolResult, ContentBlock};
use serde_json::{Value, json};

use super::session::Session;

/// The PNG's size and the layout text of a preview.
fn image_and_layout(result: &CallToolResult) -> ((u32, u32), Value) {
    assert_eq!(result.is_error, Some(false), "{result:?}");
    let [ContentBlock::Image(image), ContentBlock::Text(text)] = result.content.as_slice() else {
        panic!("an image and a text expected: {result:?}");
    };
    assert_eq!(image.mime_type, "image/png");
    let png = STANDARD.decode(&image.data).unwrap();
    let reader = png::Decoder::new(std::io::Cursor::new(png))
        .read_info()
        .unwrap();
    let size = (reader.info().width, reader.info().height);
    (size, serde_json::from_str(&text.text).unwrap())
}

#[tokio::test]
async fn a_frame_preview_is_a_png_of_the_announced_size() {
    let session = Session::start().await;
    let id = session.create(16, 8).await["id"].clone();

    let result = session
        .call(
            "render_preview",
            json!({ "id": id, "frame": 0, "scale": 3 }),
        )
        .await;

    let (size, layout) = image_and_layout(&result);
    assert_eq!(size, (48, 24));
    let cell = json!({ "frame": 0, "x": 0, "y": 0, "width": 48, "height": 24 });
    assert_eq!(
        layout,
        json!({ "width": 48, "height": 24, "scale": 3, "layout": [cell] })
    );
}

#[tokio::test]
async fn a_contact_sheet_lays_every_frame_out_at_the_automatic_scale() {
    let session = Session::start().await;
    let id = session.create(16, 16).await["id"].clone();
    session
        .ok(
            "edit_frames",
            json!({ "id": id, "edits": [{ "op": "add" }] }),
        )
        .await;

    let result = session.call("render_preview", json!({ "id": id })).await;

    let (size, layout) = image_and_layout(&result);
    assert_eq!(layout["scale"], 8);
    assert_eq!(
        json!([size.0, size.1]),
        json!([layout["width"], layout["height"]])
    );
    assert_eq!(layout["layout"].as_array().unwrap().len(), 2);
    assert_eq!(layout["layout"][1]["frame"], 1);
}

#[tokio::test]
async fn a_preview_beyond_its_caps_or_scale_fails() {
    let session = Session::start().await;
    let side = CANVAS_MAX_SIDE;
    let id = session.create(side, side).await["id"].clone();

    let too_large = json!({ "id": id, "frame": 0, "scale": EXPORT_MAX_SCALE });
    let (code, params) = session.fails("render_preview", too_large).await;
    let scale = json!({ "id": id, "frame": 0, "scale": 0 });
    let frame = json!({ "id": id, "frame": 9 });

    assert_eq!(code, "preview.too_large");
    assert!(params["maxSide"].is_number());
    assert_eq!(session.code("render_preview", scale).await, "export.scale");
    assert_eq!(
        session.code("render_preview", frame).await,
        "edit.frame_not_found"
    );
}
