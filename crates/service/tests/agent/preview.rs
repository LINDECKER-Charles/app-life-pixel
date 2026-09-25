//! `preview`: one frame or a contact sheet, at the automatic scale and at the caps.

use std::io::Cursor;

use life_pixel_core::limits::{EXPORT_MAX_SCALE, EXPORT_MIN_SCALE, PREVIEW_MAX_SIDE};
use life_pixel_service::animation::{
    EditFramesRequest, FrameEdit, Preview, PreviewCell, PreviewRequest,
};
use png::{ColorType, Decoder, Transformations};
use serde_json::json;

use super::support::{Fixture, assert_coded};
use crate::common::ACCOUNT;

async fn preview(fixture: &Fixture, frame: Option<u16>, scale: Option<u8>) -> Preview {
    let request = PreviewRequest {
        id: fixture.id,
        frame,
        scale,
    };
    fixture.editing.preview(&ACCOUNT, request).await.unwrap()
}

/// The size, palette length and palette indices of an indexed PNG.
fn decode(png: &[u8]) -> ((u32, u32), usize, Vec<u8>) {
    let mut decoder = Decoder::new(Cursor::new(png));
    decoder.set_transformations(Transformations::IDENTITY);
    let mut reader = decoder.read_info().unwrap();
    assert_eq!(reader.info().color_type, ColorType::Indexed);
    let palette_len = reader.info().palette.as_ref().unwrap().len() / 3;
    assert_eq!(reader.info().trns.as_ref().unwrap().len(), palette_len);
    let mut buffer = vec![0; reader.output_buffer_size().unwrap()];
    let frame = reader.next_frame(&mut buffer).unwrap();
    buffer.truncate(frame.buffer_size());
    ((frame.width, frame.height), palette_len, buffer)
}

async fn add_frames(fixture: &Fixture, count: usize) {
    let add = FrameEdit::Add {
        position: None,
        duration_ms: None,
    };
    let request = EditFramesRequest {
        id: fixture.id,
        edits: vec![add; count],
    };
    fixture
        .editing
        .edit_frames(&ACCOUNT, request)
        .await
        .unwrap();
}

#[tokio::test]
async fn a_frame_is_scaled_up_to_8_within_512_pixels() {
    let fixture = Fixture::blank(16, 16).await;
    let mut rows = vec!["................"; 16];
    rows[0] = "3...............";
    fixture.write(0, &rows).await.unwrap();

    let preview = preview(&fixture, Some(0), None).await;

    assert_eq!(
        (preview.width, preview.height, preview.scale),
        (128, 128, 8)
    );
    let cell = PreviewCell {
        frame: 0,
        x: 0,
        y: 0,
        width: 128,
        height: 128,
    };
    assert_eq!(preview.layout, [cell]);
    let ((width, height), palette_len, pixels) = decode(&preview.png);
    assert_eq!((width, height, palette_len), (128, 128, 16));
    assert_eq!((pixels[0], pixels[7], pixels[8]), (3, 3, 0));
    assert_eq!((pixels[7 * 128], pixels[8 * 128]), (3, 0));
}

#[tokio::test]
async fn the_automatic_scale_keeps_the_longest_side_within_512() {
    for (width, height, scale) in [(100, 30, 5), (512, 1, 1), (65, 65, 7)] {
        let fixture = Fixture::blank(width, height).await;

        let preview = preview(&fixture, None, None).await;

        assert_eq!(preview.scale, scale, "{width} × {height}");
        let expected = (u32::from(width) * scale, u32::from(height) * scale);
        assert_eq!((preview.width, preview.height), expected);
    }
}

#[tokio::test]
async fn a_contact_sheet_lays_frames_out_in_square_root_columns_with_a_gap() {
    let fixture = Fixture::blank(2, 2).await;
    add_frames(&fixture, 2).await;
    fixture.write(2, &["44", "44"]).await.unwrap();

    let preview = preview(&fixture, None, Some(1)).await;

    assert_eq!((preview.width, preview.height), (6, 6));
    let origins: Vec<(u16, u32, u32)> = preview
        .layout
        .iter()
        .map(|cell| (cell.frame, cell.x, cell.y))
        .collect();
    assert_eq!(origins, [(0, 0, 0), (1, 4, 0), (2, 0, 4)]);
    let (_, _, pixels) = decode(&preview.png);
    let rows: Vec<&[u8]> = pixels.chunks(6).collect();
    assert_eq!(rows[4], [4, 4, 0, 0, 0, 0]);
    assert!(rows[2].iter().chain(rows[3]).all(|&index| index == 0));
}

#[tokio::test]
async fn a_scaled_sheet_scales_its_gaps_and_its_layout() {
    let fixture = Fixture::blank(10, 5).await;
    add_frames(&fixture, 1).await;

    let preview = preview(&fixture, None, None).await;

    assert_eq!(preview.scale, 8);
    assert_eq!((preview.width, preview.height), (22 * 8, 5 * 8));
    let second = preview.layout[1];
    assert_eq!(
        (second.x, second.y, second.width, second.height),
        (12 * 8, 0, 80, 40)
    );
}

#[tokio::test]
async fn a_preview_never_exceeds_the_largest_side() {
    let fits = Fixture::blank(64, 64).await;
    let wider = Fixture::blank(65, 64).await;
    let largest = u8::try_from(PREVIEW_MAX_SIDE / 64).unwrap();
    let request = |fixture: &Fixture, scale| PreviewRequest {
        id: fixture.id,
        frame: Some(0),
        scale: Some(scale),
    };

    let at_cap = fits
        .editing
        .preview(&ACCOUNT, request(&fits, largest))
        .await;
    let above = wider
        .editing
        .preview(&ACCOUNT, request(&wider, largest))
        .await;
    let zero = fits.editing.preview(&ACCOUNT, request(&fits, 0)).await;

    assert_eq!(at_cap.unwrap().width, PREVIEW_MAX_SIDE);
    let max_side = json!({ "maxSide": PREVIEW_MAX_SIDE });
    assert_coded(&above.unwrap_err(), "preview.too_large", max_side);
    let bounds = json!({ "min": EXPORT_MIN_SCALE, "max": EXPORT_MAX_SCALE });
    assert_coded(&zero.unwrap_err(), "export.scale", bounds);
}

#[tokio::test]
async fn a_sheet_too_large_even_at_scale_1_is_refused() {
    let fixture = Fixture::blank(512, 512).await;
    add_frames(&fixture, 3).await;
    let request = |frame| PreviewRequest {
        id: fixture.id,
        frame,
        scale: None,
    };

    let sheet = fixture.editing.preview(&ACCOUNT, request(None)).await;
    let single = fixture.editing.preview(&ACCOUNT, request(Some(3))).await;
    let missing = fixture.editing.preview(&ACCOUNT, request(Some(4))).await;

    let max_side = json!({ "maxSide": PREVIEW_MAX_SIDE });
    assert_coded(&sheet.unwrap_err(), "preview.too_large", max_side);
    assert_eq!(single.unwrap().scale, 1);
    assert_coded(&missing.unwrap_err(), "edit.frame_not_found", json!({}));
}
