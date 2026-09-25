//! PNG images and sprite sheets: every colour type reduced to the palette, the size limits
//! checked before any pixel is decoded, and the frames a sheet inserts.

use life_pixel_core::edit::{self, Operation, TagSpec};
use life_pixel_core::limits::{IMPORT_MAX_BYTES, IMPORT_MAX_SIDE, MAX_FRAMES};
use life_pixel_core::{Animation, FrameId, LayerId, LoopMode, Point};
use png::{BitDepth, ColorType};

use crate::support::png::{PngSpec, encode, header_only, rgba};
use crate::support::{FRAME, LAYER, apply, assert_refused, blank, point, rows, sketch};

const BLACK: [u8; 4] = [0, 0, 0, 255];
const WHITE: [u8; 4] = [255, 255, 255, 255];
const CLEAR: [u8; 4] = [255, 255, 255, 0];

fn import_image(png: Vec<u8>, at: Point) -> Operation {
    Operation::ImportImage {
        layer: LAYER,
        frame: FRAME,
        png,
        at,
    }
}

/// Imports `spec`, 4 × 1 pixels, on a blank 4 × 1 animation; returns the row it leaves.
fn imported_row(spec: &PngSpec) -> String {
    let mut animation = blank(4, 1);
    apply(&mut animation, &import_image(encode(spec), point(0, 0)));
    rows(&animation, LAYER, FRAME).remove(0)
}

fn spec(color: ColorType, depth: BitDepth, data: &[u8]) -> PngSpec<'_> {
    PngSpec {
        width: 4,
        height: 1,
        color,
        depth,
        data,
        palette: None,
        transparency: None,
    }
}

#[test]
fn a_paletted_png_with_transparency_is_reduced_to_the_palette() {
    let palette = [0, 0, 0, 255, 255, 255, 0xed, 0x1c, 0x24, 0x22, 0xb1, 0x4c];
    let data = [0b0001_1011];
    let paletted = PngSpec {
        palette: Some(&palette),
        transparency: Some(&[255, 255, 255, 0]),
        ..spec(ColorType::Indexed, BitDepth::Two, &data)
    };
    assert_eq!(imported_row(&paletted), "126.");
}

#[test]
fn grey_pngs_of_8_and_16_bits_take_the_nearest_entry() {
    let eight = [0, 255, 127, 200];
    assert_eq!(
        imported_row(&spec(ColorType::Grayscale, BitDepth::Eight, &eight)),
        "1234"
    );
    let sixteen = [0, 0, 255, 255, 127, 1, 200, 0];
    assert_eq!(
        imported_row(&spec(ColorType::Grayscale, BitDepth::Sixteen, &sixteen)),
        "1234"
    );
    let with_alpha = [255, 255, 255, 127, 0, 128, 0, 0];
    let grey_alpha = spec(ColorType::GrayscaleAlpha, BitDepth::Eight, &with_alpha);
    assert_eq!(imported_row(&grey_alpha), "2.1.");
}

#[test]
fn colour_pngs_of_16_bits_lose_their_low_bytes() {
    let red = [0xed, 0x00, 0x1c, 0x00, 0x24, 0x00];
    let rgb: Vec<u8> = [red, [0; 6], [0xff; 6], red].concat();
    assert_eq!(
        imported_row(&spec(ColorType::Rgb, BitDepth::Sixteen, &rgb)),
        "6126"
    );
    let half_clear = [0xed, 0x00, 0x1c, 0x00, 0x24, 0x00, 0x7f, 0xff];
    let opaque = [0x22, 0, 0xb1, 0, 0x4c, 0, 0x80, 0];
    let rgba: Vec<u8> = [half_clear, opaque, half_clear, opaque].concat();
    assert_eq!(
        imported_row(&spec(ColorType::Rgba, BitDepth::Sixteen, &rgba)),
        ".9.9"
    );
}

#[test]
fn transparent_pixels_leave_the_cel_as_it_was_and_the_image_is_clipped() {
    let mut animation = sketch(&[&["55", "55"]]);
    apply(
        &mut animation,
        &import_image(rgba(2, 1, &[WHITE, CLEAR]), point(0, 1)),
    );
    assert_eq!(rows(&animation, LAYER, FRAME), ["55", "25"]);
    let mut animation = blank(3, 3);
    let square = rgba(2, 2, &[BLACK; 4]);
    apply(&mut animation, &import_image(square, point(-1, 1)));
    assert_eq!(rows(&animation, LAYER, FRAME), ["...", "1..", "1.."]);
}

