//! `set_palette`: positional entries, entry 0 kept transparent, colours in use kept.

use life_pixel_core::limits::MAX_PALETTE_ENTRIES;
use life_pixel_service::animation::{EditingError, SetPaletteRequest};
use serde_json::json;

use super::support::{Fixture, assert_coded, colours};
use crate::common::ACCOUNT;

const SIX: [&str; 6] = [
    "#00000000",
    "#1d2b53ff",
    "#7e2553ff",
    "#008751ff",
    "#ab5236ff",
    "#5f574fff",
];

async fn set(fixture: &Fixture, colors: &[&str]) -> Result<Vec<String>, EditingError> {
    let request = SetPaletteRequest {
        id: fixture.id,
        colors: colours(colors),
    };
    let view = fixture.editing.set_palette(&ACCOUNT, request).await?;
    Ok(view.palette.iter().map(ToString::to_string).collect())
}

#[tokio::test]
async fn a_palette_replaces_the_entries_and_keeps_the_pixels_indices() {
    let fixture = Fixture::blank(4, 1).await;
    fixture.write(0, &["1234"]).await.unwrap();

    let palette = set(&fixture, &SIX).await.unwrap();

    assert_eq!(palette, SIX);
    assert_eq!(fixture.describe(&[0]).await.grids[0].rows, ["1234"]);
}

#[tokio::test]
async fn a_palette_grows_up_to_the_limit() {
    let fixture = Fixture::blank(2, 2).await;
    let mut many = vec!["#00000000"];
    many.resize(MAX_PALETTE_ENTRIES, "#ffffffff");

    let palette = set(&fixture, &many).await.unwrap();

    assert_eq!(palette.len(), MAX_PALETTE_ENTRIES);
}

#[tokio::test]
async fn entry_0_must_be_transparent_and_keeps_its_own_colour() {
    let fixture = Fixture::blank(2, 2).await;
    let request = |colors: &[&str]| SetPaletteRequest {
        id: fixture.id,
        colors: colours(colors),
    };

    let opaque = fixture
        .editing
        .set_palette(&ACCOUNT, request(&["#ff0000ff"]));
    let empty = fixture.editing.set_palette(&ACCOUNT, request(&[]));

    let params = json!({ "max": MAX_PALETTE_ENTRIES });
    assert_coded(
        &opaque.await.unwrap_err(),
        "document.palette",
        params.clone(),
    );
    assert_coded(&empty.await.unwrap_err(), "document.palette", params);
    let palette = set(&fixture, &["#ff000000", "#ffffffff"]).await.unwrap();
    assert_eq!(palette, ["#00000000", "#ffffffff"]);
}

#[tokio::test]
async fn a_colour_still_used_cannot_be_removed() {
    let fixture = Fixture::blank(4, 1).await;
    fixture.write(0, &["1.4."]).await.unwrap();
    let before = fixture.version().await;

    let dropping_3_and_above = set(&fixture, &SIX[..3]).await.unwrap_err();
    let dropping_4_and_above = set(&fixture, &SIX[..4]).await.unwrap_err();

    assert_coded(
        &dropping_3_and_above,
        "edit.palette_in_use",
        json!({ "index": 4 }),
    );
    assert_coded(
        &dropping_4_and_above,
        "edit.palette_in_use",
        json!({ "index": 4 }),
    );
    assert_eq!(fixture.version().await, before);
    assert_eq!(set(&fixture, &SIX[..5]).await.unwrap(), SIX[..5]);
}
