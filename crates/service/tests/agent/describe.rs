//! `describe`: the structure without pixels, and composites as text grids on demand.

use life_pixel_core::DEFAULT_PALETTE;
use life_pixel_core::limits::DEFAULT_FRAME_DURATION_MS;
use life_pixel_service::animation::{DescribeRequest, FrameView, LayerView};
use life_pixel_service::{AnimationId, Owner};
use serde_json::json;
use uuid::Uuid;

use super::support::{Fixture, assert_coded, two_layers};
use crate::common::{ACCOUNT, OTHER_ACCOUNT};

fn request(id: AnimationId, pixels: Option<Vec<u16>>) -> DescribeRequest {
    DescribeRequest { id, pixels }
}

#[tokio::test]
async fn a_view_without_pixels_holds_the_structure_only() {
    let fixture = Fixture::blank(8, 6).await;

    let view = fixture
        .editing
        .describe(&ACCOUNT, request(fixture.id, None));

    let view = view.await.unwrap();
    assert_eq!(
        (view.title.as_str(), view.width, view.height),
        ("Mascot", 8, 6)
    );
    assert_eq!(view.version, fixture.version().await);
    assert_eq!(view.palette, DEFAULT_PALETTE);
    let layer = LayerView {
        id: 1,
        name: "Layer 1".to_owned(),
        visible: true,
    };
    assert_eq!(view.layers, [layer]);
    let frame = FrameView {
        position: 0,
        duration_ms: DEFAULT_FRAME_DURATION_MS,
    };
    assert_eq!(view.frames, [frame]);
    assert!(view.tags.is_empty() && view.grids.is_empty());
    let json = serde_json::to_value(&view).unwrap();
    assert!(json.get("grids").is_none());
    assert_eq!(json["palette"][1], json!("#000000ff"));
    assert_eq!(
        json["frames"][0],
        json!({ "position": 0, "duration_ms": 100 })
    );
}

#[tokio::test]
async fn pixels_are_the_composites_of_the_frames_asked_for() {
    let mut animation = two_layers();
    let bottom = life_pixel_core::edit::Operation::Rectangle {
        layer: life_pixel_core::LayerId::new(1),
        frame: animation.frames()[0].id(),
        from: life_pixel_core::Point { x: 0, y: 0 },
        to: life_pixel_core::Point { x: 3, y: 3 },
        index: 2,
        filled: true,
    };
    life_pixel_core::edit::apply(&mut animation, &bottom).unwrap();
    let fixture = Fixture::imported(&animation).await;
    fixture
        .write(0, &["1...", ".1..", "..1.", "...1"])
        .await
        .unwrap();

    let view = fixture.describe(&[0]).await;

    assert_eq!(view.grids.len(), 1);
    assert_eq!(view.grids[0].frame, 0);
    assert_eq!(view.grids[0].rows, ["1222", "2122", "2212", "2221"]);
}

#[tokio::test]
async fn a_frame_beyond_the_last_is_not_found() {
    let fixture = Fixture::blank(4, 4).await;

    let error = fixture
        .editing
        .describe(&ACCOUNT, request(fixture.id, Some(vec![0, 1])));

    assert_coded(&error.await.unwrap_err(), "edit.frame_not_found", json!({}));
}

#[tokio::test]
async fn another_owner_or_a_missing_animation_is_not_found() {
    let fixture = Fixture::blank(4, 4).await;
    let missing = AnimationId::from_uuid(Uuid::from_u128(0xdead));

    for (owner, id) in [
        (OTHER_ACCOUNT, fixture.id),
        (Owner::Local, fixture.id),
        (ACCOUNT, missing),
    ] {
        let error = fixture.editing.describe(&owner, request(id, None));
        let error = error.await.unwrap_err();
        assert_coded(&error, "library.animation_not_found", json!({}));
    }
}