#[test]
fn a_file_or_side_beyond_the_limits_is_refused_before_its_pixels_are_decoded() {
    let mut animation = blank(2, 2);
    let too_many_bytes = vec![0; IMPORT_MAX_BYTES + 1];
    let at = point(0, 0);
    assert_refused(
        &mut animation,
        &import_image(too_many_bytes, at),
        "import.image_too_large",
    );
    // The header alone claims 40 GB of pixels and no pixel data follows: had the decoder gone
    // past the header, it would have failed as malformed, not as too large.
    let huge = header_only(100_000, 100_000);
    assert_refused(
        &mut animation,
        &import_image(huge, at),
        "import.image_too_large",
    );
    let wide = rgba(
        IMPORT_MAX_SIDE + 1,
        1,
        &vec![BLACK; IMPORT_MAX_SIDE as usize + 1],
    );
    assert_refused(
        &mut animation,
        &import_image(wide, at),
        "import.image_too_large",
    );
    let widest = rgba(IMPORT_MAX_SIDE, 1, &vec![BLACK; IMPORT_MAX_SIDE as usize]);
    apply(&mut animation, &import_image(widest, at));
}

#[test]
fn a_file_that_is_not_a_whole_png_is_malformed_and_references_come_first() {
    let mut animation = blank(2, 2);
    let at = point(0, 0);
    let text = b"not a png".to_vec();
    assert_refused(
        &mut animation,
        &import_image(text.clone(), at),
        "import.image_malformed",
    );
    let truncated = header_only(2, 2);
    assert_refused(
        &mut animation,
        &import_image(truncated, at),
        "import.image_malformed",
    );
    let elsewhere = Operation::ImportImage {
        layer: LayerId::new(9),
        frame: FRAME,
        png: text,
        at,
    };
    assert_refused(&mut animation, &elsewhere, "edit.layer_not_found");
}

fn sheet(png: Vec<u8>, position: u32, cell_width: u32) -> Operation {
    Operation::ImportSpriteSheet {
        layer: LAYER,
        position,
        png,
        cell_width,
        cell_height: 1,
        duration_ms: 90,
    }
}

/// Frames 2 and 3, 2 × 1 pixels, both in the tag `walk`.
fn walk() -> Animation {
    let mut animation = blank(2, 1);
    let add = Operation::AddFrame {
        position: 1,
        duration_ms: 100,
    };
    apply(&mut animation, &add);
    let walk = TagSpec {
        name: "walk".to_owned(),
        first: 0,
        last: 1,
        loop_mode: LoopMode::Loop,
    };
    apply(&mut animation, &Operation::AddTag { tag: walk });
    animation
}

#[test]
fn a_sheet_is_cut_row_by_row_into_new_frames_without_its_trailing_empty_cells() {
    let mut animation = walk();
    let cells = [BLACK, WHITE, CLEAR, CLEAR, WHITE, CLEAR, CLEAR, CLEAR];
    apply(&mut animation, &sheet(rgba(4, 2, &cells), 1, 2));
    let ids: Vec<u32> = animation
        .frames()
        .iter()
        .map(|frame| frame.id().get())
        .collect();
    assert_eq!(ids, [2, 4, 5, 6, 3]);
    assert!(
        animation.frames()[1..4]
            .iter()
            .all(|frame| frame.duration_ms() == 90)
    );
    assert_eq!(rows(&animation, LAYER, FrameId::new(4)), ["12"]);
    assert!(animation.cel(LAYER, FrameId::new(5)).is_none());
    assert_eq!(rows(&animation, LAYER, FrameId::new(6)), ["2."]);
    let tag = &animation.tags()[0];
    assert_eq!((tag.first(), tag.last()), (0, 4));
}

#[test]
fn a_sheet_needs_a_grid_that_fits_and_room_for_its_frames() {
    let mut animation = blank(2, 1);
    let image = rgba(4, 1, &[BLACK; 4]);
    assert_refused(
        &mut animation,
        &sheet(image.clone(), 0, 3),
        "import.sheet_grid",
    );
    assert_refused(
        &mut animation,
        &sheet(image.clone(), 0, 0),
        "import.sheet_grid",
    );
    assert_refused(
        &mut animation,
        &sheet(image, 2, 2),
        "edit.position_out_of_range",
    );
    let long = rgba(MAX_FRAMES as u32, 1, &vec![BLACK; MAX_FRAMES]);
    assert_refused(&mut animation, &sheet(long, 0, 1), "document.frame_count");
    let clear = rgba(4, 1, &[CLEAR; 4]);
    let unchanged = edit::apply(&mut animation, &sheet(clear, 0, 2)).unwrap();
    assert!(unchanged.is_empty(), "a sheet of empty cells adds nothing");
}
