//! A request's screenshot: checked before it is decoded, re-encoded as a PNG without its
//! metadata at `support/<request>/screenshot.png`, never served back, and swept once no row
//! points at it.

use std::time::Duration;

use axum::http::Method;
use life_pixel_core::limits::{SCREENSHOT_MAX_BYTES, SCREENSHOT_MAX_SIDE};
use life_pixel_server::storage::Sweeper;
use life_pixel_service::support::memory::{
    jpeg_screenshot, png_claiming_side, png_screenshot_with_text,
};
use object_store::ObjectStoreExt;
use object_store::path::Path;
use serde_json::json;

use crate::stack::{ApiStack, api, get};
use crate::{CONTEXT, Part, REQUESTS, execute, id, new_request, stored_requests, with_form};

/// The first bytes of every PNG.
const PNG_SIGNATURE: &[u8] = b"\x89PNG\r\n\x1a\n";
/// The text a screenshot's metadata hides.
const SECRET: &str = "ada@example.org";

/// The key of the screenshot of the request `id`, under the test's prefix.
fn key_of(id: &str) -> String {
    format!("support/{id}/screenshot.png")
}

/// The bytes stored under `key`.
async fn stored(stack: &ApiStack, key: &str) -> Vec<u8> {
    let objects = stack.storage.objects();
    let read = objects.get(&Path::from(key)).await.unwrap();
    read.bytes().await.unwrap().to_vec()
}

/// Whether `needle` is somewhere in `haystack`.
fn contains(haystack: &[u8], needle: &[u8]) -> bool {
    haystack
        .windows(needle.len())
        .any(|window| window == needle)
}

#[tokio::test]
async fn a_png_is_stored_re_encoded_without_its_metadata_and_never_served_back() {
    let stack = ApiStack::new().await;
    let ada = stack.sign_up("ada@example.org").await;
    let upload = png_screenshot_with_text((32, 24), ("Author", SECRET));
    assert!(contains(&upload, SECRET.as_bytes()));

    let request = new_request(&ada, ("bug", "See the capture"), Some(&upload));
    let created = stack.created(request).await;

    assert_eq!(created["hasScreenshot"], true);
    let key = key_of(&id(&created));
    assert_eq!(
        stack.storage.keys().await.unwrap(),
        std::slice::from_ref(&key)
    );
    let png = stored(&stack, &key).await;
    assert!(png.starts_with(PNG_SIGNATURE));
    assert!(!contains(&png, b"tEXt"));
    assert!(!contains(&png, SECRET.as_bytes()));
    let detail = stack
        .expect(200, get(&ada, &format!("{REQUESTS}/{}", id(&created))))
        .await;
    assert!(!contains(&detail.body, b"support/"), "the key stays inside");
    assert_eq!(detail.json()["hasScreenshot"], true);
}

#[tokio::test]
async fn a_jpeg_is_stored_as_a_png() {
    let stack = ApiStack::new().await;
    let ada = stack.sign_up("ada@example.org").await;
    let upload = jpeg_screenshot((40, 30));
    let parts = [
        ("category", Part::Text("bug")),
        ("message", Part::Text("A photo of the screen")),
        ("context", Part::Text(CONTEXT)),
        ("screenshot", Part::File(&upload, "image/jpeg")),
    ];

    let request = with_form(api(&ada, Method::POST, REQUESTS), &parts);
    let created = stack.created(request).await;

    let png = stored(&stack, &key_of(&id(&created))).await;
    assert!(png.starts_with(PNG_SIGNATURE));
}

#[tokio::test]
async fn a_screenshot_claiming_too_many_pixels_is_refused_before_it_is_decoded() {
    let stack = ApiStack::new().await;
    let ada = stack.sign_up("ada@example.org").await;
    let giant = png_claiming_side(100_000);

    let request = new_request(&ada, ("bug", "Huge"), Some(&giant));
    let params = stack.refused(request, (422, "support.screenshot")).await;

    let limits = json!({ "maxSide": SCREENSHOT_MAX_SIDE, "maxBytes": SCREENSHOT_MAX_BYTES });
    assert_eq!(params, limits);
    assert_eq!(stored_requests(&stack).await, 0);
    assert!(stack.storage.keys().await.unwrap().is_empty());
}

#[tokio::test]
async fn a_screenshot_neither_png_nor_jpeg_or_too_heavy_is_refused() {
    let stack = ApiStack::new().await;
    let ada = stack.sign_up("ada@example.org").await;
    let gif = b"GIF89a\x01\x00\x01\x00\x00\x00\x00;".to_vec();
    let mut heavy = png_screenshot_with_text((8, 8), ("Comment", "padding"));
    heavy.resize(SCREENSHOT_MAX_BYTES + 1, 0);

    for upload in [gif, heavy] {
        let request = new_request(&ada, ("bug", "Look"), Some(&upload));
        stack.refused(request, (422, "support.screenshot")).await;
    }
    let too_large = vec![0; 6 * 1024 * 1024 + 1];
    let request = new_request(&ada, ("bug", "Look"), Some(&too_large));
    stack.refused(request, (413, "request.too_large")).await;
    assert_eq!(stored_requests(&stack).await, 0);
    assert!(stack.storage.keys().await.unwrap().is_empty());
}

#[tokio::test]
async fn the_sweeper_deletes_the_screenshots_no_request_points_at() {
    let stack = ApiStack::new().await;
    let ada = stack.sign_up("ada@example.org").await;
    let upload = png_screenshot_with_text((8, 8), ("Comment", "kept"));
    let kept = new_request(&ada, ("bug", "Kept"), Some(&upload));
    let kept = id(&stack.created(kept).await);
    let orphan = new_request(&ada, ("bug", "Orphaned"), Some(&upload));
    let orphan = id(&stack.created(orphan).await);
    execute(
        &stack,
        "delete from support_requests where id = $1",
        &orphan,
    )
    .await;

    let sweeper = Sweeper::new(stack.database.pool().clone(), stack.storage.objects());
    let deleted = sweeper.with_min_age(Duration::ZERO).sweep().await.unwrap();

    assert_eq!(deleted, 1);
    assert_eq!(stack.storage.keys().await.unwrap(), [key_of(&kept)]);
}
