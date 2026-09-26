//! Screenshots: checked before decoding, re-encoded as PNG, stored before their request.

use std::io::Cursor;

use image::{ImageFormat, ImageReader};
use life_pixel_core::limits::{SCREENSHOT_MAX_BYTES, SCREENSHOT_MAX_SIDE};

use super::{Harness, account, submission};
use crate::Coded;
use crate::support::memory::{jpeg_screenshot, png_claiming_side, png_screenshot_with_text};
use crate::support::{Screenshot, ScreenshotRejection, SupportError};

/// The text chunk a sample carries, which must not survive.
const SECRET: (&str, &str) = ("Author", "ada@example.com");

/// The chunk types of `png`, in order.
fn chunk_types(png: &[u8]) -> Vec<String> {
    let mut types = Vec::new();
    let mut at = 8;
    while at + 8 <= png.len() {
        let length = u32::from_be_bytes(png[at..at + 4].try_into().unwrap());
        types.push(String::from_utf8_lossy(&png[at + 4..at + 8]).into_owned());
        at += 12 + usize::try_from(length).unwrap();
    }
    types
}

#[test]
fn a_png_is_re_encoded_without_its_metadata_and_keeps_its_pixels() {
    let upload = png_screenshot_with_text((3, 2), SECRET);
    assert!(chunk_types(&upload).contains(&"tEXt".to_owned()));

    let png = Screenshot::prepare(&upload).unwrap().into_png();

    assert_eq!(chunk_types(&png), ["IHDR", "IDAT", "IEND"]);
    assert!(
        !png.windows(SECRET.1.len())
            .any(|window| window == SECRET.1.as_bytes())
    );
    let decode = |bytes: &[u8]| {
        let reader = ImageReader::with_format(Cursor::new(bytes), ImageFormat::Png);
        reader.decode().unwrap().to_rgba8().into_raw()
    };
    assert_eq!(decode(&png), decode(&upload));
}

#[test]
fn a_jpeg_becomes_a_png() {
    let png = Screenshot::prepare(&jpeg_screenshot((8, 5)))
        .unwrap()
        .into_png();

    assert_eq!(image::guess_format(&png).unwrap(), ImageFormat::Png);
    let reader = ImageReader::with_format(Cursor::new(&png[..]), ImageFormat::Png);
    assert_eq!(reader.into_dimensions().unwrap(), (8, 5));
}

#[test]
fn a_header_claiming_100_000_pixels_a_side_is_refused_before_decoding() {
    // Its pixels are missing: decoding would fail as `Undecodable`, the header check first.
    let refused = Screenshot::prepare(&png_claiming_side(100_000));
    assert_eq!(refused, Err(ScreenshotRejection::Dimensions));
    let just_over = Screenshot::prepare(&png_claiming_side(SCREENSHOT_MAX_SIDE + 1));
    assert_eq!(just_over, Err(ScreenshotRejection::Dimensions));
    let within = Screenshot::prepare(&png_claiming_side(SCREENSHOT_MAX_SIDE));
    assert_eq!(within, Err(ScreenshotRejection::Undecodable));
}

#[test]
fn only_pngs_and_jpegs_within_the_bytes_limit_are_screenshots() {
    let gif = b"GIF89a\x01\x00\x01\x00\x00\x00\x00;".to_vec();
    assert_eq!(Screenshot::prepare(&gif), Err(ScreenshotRejection::Format));
    assert_eq!(
        Screenshot::prepare(b"hello"),
        Err(ScreenshotRejection::Format)
    );
    let mut huge = png_screenshot_with_text((1, 1), SECRET);
    huge.resize(SCREENSHOT_MAX_BYTES + 1, 0);
    assert_eq!(
        Screenshot::prepare(&huge),
        Err(ScreenshotRejection::TooManyBytes)
    );
}

#[tokio::test]
async fn a_request_stores_its_screenshot_under_its_id() {
    let harness = Harness::new();
    let upload = png_screenshot_with_text((4, 4), SECRET);

    let sent = harness
        .support
        .create(account(1), submission("Look", Some(upload)))
        .await;

    let request = sent.unwrap().request;
    assert!(request.has_screenshot);
    let key = format!("support/{}/screenshot.png", request.id);
    assert_eq!(harness.screenshots.keys(), vec![key.clone()]);
    let (_, stored_key) = harness.requests.stored(request.id).unwrap();
    assert_eq!(stored_key, Some(key));
}

#[tokio::test]
async fn a_refused_screenshot_refuses_the_request_with_the_limits() {
    let harness = Harness::new();
    let upload = png_claiming_side(100_000);

    let refused = harness
        .support
        .create(account(1), submission("Look", Some(upload)))
        .await;

    let error = refused.unwrap_err();
    assert_eq!(error, SupportError::Screenshot);
    assert_eq!(error.code(), "support.screenshot");
    let params = error.params();
    assert_eq!(params["maxSide"], 4_096);
    assert_eq!(params["maxBytes"], 5_242_880);
    assert!(harness.screenshots.keys().is_empty());
    assert!(harness.events.events().is_empty());
}

#[tokio::test]
async fn a_request_the_store_refuses_leaves_no_screenshot_behind() {
    let harness = Harness::new();
    harness.requests.set_unavailable(true);
    let upload = png_screenshot_with_text((2, 2), SECRET);

    let refused = harness
        .support
        .create(account(1), submission("Look", Some(upload)))
        .await;

    assert_eq!(refused.unwrap_err(), SupportError::Unavailable);
    assert!(harness.screenshots.keys().is_empty());
}
