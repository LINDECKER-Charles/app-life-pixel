//! The pixel tools: stroke, fill, line, rectangle and selection move, and their previews.

use std::collections::BTreeSet;

use life_pixel_core::edit::{self, Area, Operation};
use life_pixel_core::limits::STROKE_MAX_POINTS;
use life_pixel_core::{FrameId, LayerId, Point};

use crate::support::{FRAME, LAYER, apply, assert_refused, blank, document, point, rows, sketch};

fn stroke(points: Vec<Point>, index: u32) -> Operation {
    Operation::PaintStroke {
        layer: LAYER,
        frame: FRAME,
        points,
        index,
    }
}

fn line(from: Point, to: Point) -> Operation {
    Operation::Line {
        layer: LAYER,
        frame: FRAME,
        from,
        to,
        index: 1,
    }
}

fn rectangle(from: Point, to: Point, filled: bool) -> Operation {
    Operation::Rectangle {
        layer: LAYER,
        frame: FRAME,
        from,
        to,
        index: 1,
        filled,
    }
}

fn fill(layer: LayerId, frame: FrameId, at: Point) -> Operation {
    Operation::Fill {
        layer,
        frame,
        at,
        index: 2,
    }
}

fn move_selection(area: Area, offset: Point) -> Operation {
    Operation::MoveSelection {
        layer: LAYER,
        frame: FRAME,
        area,
        offset,
    }
}

/// The pixels of the layer's cel that are not 0.
fn painted(animation: &life_pixel_core::Animation) -> BTreeSet<(i32, i32)> {
    let rows = rows(animation, LAYER, FRAME);
    let mut pixels = BTreeSet::new();
    for (y, row) in (0..).zip(&rows) {
        for (x, character) in (0..).zip(row.chars()) {
            if character != '.' {
                pixels.insert((x, y));
            }
        }
    }
    pixels
}

#[test]
fn a_stroke_paints_each_point_and_the_lines_between_ignoring_the_outside() {
    let mut animation = blank(5, 3);
    let points = vec![point(-2, 0), point(2, 0), point(2, 2), point(9, 2)];
    apply(&mut animation, &stroke(points, 1));
    assert_eq!(rows(&animation, LAYER, FRAME), ["111..", "..1..", "..111"]);
    apply(&mut animation, &stroke(vec![point(4, 0)], 3));
    assert_eq!(rows(&animation, LAYER, FRAME), ["111.3", "..1..", "..111"]);
}

#[test]
fn the_eraser_is_a_stroke_of_index_0() {
    let mut animation = sketch(&[&["111", "111"]]);
    apply(&mut animation, &stroke(vec![point(0, 1), point(2, 1)], 0));
    assert_eq!(rows(&animation, LAYER, FRAME), ["111", "..."]);
    apply(&mut animation, &stroke(vec![point(0, 0), point(2, 0)], 0));
    assert!(
        animation.cel(LAYER, FRAME).is_none(),
        "a blank cel is not stored"
    );
}

#[test]
fn a_stroke_holds_at_most_its_limit_of_points() {
    let mut animation = blank(2, 2);
    apply(
        &mut animation,
        &stroke(vec![point(0, 0); STROKE_MAX_POINTS], 1),
    );
    let too_long = stroke(vec![point(1, 1); STROKE_MAX_POINTS + 1], 1);
    assert_refused(&mut animation, &too_long, "edit.stroke_too_long");
}

#[test]
fn references_are_checked_first_layer_then_frame_then_index() {
    let mut animation = blank(2, 2);
    let points = vec![point(0, 0); STROKE_MAX_POINTS + 1];
    let cases = [
        (LayerId::new(9), FrameId::new(9), 99, "edit.layer_not_found"),
        (LAYER, FrameId::new(9), 99, "edit.frame_not_found"),
        (LAYER, FRAME, 16, "document.palette"),
        (LAYER, FRAME, 1, "edit.stroke_too_long"),
    ];
    for (layer, frame, index, code) in cases {
        let operation = Operation::PaintStroke {
            layer,
            frame,
            points: points.clone(),
            index,
        };
        assert_refused(&mut animation, &operation, code);
    }
}

#[test]
fn a_fill_floods_the_4_connected_region_of_the_layer_own_cel() {
    let mut animation = sketch(&[&["..1.", ".1..", "1..."], &["....", "....", "...."]]);
    let (bottom, top, frame) = (LayerId::new(1), LayerId::new(2), FrameId::new(3));
    apply(&mut animation, &fill(bottom, frame, point(0, 0)));
    assert_eq!(rows(&animation, bottom, frame), ["221.", "21..", "1..."]);
    apply(&mut animation, &fill(top, frame, point(3, 2)));
    assert_eq!(rows(&animation, top, frame), ["2222", "2222", "2222"]);
}

#[test]
fn a_fill_follows_a_spiral_to_its_end() {
    let spiral = [
        ".........",
        "11111111.",
        ".......1.",
        ".11111.1.",
        ".1...1.1.",
        ".1.111.1.",
        ".1.....1.",
        ".1111111.",
        ".........",
    ];
    let mut animation = sketch(&[&spiral]);
    apply(&mut animation, &fill(LAYER, FRAME, point(4, 4)));
    let filled: Vec<String> = spiral.iter().map(|row| row.replace('.', "2")).collect();
    assert_eq!(rows(&animation, LAYER, FRAME), filled);
}

