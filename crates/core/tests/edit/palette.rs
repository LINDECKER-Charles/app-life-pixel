//! Palette entries — set, add, remove, move, every cel remapped — and the title.

use life_pixel_core::Rgba;
use life_pixel_core::edit::{self, Operation};
use life_pixel_core::limits::MAX_PALETTE_ENTRIES;

use crate::support::{FRAME, LAYER, apply, assert_refused, rows, sketch};

const ORANGE: Rgba = Rgba::from_u32(0xff7f_27c0);

#[test]
fn an_entry_other_than_0_takes_a_new_colour() {
    let mut animation = sketch(&[&["1."]]);
    let set = |index| Operation::SetPaletteEntry {
        index,
        color: ORANGE,
    };
    apply(&mut animation, &set(1));
    assert_eq!(animation.palette().get(1), Some(ORANGE));
    assert_refused(&mut animation, &set(0), "document.palette");
    assert_refused(&mut animation, &set(16), "document.palette");
}

#[test]
fn an_entry_is_appended_until_the_palette_is_full() {
    let mut animation = sketch(&[&["1."]]);
    let add = Operation::AddPaletteEntry { color: ORANGE };
    apply(&mut animation, &add);
    assert_eq!(animation.palette().len(), 17);
    assert_eq!(animation.palette().get(16), Some(ORANGE));
    while animation.palette().len() < MAX_PALETTE_ENTRIES {
        edit::apply(&mut animation, &add).unwrap();
    }
    assert_refused(&mut animation, &add, "edit.palette_full");
}

#[test]
fn a_removed_entry_leaves_0_and_moves_higher_indices_down() {
    let mut animation = sketch(&[&["123", "3.."]]);
    let grey = animation.palette().get(3);
    let remove = |index| Operation::RemovePaletteEntry { index };
    apply(&mut animation, &remove(2));
    assert_eq!(rows(&animation, LAYER, FRAME), ["1.2", "2.."]);
    assert_eq!(animation.palette().len(), 15);
    assert_eq!(animation.palette().get(2), grey);
    assert_refused(&mut animation, &remove(0), "document.palette");
    assert_refused(&mut animation, &remove(15), "document.palette");
}

#[test]
fn a_moved_entry_takes_its_pixels_along() {
    let mut animation = sketch(&[&["123", "4.."]]);
    let black = animation.palette().get(1);
    let move_entry = |from, to| Operation::MovePaletteEntry { from, to };
    apply(&mut animation, &move_entry(1, 3));
    assert_eq!(rows(&animation, LAYER, FRAME), ["312", "4.."]);
    assert_eq!(animation.palette().get(3), black);
    apply(&mut animation, &move_entry(3, 1));
    assert_eq!(rows(&animation, LAYER, FRAME), ["123", "4.."]);
    assert_refused(&mut animation, &move_entry(0, 2), "document.palette");
    assert_refused(&mut animation, &move_entry(2, 0), "document.palette");
    assert_refused(&mut animation, &move_entry(2, 16), "document.palette");
}

#[test]
fn the_title_is_a_name() {
    let mut animation = sketch(&[&["1."]]);
    let set_title = |title: &str| Operation::SetTitle {
        title: title.to_owned(),
    };
    apply(&mut animation, &set_title("  Mascot "));
    assert_eq!(animation.title().as_str(), "Mascot");
    assert_refused(&mut animation, &set_title("  "), "document.name");
}
