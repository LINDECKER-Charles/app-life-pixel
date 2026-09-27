//! Frames — added, duplicated, deleted, moved, timed — with the tag rules they follow, and the
//! tag operations.

use life_pixel_core::edit::{self, EditError, Operation, TagSpec};
use life_pixel_core::limits::{MAX_FRAMES, MAX_TAGS};
use life_pixel_core::{Animation, FrameId, LoopMode};

use crate::support::{LAYER, apply, assert_refused, blank, point, rows};

/// A 2 × 1 animation of `count` frames with ids 2, 3, …; the first frame's pixel 0 is painted.
fn timeline(count: u32) -> Animation {
    let mut animation = blank(2, 1);
    for position in 1..count {
        let add = Operation::AddFrame {
            position,
            duration_ms: 100 + position,
        };
        edit::apply(&mut animation, &add).unwrap();
    }
    let paint = Operation::PaintStroke {
        layer: LAYER,
        frame: FrameId::new(2),
        points: vec![point(0, 0)],
        index: 1,
    };
    edit::apply(&mut animation, &paint).unwrap();
    animation
}

fn tag(name: &str, first: u32, last: u32) -> TagSpec {
    TagSpec {
        name: name.to_owned(),
        first,
        last,
        loop_mode: LoopMode::Loop,
    }
}

fn with_tags(mut animation: Animation, tags: &[(&str, u32, u32)]) -> Animation {
    let tags = tags
        .iter()
        .map(|&(name, first, last)| tag(name, first, last));
    let replace = Operation::ReplaceTags {
        tags: tags.collect(),
    };
    apply(&mut animation, &replace);
    animation
}

fn tag_ranges(animation: &Animation) -> Vec<(String, u16, u16)> {
    let tags = animation.tags().iter();
    tags.map(|tag| (tag.name().to_string(), tag.first(), tag.last()))
        .collect()
}

fn ranges(expected: &[(&str, u16, u16)]) -> Vec<(String, u16, u16)> {
    let expected = expected.iter();
    expected
        .map(|&(name, first, last)| (name.to_owned(), first, last))
        .collect()
}

fn frame_ids(animation: &Animation) -> Vec<u32> {
    let frames = animation.frames().iter();
    frames.map(|frame| frame.id().get()).collect()
}

#[test]
fn an_inserted_frame_moves_later_tags_and_extends_those_it_spans() {
    let tags = [("before", 0, 1), ("after", 2, 3), ("across", 1, 2)];
    let mut animation = with_tags(timeline(4), &tags);
    let add = Operation::AddFrame {
        position: 2,
        duration_ms: 80,
    };
    apply(&mut animation, &add);
    assert_eq!(frame_ids(&animation), [2, 3, 6, 4, 5]);
    assert_eq!(animation.frames()[2].duration_ms(), 80);
    assert_eq!(animation.cels().len(), 1, "the new frame's cels are empty");
    let expected = [("before", 0, 1), ("after", 3, 4), ("across", 1, 3)];
    assert_eq!(tag_ranges(&animation), ranges(&expected));
}

#[test]
fn a_new_frame_needs_a_position_a_duration_and_room() {
    let mut animation = timeline(2);
    let add = |position, duration_ms| Operation::AddFrame {
        position,
        duration_ms,
    };
    assert_refused(&mut animation, &add(3, 100), "edit.position_out_of_range");
    assert_refused(&mut animation, &add(0, 9), "document.frame_duration");
    assert_refused(&mut animation, &add(0, 70_000), "document.frame_duration");
    while animation.frames().len() < MAX_FRAMES {
        edit::apply(&mut animation, &add(0, 100)).unwrap();
    }
    assert_refused(&mut animation, &add(0, 100), "document.frame_count");
}

#[test]
fn a_duplicate_goes_right_after_its_frame_with_its_cels() {
    let mut animation = with_tags(timeline(3), &[("start", 0, 0), ("end", 1, 2)]);
    let duplicate = Operation::DuplicateFrame {
        frame: FrameId::new(2),
    };
    apply(&mut animation, &duplicate);
    assert_eq!(frame_ids(&animation), [2, 5, 3, 4]);
    assert_eq!(rows(&animation, LAYER, FrameId::new(5)), ["1."]);
    assert_eq!(animation.frames()[1].duration_ms(), 100);
    assert_eq!(
        tag_ranges(&animation),
        ranges(&[("start", 0, 0), ("end", 2, 3)])
    );
}