#[test]
fn a_fill_outside_the_canvas_is_refused_and_one_of_the_same_index_changes_nothing() {
    let mut animation = sketch(&[&["22", "2."]]);
    let outside = fill(LAYER, FRAME, point(2, 0));
    assert_refused(&mut animation, &outside, "edit.out_of_canvas");
    let same = edit::apply(&mut animation, &fill(LAYER, FRAME, point(0, 0))).unwrap();
    assert!(same.is_empty());
}

#[test]
fn bresenham_lines_mirror_each_other_in_all_octants() {
    let first_octant = [(0, 0), (1, 0), (2, 1), (3, 1), (4, 2), (5, 2)];
    let center = 6;
    for is_steep in [false, true] {
        for (sign_x, sign_y) in [(1, 1), (-1, 1), (1, -1), (-1, -1)] {
            let place = |(x, y): (i32, i32)| {
                let (x, y) = if is_steep { (y, x) } else { (x, y) };
                (center + sign_x * x, center + sign_y * y)
            };
            let expected: BTreeSet<(i32, i32)> = first_octant.into_iter().map(place).collect();
            let (end_x, end_y) = place((5, 2));
            let (from, to) = (point(center, center), point(end_x, end_y));
            for (start, end) in [(from, to), (to, from)] {
                let mut animation = blank(13, 13);
                apply(&mut animation, &line(start, end));
                assert_eq!(painted(&animation), expected, "{start:?} to {end:?}");
            }
        }
    }
}

#[test]
fn a_line_between_far_points_is_clipped_to_the_canvas() {
    let mut animation = blank(4, 4);
    apply(
        &mut animation,
        &line(point(i32::MIN, i32::MIN), point(i32::MAX, i32::MAX)),
    );
    assert_eq!(
        rows(&animation, LAYER, FRAME),
        ["1...", ".1..", "..1.", "...1"]
    );
    let mut animation = blank(4, 4);
    apply(&mut animation, &line(point(2, -100), point(2, 100)));
    assert_eq!(
        rows(&animation, LAYER, FRAME),
        ["..1.", "..1.", "..1.", "..1."]
    );
}

#[test]
fn a_rectangle_takes_its_corners_in_any_order_and_is_clipped() {
    let mut animation = blank(5, 4);
    apply(&mut animation, &rectangle(point(3, 2), point(0, 0), false));
    assert_eq!(
        rows(&animation, LAYER, FRAME),
        ["1111.", "1..1.", "1111.", "....."]
    );
    let mut animation = blank(4, 4);
    apply(&mut animation, &rectangle(point(1, 1), point(-3, 9), true));
    assert_eq!(
        rows(&animation, LAYER, FRAME),
        ["....", "11..", "11..", "11.."]
    );
    let mut animation = blank(4, 4);
    apply(
        &mut animation,
        &rectangle(point(-1, -1), point(2, 2), false),
    );
    assert_eq!(
        rows(&animation, LAYER, FRAME),
        ["..1.", "..1.", "111.", "...."]
    );
}

#[test]
fn a_moved_selection_clears_its_area_then_pastes_all_but_index_0() {
    let mut animation = sketch(&[&["12..", "3...", "..44", "...5"]]);
    let area = Area {
        x: 0,
        y: 0,
        width: 2,
        height: 2,
    };
    apply(&mut animation, &move_selection(area, point(2, 2)));
    assert_eq!(
        rows(&animation, LAYER, FRAME),
        ["....", "....", "..12", "..35"]
    );
}

#[test]
fn a_selection_is_clipped_and_its_pixels_leaving_the_canvas_are_lost() {
    let mut animation = sketch(&[&["12.", "3.."]]);
    let area = Area {
        x: -1,
        y: -1,
        width: 3,
        height: 2,
    };
    apply(&mut animation, &move_selection(area, point(2, 1)));
    assert_eq!(rows(&animation, LAYER, FRAME), ["...", "3.1"]);
}

#[test]
fn a_preview_is_the_cel_apply_produces_and_changes_nothing() {
    let animation = sketch(&[&["12..", "3...", "..44", "...5"]]);
    let area = Area {
        x: 1,
        y: 1,
        width: 2,
        height: 2,
    };
    let operations = [
        stroke(vec![point(0, 3), point(3, 0)], 6),
        fill(LAYER, FRAME, point(3, 0)),
        line(point(0, 0), point(3, 3)),
        rectangle(point(0, 1), point(2, 3), true),
        move_selection(area, point(-1, 0)),
    ];
    let before = document(&animation);
    for operation in operations {
        let (layer, frame, cel) = edit::preview(&animation, &operation).unwrap().unwrap();
        assert_eq!(document(&animation), before, "{operation:?}");
        let mut applied = animation.clone();
        edit::apply(&mut applied, &operation).unwrap();
        assert_eq!((layer, frame), (LAYER, FRAME));
        assert_eq!(Some(&cel), applied.cel(layer, frame), "{operation:?}");
    }
}

#[test]
fn a_preview_fails_as_apply_would_and_is_none_for_other_operations() {
    let animation = blank(2, 2);
    let outside = fill(LAYER, FRAME, point(5, 5));
    let error = edit::preview(&animation, &outside).unwrap_err();
    assert_eq!(error.code(), "edit.out_of_canvas");
    let title = Operation::SetTitle {
        title: "Other".to_owned(),
    };
    assert_eq!(edit::preview(&animation, &title), Ok(None));
}
