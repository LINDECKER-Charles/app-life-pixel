//! `write_frame` and `draw`: text grids written then described back, each drawing operation,
//! and the limits and quota of the interface.

use life_pixel_core::limits::{CANVAS_MAX_SIDE, DRAW_MAX_OPERATIONS};
use life_pixel_service::animation::{DrawOperation, DrawRequest, EditingError, WriteFrameRequest};
use serde_json::json;

use super::support::{Fixture, assert_coded, grid, two_layers};
use crate::common::{ACCOUNT, spec};

const CROSS: [&str; 5] = ["..2..", "..2..", "22222", "..2..", "..2.."];

async fn draw(
    fixture: &Fixture,
    operations: Vec<DrawOperation>,
) -> Result<Vec<String>, EditingError> {
    let request = DrawRequest {
        id: fixture.id,
        frame: 0,
        layer: None,
        operations,
    };
    fixture.editing.draw(&ACCOUNT, request).await?;
    Ok(fixture.describe(&[0]).await.grids[0].rows.clone())
}

#[tokio::test]
async fn a_grid_written_is_described_back() {
    let fixture = Fixture::blank(5, 5).await;

    let view = fixture.write(0, &CROSS).await.unwrap();

    assert_ne!(view.version, 0);
    assert_eq!(view.version, fixture.version().await);
    assert_eq!(fixture.describe(&[0]).await.grids[0].rows, CROSS);
}

#[tokio::test]
async fn a_grid_replaces_the_whole_cel() {
    let fixture = Fixture::blank(3, 2).await;
    fixture.write(0, &["123", "456"]).await.unwrap();

    fixture.write(0, &["..7", "7.."]).await.unwrap();

    assert_eq!(fixture.describe(&[0]).await.grids[0].rows, ["..7", "7.."]);
}

#[tokio::test]
async fn a_blank_grid_clears_the_cel_and_the_largest_canvas_fits() {
    let side = usize::from(CANVAS_MAX_SIDE);
    let spec = spec("Mascot", CANVAS_MAX_SIDE, CANVAS_MAX_SIDE);
    let fixture = Fixture::with_quota(10_000_000, spec).await;
    let checkerboard: Vec<String> = (0..side)
        .map(|y| (0..side).map(|x| ['1', '2'][(x + y) % 2]).collect())
        .collect();
    let blank = vec![".".repeat(side); side];
    let request = |rows: &Vec<String>| WriteFrameRequest {
        id: fixture.id,
        frame: 0,
        layer: None,
        grid: rows.clone(),
    };

    let full = fixture
        .editing
        .write_frame(&ACCOUNT, request(&checkerboard));
    full.await.unwrap();
    assert_eq!(fixture.describe(&[0]).await.grids[0].rows, checkerboard);
    let cleared = fixture.editing.write_frame(&ACCOUNT, request(&blank));
    cleared.await.unwrap();

    assert_eq!(fixture.describe(&[0]).await.grids[0].rows, blank);
}

#[tokio::test]
async fn a_grid_goes_on_the_layer_asked_for_or_the_top_one() {
    let fixture = Fixture::imported(&two_layers()).await;
    let bottom = WriteFrameRequest {
        id: fixture.id,
        frame: 0,
        layer: Some(1),
        grid: grid(&["1111"; 4]),
    };
    fixture.editing.write_frame(&ACCOUNT, bottom).await.unwrap();

    fixture
        .write(0, &["2...", "....", "....", "...."])
        .await
        .unwrap();

    let rows = &fixture.describe(&[0]).await.grids[0].rows;
    assert_eq!(rows, &["2111", "1111", "1111", "1111"]);
}

#[tokio::test]
async fn a_grid_that_does_not_fit_is_refused_with_its_code() {
    let fixture = Fixture::blank(2, 2).await;
    let before = fixture.version().await;

    let size = fixture.write(0, &["..", "...", ".."]).await.unwrap_err();
    let character = fixture.write(0, &["..", ".#"]).await.unwrap_err();
    let index = fixture.write(0, &["..", ".g"]).await.unwrap_err();
    let frame = fixture.write(1, &["..", ".."]).await.unwrap_err();

    assert_coded(&size, "grid.size", json!({ "width": 2, "height": 2 }));
    assert_coded(
        &character,
        "grid.character",
        json!({ "row": 1, "column": 1 }),
    );
    let params = json!({ "row": 1, "column": 1, "index": 16 });
    assert_coded(&index, "grid.index", params);
    assert_coded(&frame, "edit.frame_not_found", json!({}));
    assert_eq!(fixture.version().await, before);
}