#[test]
fn a_deleted_frame_mirrors_an_insertion_and_takes_a_tag_of_that_frame_alone() {
    let tags = [
        ("before", 0, 1),
        ("alone", 2, 2),
        ("across", 1, 3),
        ("after", 3, 3),
    ];
    let mut animation = with_tags(timeline(4), &tags);
    let delete = |id| Operation::DeleteFrame {
        frame: FrameId::new(id),
    };
    apply(&mut animation, &delete(4));
    assert_eq!(frame_ids(&animation), [2, 3, 5]);
    let expected = [("before", 0, 1), ("across", 1, 2), ("after", 2, 2)];
    assert_eq!(tag_ranges(&animation), ranges(&expected));
    apply(&mut animation, &delete(2));
    assert!(animation.cels().is_empty(), "its cels go too");
    assert_refused(&mut animation, &delete(2), "edit.frame_not_found");
    let mut single = timeline(1);
    assert_refused(&mut single, &delete(2), "edit.last_frame");
}

#[test]
fn a_moved_frame_leaves_the_tags_in_place() {
    let mut animation = with_tags(timeline(3), &[("start", 0, 0)]);
    let move_frame = |position| Operation::MoveFrame {
        frame: FrameId::new(2),
        position,
    };
    apply(&mut animation, &move_frame(2));
    assert_eq!(frame_ids(&animation), [3, 4, 2]);
    assert_eq!(tag_ranges(&animation), ranges(&[("start", 0, 0)]));
    assert_refused(&mut animation, &move_frame(3), "edit.position_out_of_range");
}

#[test]
fn a_frame_duration_stays_in_its_range() {
    let mut animation = timeline(2);
    let set = |id, duration_ms| Operation::SetFrameDuration {
        frame: FrameId::new(id),
        duration_ms,
    };
    apply(&mut animation, &set(3, 65_535));
    assert_eq!(animation.frames()[1].duration_ms(), 65_535);
    assert_refused(&mut animation, &set(3, 65_536), "document.frame_duration");
    assert_refused(&mut animation, &set(9, 100), "edit.frame_not_found");
}

#[test]
fn tags_are_added_updated_and_deleted_by_name() {
    let mut animation = timeline(3);
    apply(
        &mut animation,
        &Operation::AddTag {
            tag: tag("idle", 0, 1),
        },
    );
    let update = Operation::UpdateTag {
        name: "idle".to_owned(),
        tag: TagSpec {
            loop_mode: LoopMode::Once,
            ..tag("rest", 1, 2)
        },
    };
    apply(&mut animation, &update);
    assert_eq!(tag_ranges(&animation), ranges(&[("rest", 1, 2)]));
    assert_eq!(animation.tags()[0].loop_mode(), LoopMode::Once);
    let delete = |name: &str| Operation::DeleteTag {
        name: name.to_owned(),
    };
    let error = edit::apply(&mut animation, &delete("idle")).unwrap_err();
    assert_eq!(
        (error.code(), error.params()["name"].as_str()),
        ("edit.tag_not_found", Some("idle"))
    );
    apply(&mut animation, &delete("rest"));
    assert!(animation.tags().is_empty());
}

#[test]
fn a_tag_keeps_the_rules_of_the_model() {
    let mut animation = with_tags(timeline(3), &[("idle", 0, 0)]);
    let add = |spec: TagSpec| Operation::AddTag { tag: spec };
    let refusals = [
        (tag("idle", 1, 1), "idle"),
        (tag("late", 2, 3), "late"),
        (tag("back", 2, 1), "back"),
        (tag("Bad name", 0, 0), "Bad name"),
        (tag("huge", 0, 70_000), "huge"),
    ];
    for (spec, name) in refusals {
        let error = edit::apply(&mut animation, &add(spec.clone())).unwrap_err();
        let expected = EditError::Document(life_pixel_core::DocumentError::Tag {
            name: name.to_owned(),
        });
        assert_eq!(error, expected);
        assert_refused(&mut animation, &add(spec), "document.tag");
    }
    let too_many = (0..=MAX_TAGS).map(|number| tag(&format!("t{number}"), 0, 0));
    let replace = Operation::ReplaceTags {
        tags: too_many.collect(),
    };
    assert_refused(&mut animation, &replace, "document.tag_count");
}
