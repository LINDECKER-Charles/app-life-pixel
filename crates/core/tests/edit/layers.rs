//! Layers: added, deleted with their cels, moved, renamed, shown and hidden.

use life_pixel_core::edit::{self, EditError, Operation};
use life_pixel_core::limits::MAX_LAYERS;
use life_pixel_core::{Animation, FrameId, LayerId};

use crate::support::{apply, assert_refused, sketch};

const FRAME: FrameId = FrameId::new(3);

/// Layer 1 is `12`, layer 2 `.3`, both on frame 3.
fn two_layers() -> Animation {
    sketch(&[&["12"], &[".3"]])
}

fn layer_ids(animation: &Animation) -> Vec<u32> {
    let layers = animation.layers().iter();
    layers.map(|layer| layer.id().get()).collect()
}

fn add(position: u32, name: &str) -> Operation {
    Operation::AddLayer {
        position,
        name: name.to_owned(),
    }
}

#[test]
fn a_new_layer_takes_the_next_id_at_its_position_from_the_bottom() {
    let mut animation = two_layers();
    apply(&mut animation, &add(0, " Shadow "));
    assert_eq!(layer_ids(&animation), [4, 1, 2]);
    assert_eq!(animation.layers()[0].name().as_str(), "Shadow");
    assert!(animation.layers()[0].is_visible());
    assert_eq!(animation.next_id(), 5);
    apply(&mut animation, &add(3, "Top"));
    assert_eq!(layer_ids(&animation), [4, 1, 2, 5]);
}

#[test]
fn a_new_layer_needs_a_position_a_name_and_room() {
    let mut animation = two_layers();
    let error = edit::apply(&mut animation, &add(3, "Top")).unwrap_err();
    assert_eq!(error, EditError::PositionOutOfRange { max: 2 });
    assert_refused(&mut animation, &add(3, "Top"), "edit.position_out_of_range");
    assert_refused(&mut animation, &add(0, "\u{7}"), "document.name");
    while animation.layers().len() < MAX_LAYERS {
        edit::apply(&mut animation, &add(0, "More")).unwrap();
    }
    assert_refused(
        &mut animation,
        &add(0, "One too many"),
        "document.layer_count",
    );
}

#[test]
fn a_deleted_layer_takes_its_cels_but_the_last_one_stays() {
    let mut animation = two_layers();
    let delete = |id| Operation::DeleteLayer {
        layer: LayerId::new(id),
    };
    apply(&mut animation, &delete(2));
    assert_eq!(layer_ids(&animation), [1]);
    assert!(animation.cel(LayerId::new(2), FRAME).is_none());
    assert_eq!(animation.cels().len(), 1);
    assert_refused(&mut animation, &delete(1), "edit.last_layer");
    assert_refused(&mut animation, &delete(2), "edit.layer_not_found");
}

#[test]
fn a_layer_moves_within_the_stack() {
    let mut animation = two_layers();
    let move_layer = |position| Operation::MoveLayer {
        layer: LayerId::new(1),
        position,
    };
    apply(&mut animation, &move_layer(1));
    assert_eq!(layer_ids(&animation), [2, 1]);
    assert_refused(&mut animation, &move_layer(2), "edit.position_out_of_range");
}

#[test]
fn a_layer_is_renamed_shown_and_hidden() {
    let mut animation = two_layers();
    let layer = LayerId::new(2);
    let rename = |name: &str| Operation::RenameLayer {
        layer,
        name: name.to_owned(),
    };
    apply(&mut animation, &rename("Face"));
    apply(
        &mut animation,
        &Operation::SetLayerVisibility {
            layer,
            visible: false,
        },
    );
    let face = animation.layer(layer).unwrap();
    assert_eq!(face.name().as_str(), "Face");
    assert!(!face.is_visible());
    apply(&mut animation, &rename("Visible still hidden"));
    assert!(!animation.layer(layer).unwrap().is_visible());
    assert_refused(&mut animation, &rename(""), "document.name");
    let missing = Operation::RenameLayer {
        layer: LayerId::new(9),
        name: "Ghost".to_owned(),
    };
    assert_refused(&mut animation, &missing, "edit.layer_not_found");
}
