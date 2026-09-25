//! Golden images: the RGBA renders of scripted sequences of operations, compared pixel for pixel
//! with `crates/core/tests/golden/`. `LP_UPDATE_GOLDEN=1 cargo test -p life-pixel-core` rewrites
//! them, for review in the diff.

use life_pixel_core::edit::{Area, History, Operation};
use life_pixel_core::render::rgba;
use life_pixel_core::{Animation, FrameId, LayerId, Point, Rgba};

use crate::support::golden::{Image, assert_golden};
use crate::support::{FRAME, LAYER, blank, point};

fn run(animation: &mut Animation, script: Vec<Operation>) {
    let mut history = History::default();
    for operation in script {
        history
            .apply(animation, operation.clone())
            .unwrap_or_else(|error| panic!("{operation:?}: {}", error.code()));
    }
}

fn render(animation: &Animation, frame: FrameId) -> Image {
    Image {
        width: u32::from(animation.width()),
        height: u32::from(animation.height()),
        pixels: rgba(animation, frame),
    }
}

/// A rectangle of `layer`; `paint` is its palette index and whether it is filled.
fn rectangle(layer: LayerId, corners: (Point, Point), paint: (u32, bool)) -> Operation {
    Operation::Rectangle {
        layer,
        frame: FRAME,
        from: corners.0,
        to: corners.1,
        index: paint.0,
        filled: paint.1,
    }
}

fn stroke(points: Vec<Point>, index: u32) -> Operation {
    Operation::PaintStroke {
        layer: LAYER,
        frame: FRAME,
        points,
        index,
    }
}

/// Two rectangles, a fill inside the outline and a line across.
fn shapes() -> Vec<Operation> {
    vec![
        rectangle(LAYER, (point(1, 1), point(10, 8)), (1, false)),
        rectangle(LAYER, (point(20, 6), point(13, 2)), (6, true)),
        Operation::Fill {
            layer: LAYER,
            frame: FRAME,
            at: point(5, 5),
            index: 8,
        },
        Operation::Line {
            layer: LAYER,
            frame: FRAME,
            from: point(0, 15),
            to: point(23, 9),
            index: 10,
        },
    ]
}

#[test]
fn every_tool_on_one_layer() {
    let mut animation = blank(24, 16);
    let mut script = shapes();
    script.extend([
        stroke(vec![point(2, 12), point(6, 10), point(9, 14)], 12),
        Operation::MoveSelection {
            layer: LAYER,
            frame: FRAME,
            area: Area {
                x: 13,
                y: 2,
                width: 4,
                height: 3,
            },
            offset: point(0, 8),
        },
        stroke(vec![point(15, 4), point(18, 4)], 0),
    ]);
    run(&mut animation, script);
    assert_golden("every-tool", &render(&animation, FRAME));
}

#[test]
fn layers_visibility_and_palette_changes() {
    let mut animation = blank(16, 16);
    let (top, hidden) = (LayerId::new(3), LayerId::new(4));
    let script = vec![
        rectangle(LAYER, (point(0, 0), point(15, 15)), (2, true)),
        Operation::AddLayer {
            position: 1,
            name: "Top".to_owned(),
        },
        rectangle(top, (point(4, 4), point(11, 11)), (6, true)),
        rectangle(top, (point(6, 6), point(9, 9)), (0, true)),
        Operation::AddLayer {
            position: 2,
            name: "Hidden".to_owned(),
        },
        rectangle(hidden, (point(0, 0), point(15, 15)), (1, true)),
        Operation::SetLayerVisibility {
            layer: hidden,
            visible: false,
        },
        Operation::SetPaletteEntry {
            index: 6,
            color: Rgba::from_u32(0x3f48_cc80),
        },
        Operation::RemovePaletteEntry { index: 1 },
        Operation::MovePaletteEntry { from: 1, to: 14 },
    ];
    run(&mut animation, script);
    assert_golden("layers-and-palette", &render(&animation, FRAME));
}

/// A 32 × 8 sheet of four 8 × 8 cells: a diagonal ramp, a checker, then two empty ones.
fn sheet() -> Vec<u8> {
    let (width, height) = (32, 8);
    let mut pixels = Vec::new();
    for y in 0..height {
        for x in 0..width {
            let pixel = match x / 8 {
                0 => [u8::try_from(x * 32 + y * 4).unwrap(), 64, 200, 255],
                1 if (x + y) % 2 == 0 => [250, 240, 10, 255],
                1 => [0, 0, 0, 90],
                _ => [0, 0, 0, 0],
            };
            pixels.push(pixel);
        }
    }
    crate::support::png::rgba(width, height, &pixels)
}

#[test]
fn a_sprite_sheet_and_an_image_imported() {
    let mut animation = blank(8, 8);
    let script = vec![
        Operation::ImportSpriteSheet {
            layer: LAYER,
            position: 1,
            png: sheet(),
            cell_width: 8,
            cell_height: 8,
            duration_ms: 100,
        },
        Operation::ImportImage {
            layer: LAYER,
            frame: FRAME,
            png: sheet(),
            at: point(-4, 2),
        },
    ];
    run(&mut animation, script);
    let frames: Vec<Image> = animation
        .frames()
        .iter()
        .map(|frame| render(&animation, frame.id()))
        .collect();
    assert_eq!(frames.len(), 3, "the empty cells at the end are dropped");
    assert_golden("sprite-sheet", &Image::side_by_side(&frames));
}
