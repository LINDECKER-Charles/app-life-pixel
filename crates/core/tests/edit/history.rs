//! The history: undo and redo, the saved state across them, failed and empty operations, and
//! the step and byte limits.

use life_pixel_core::edit::{History, Operation};
use life_pixel_core::limits::{HISTORY_MAX_BYTES, HISTORY_MAX_STEPS};
use life_pixel_core::{Animation, LayerId};

use crate::support::{FRAME, LAYER, blank, document, point};

fn dot(x: i32, y: i32) -> Operation {
    Operation::PaintStroke {
        layer: LAYER,
        frame: FRAME,
        points: vec![point(x, y)],
        index: 1,
    }
}

/// Undoes every step; returns how many there were.
fn undo_all(history: &mut History, animation: &mut Animation) -> usize {
    let mut count = 0;
    while history.undo(animation) {
        count += 1;
    }
    count
}

#[test]
fn undo_and_redo_walk_the_steps_and_a_new_step_clears_redo() {
    let mut animation = blank(3, 1);
    let mut history = History::default();
    let states: Vec<String> = (0..3)
        .map(|x| {
            let before = document(&animation);
            history.apply(&mut animation, dot(x, 0)).unwrap();
            before
        })
        .collect();
    let last = document(&animation);
    assert!(history.can_undo() && !history.can_redo());
    for state in states.iter().rev() {
        assert!(history.undo(&mut animation));
        assert_eq!(&document(&animation), state);
    }
    assert!(!history.undo(&mut animation) && history.can_redo());
    while history.redo(&mut animation) {}
    assert_eq!(document(&animation), last);
    history.undo(&mut animation);
    history.apply(&mut animation, dot(2, 0)).unwrap();
    assert!(!history.can_redo() && !history.redo(&mut animation));
}

#[test]
fn a_failed_or_empty_operation_leaves_the_animation_and_the_history_as_they_were() {
    let mut animation = blank(2, 1);
    let mut history = History::default();
    history.apply(&mut animation, dot(0, 0)).unwrap();
    history.apply(&mut animation, dot(1, 0)).unwrap();
    history.undo(&mut animation);
    let before = document(&animation);
    let missing = Operation::DeleteLayer {
        layer: LayerId::new(9),
    };
    let error = history.apply(&mut animation, missing).unwrap_err();
    assert_eq!(error.code(), "edit.layer_not_found");
    history.apply(&mut animation, dot(0, 0)).unwrap();
    assert_eq!(document(&animation), before);
    assert!(history.can_redo(), "neither cleared the redo stack");
    assert_eq!(undo_all(&mut history, &mut animation), 1);
}

#[test]
fn the_saved_state_is_found_again_across_undo_and_redo() {
    let mut animation = blank(2, 1);
    let mut history = History::default();
    assert!(history.is_saved(), "a new history starts saved");
    history.apply(&mut animation, dot(0, 0)).unwrap();
    assert!(!history.is_saved());
    history.undo(&mut animation);
    assert!(history.is_saved());
    history.redo(&mut animation);
    history.mark_saved();
    history.apply(&mut animation, dot(1, 0)).unwrap();
    assert!(!history.is_saved());
    history.undo(&mut animation);
    assert!(history.is_saved());
    history.undo(&mut animation);
    assert!(!history.is_saved());
    history.apply(&mut animation, dot(1, 0)).unwrap();
    history.undo(&mut animation);
    assert!(
        !history.is_saved(),
        "the saved state left with the redo stack"
    );
}

#[test]
fn the_oldest_steps_go_beyond_the_step_limit() {
    let mut animation = blank(16, 16);
    let mut history = History::default();
    let extra = 5;
    let mut after_extra = String::new();
    for step in 0..HISTORY_MAX_STEPS + extra {
        let x = i32::try_from(step % 16).unwrap();
        let y = i32::try_from(step / 16).unwrap();
        history.apply(&mut animation, dot(x, y)).unwrap();
        if step + 1 == extra {
            after_extra = document(&animation);
        }
    }
    assert_eq!(undo_all(&mut history, &mut animation), HISTORY_MAX_STEPS);
    assert_eq!(document(&animation), after_extra);
    assert!(
        !history.is_saved(),
        "the saved state was dropped with the oldest steps"
    );
}

/// A `side` × `side` animation of `layer_count` layers, each filled with index 1.
fn filled_layers(side: usize, layer_count: u32) -> (Animation, History) {
    let mut animation = blank(side, side);
    let mut history = History::default();
    for position in 1..layer_count {
        let add = Operation::AddLayer {
            position,
            name: "Layer".to_owned(),
        };
        history.apply(&mut animation, add).unwrap();
    }
    for layer in animation.layers().to_vec() {
        let fill = Operation::Fill {
            layer: layer.id(),
            frame: FRAME,
            at: point(0, 0),
            index: 1,
        };
        history.apply(&mut animation, fill).unwrap();
    }
    (animation, history)
}

#[test]
fn the_oldest_steps_go_beyond_the_byte_limit() {
    let (side, layer_count) = (512, 4);
    let (mut animation, mut history) = filled_layers(side, layer_count);
    let applied = 70;
    for step in 0..applied {
        let (from, to) = if step % 2 == 0 { (1, 2) } else { (2, 1) };
        let swap = Operation::MovePaletteEntry { from, to };
        history.apply(&mut animation, swap).unwrap();
    }
    let step_bytes = side * side * usize::try_from(layer_count).unwrap();
    let undone = undo_all(&mut history, &mut animation);
    assert!(undone > 0);
    assert!(
        undone <= HISTORY_MAX_BYTES / step_bytes,
        "{undone} steps kept"
    );
}
