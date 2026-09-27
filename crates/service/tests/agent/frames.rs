//! `edit_frames` and `set_tags`: frame edits in order, and what they do to the tags.

use life_pixel_core::LoopMode;
use life_pixel_core::edit::TagSpec;
use life_pixel_core::limits::{MAX_FRAME_DURATION_MS, MIN_FRAME_DURATION_MS};
use life_pixel_service::animation::{
    AnimationView, EditFramesRequest, EditingError, FrameEdit, SetTagsRequest,
};
use serde_json::json;

use super::support::{Fixture, assert_coded};
use crate::common::ACCOUNT;

async fn edit(fixture: &Fixture, edits: Vec<FrameEdit>) -> Result<AnimationView, EditingError> {
    let request = EditFramesRequest {
        id: fixture.id,
        edits,
    };
    fixture.editing.edit_frames(&ACCOUNT, request).await
}

async fn set_tags(fixture: &Fixture, tags: Vec<TagSpec>) -> Result<AnimationView, EditingError> {
    let request = SetTagsRequest {
        id: fixture.id,
        tags,
    };
    fixture.editing.set_tags(&ACCOUNT, request).await
}

fn tag(name: &str, (first, last): (u32, u32), loop_mode: LoopMode) -> TagSpec {
    TagSpec {
        name: name.to_owned(),
        first,
        last,
        loop_mode,
    }
}

fn durations(view: &AnimationView) -> Vec<u16> {
    view.frames.iter().map(|frame| frame.duration_ms).collect()
}

#[tokio::test]
async fn frame_edits_apply_in_order_by_position() {
    let fixture = Fixture::blank(2, 1).await;
    fixture.write(0, &["1."]).await.unwrap();
    let edits = vec![
        FrameEdit::Add {
            position: None,
            duration_ms: Some(200),
        },
        FrameEdit::Add {
            position: Some(0),
            duration_ms: None,
        },
        FrameEdit::SetDuration {
            frame: 2,
            duration_ms: 300,
        },
        FrameEdit::Duplicate { frame: 1 },
        FrameEdit::Move {
            frame: 3,
            position: 0,
        },
        FrameEdit::Delete { frame: 1 },
    ];

    let view = edit(&fixture, edits).await.unwrap();

    assert_eq!(durations(&view), [300, 100, 100]);
    let grids = fixture.describe(&[0, 1, 2]).await.grids;
    let rows: Vec<&str> = grids.iter().map(|grid| grid.rows[0].as_str()).collect();
    assert_eq!(rows, ["..", "1.", "1."]);
}

#[tokio::test]
async fn tags_follow_the_frames_as_core_moves_them() {
    let fixture = Fixture::blank(1, 1).await;
    edit(
        &fixture,
        vec![
            FrameEdit::Add {
                position: None,
                duration_ms: None
            };
            2
        ],
    )
    .await
    .unwrap();
    let tags = vec![
        tag("walk", (0, 1), LoopMode::Loop),
        tag("jump", (2, 2), LoopMode::Once),
    ];
    assert_eq!(set_tags(&fixture, tags.clone()).await.unwrap().tags, tags);

    let edits = vec![
        FrameEdit::Delete { frame: 2 },
        FrameEdit::Add {
            position: Some(0),
            duration_ms: None,
        },
        FrameEdit::Duplicate { frame: 1 },
    ];
    let view = edit(&fixture, edits).await.unwrap();

    assert_eq!(view.tags, [tag("walk", (1, 3), LoopMode::Loop)]);
}

#[tokio::test]
async fn a_refused_frame_edit_leaves_the_animation_as_it_was() {
    let fixture = Fixture::blank(1, 1).await;
    let before = fixture.version().await;
    let add = FrameEdit::Add {
        position: None,
        duration_ms: None,
    };

    let missing = edit(&fixture, vec![add.clone(), FrameEdit::Delete { frame: 2 }]).await;
    let last = edit(&fixture, vec![FrameEdit::Delete { frame: 0 }]).await;
    let short = FrameEdit::SetDuration {
        frame: 0,
        duration_ms: 5,
    };
    let duration = edit(&fixture, vec![add, short]).await;

    assert_coded(&missing.unwrap_err(), "edit.frame_not_found", json!({}));
    assert_coded(&last.unwrap_err(), "edit.last_frame", json!({}));
    let params = json!({ "min": MIN_FRAME_DURATION_MS, "max": MAX_FRAME_DURATION_MS });
    assert_coded(&duration.unwrap_err(), "document.frame_duration", params);
    assert_eq!(fixture.version().await, before);
}

#[tokio::test]
async fn tags_that_break_the_rules_are_refused() {
    let fixture = Fixture::blank(1, 1).await;

    let beyond = set_tags(&fixture, vec![tag("idle", (0, 1), LoopMode::Loop)]).await;
    let twice = vec![tag("idle", (0, 0), LoopMode::Loop); 2];
    let duplicate = set_tags(&fixture, twice).await;

    assert_coded(
        &beyond.unwrap_err(),
        "document.tag",
        json!({ "name": "idle" }),
    );
    assert_coded(
        &duplicate.unwrap_err(),
        "document.tag",
        json!({ "name": "idle" }),
    );
    assert!(
        set_tags(&fixture, Vec::new())
            .await
            .unwrap()
            .tags
            .is_empty()
    );
}

#[tokio::test]
async fn frame_edits_read_as_the_tool_sends_them() {
    let edits = json!([
        { "op": "add" },
        { "op": "add", "position": 0, "duration_ms": 50 },
        { "op": "duplicate", "frame": 1 },
        { "op": "delete", "frame": 0 },
        { "op": "move", "frame": 0, "position": 1 },
        { "op": "set_duration", "frame": 1, "duration_ms": 80 },
    ]);

    let edits: Vec<FrameEdit> = serde_json::from_value(edits).unwrap();

    let add = FrameEdit::Add {
        position: None,
        duration_ms: None,
    };
    assert_eq!(edits[0], add);
    let set = FrameEdit::SetDuration {
        frame: 1,
        duration_ms: 80,
    };
    assert_eq!(edits[5], set);
}