#[tokio::test]
async fn a_missing_layer_is_not_found() {
    let fixture = Fixture::blank(2, 2).await;
    let request = WriteFrameRequest {
        id: fixture.id,
        frame: 0,
        layer: Some(99),
        grid: grid(&["..", ".."]),
    };

    let error = fixture.editing.write_frame(&ACCOUNT, request).await;

    assert_coded(&error.unwrap_err(), "edit.layer_not_found", json!({}));
}

#[tokio::test]
async fn each_drawing_operation_goes_through_core() {
    let fixture = Fixture::blank(5, 5).await;
    let operations = serde_json::from_value(json!([
        { "op": "rectangle", "from": [0, 0], "to": [4, 4], "index": 1 },
        { "op": "line", "from": [1, 1], "to": [3, 3], "index": 2 },
        { "op": "pixel", "x": 3, "y": 1, "index": 3 },
        { "op": "pixel", "x": 9, "y": 9, "index": 3 },
        { "op": "fill", "x": 1, "y": 3, "index": 4 },
        { "op": "rectangle", "from": [4, 0], "to": [4, 1], "index": 5, "filled": true },
    ]));

    let rows = draw(&fixture, operations.unwrap()).await.unwrap();

    assert_eq!(rows, ["11115", "12.35", "142.1", "14421", "11111"]);
}

#[tokio::test]
async fn a_refused_operation_leaves_the_animation_as_it_was() {
    let fixture = Fixture::blank(3, 3).await;
    let before = fixture.version().await;
    let operations = vec![
        DrawOperation::Pixel {
            x: 0,
            y: 0,
            index: 1,
        },
        DrawOperation::Fill {
            x: 3,
            y: 0,
            index: 1,
        },
    ];

    let error = draw(&fixture, operations).await.unwrap_err();

    assert_coded(&error, "edit.out_of_canvas", json!({}));
    assert_eq!(fixture.version().await, before);
    assert_eq!(fixture.describe(&[0]).await.grids[0].rows, ["..."; 3]);
}

#[tokio::test]
async fn a_batch_holds_at_most_the_drawing_limit() {
    let fixture = Fixture::blank(3, 3).await;
    let pixel = DrawOperation::Pixel {
        x: 0,
        y: 0,
        index: 1,
    };

    let at_limit = draw(&fixture, vec![pixel.clone(); DRAW_MAX_OPERATIONS]).await;
    let above = draw(&fixture, vec![pixel; DRAW_MAX_OPERATIONS + 1]).await;

    assert_eq!(at_limit.unwrap()[0], "1..");
    let params = json!({ "max": DRAW_MAX_OPERATIONS });
    assert_coded(&above.unwrap_err(), "draw.too_many_operations", params);
}

#[tokio::test]
async fn drawing_operations_read_as_the_tool_sends_them() {
    let operations = json!([
        { "op": "pixel", "x": 1, "y": 2, "index": 3 },
        { "op": "line", "from": [0, 0], "to": [2, 0], "index": 1 },
        { "op": "rectangle", "from": [0, 0], "to": [1, 1], "index": 2 },
        { "op": "fill", "x": 0, "y": 0, "index": 4 },
    ]);

    let operations: Vec<DrawOperation> = serde_json::from_value(operations).unwrap();

    let rectangle = DrawOperation::Rectangle {
        from: [0, 0],
        to: [1, 1],
        index: 2,
        filled: false,
    };
    assert_eq!(operations[2], rectangle);
    assert_eq!(operations.len(), 4);
}

#[tokio::test]
async fn a_change_beyond_the_quota_is_refused() {
    let small = Fixture::blank(16, 16).await;
    let blank_bytes = small
        .harness
        .library
        .usage(&ACCOUNT)
        .await
        .unwrap()
        .used_bytes;
    let fixture = Fixture::with_quota(blank_bytes + 16, spec("Mascot", 16, 16)).await;
    let noise: Vec<String> = (0..16)
        .map(|row| {
            (0..16)
                .map(|column| if (row + column) % 2 == 0 { '1' } else { '2' })
                .collect()
        })
        .collect();
    let rows: Vec<&str> = noise.iter().map(String::as_str).collect();

    let error = fixture.write(0, &rows).await.unwrap_err();

    assert_eq!(
        life_pixel_service::Coded::code(&error),
        "quota.storage_exceeded"
    );
}
