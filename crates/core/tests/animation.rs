//! A new animation: its defaults, its limits, and its document.

use life_pixel_core::serialize::{read_document, write_document};
use life_pixel_core::{
    Animation, DEFAULT_PALETTE, DocumentError, FrameId, LayerId, Name, NewAnimation, Palette, Rgba,
};

fn name(text: &str) -> Name {
    Name::new(text).unwrap_or_else(|error| panic!("{text:?}: {error}"))
}

fn spec(width: u16, height: u16) -> NewAnimation {
    NewAnimation {
        title: name("Mascot"),
        width,
        height,
        layer_name: name("Layer 1"),
        frame_duration_ms: None,
        palette: None,
    }
}

#[test]
fn a_new_animation_has_one_layer_one_blank_frame_and_no_tag() {
    let animation = Animation::new(spec(32, 16)).unwrap();
    assert_eq!(animation.title().as_str(), "Mascot");
    assert_eq!((animation.width(), animation.height()), (32, 16));
    assert_eq!(animation.palette().entries(), DEFAULT_PALETTE);
    let [layer] = animation.layers() else {
        panic!("one layer expected")
    };
    assert_eq!(
        (layer.id(), layer.name().as_str()),
        (LayerId::new(1), "Layer 1")
    );
    assert!(layer.is_visible());
    let [frame] = animation.frames() else {
        panic!("one frame expected")
    };
    assert_eq!((frame.id(), frame.duration_ms()), (FrameId::new(2), 100));
    assert!(animation.cels().is_empty() && animation.tags().is_empty());
    assert_eq!(animation.next_id(), 3);
}

#[test]
fn a_new_animation_takes_the_given_palette_and_duration() {
    let palette = Palette::new(vec![Rgba::default(), Rgba::from_u32(0x1020_30ff)]).unwrap();
    let animation = Animation::new(NewAnimation {
        frame_duration_ms: Some(40),
        palette: Some(palette.clone()),
        ..spec(1, 512)
    })
    .unwrap();
    assert_eq!(animation.palette(), &palette);
    assert_eq!(animation.frames()[0].duration_ms(), 40);
}

#[test]
fn a_new_animation_reads_back_from_its_document() {
    let animation = Animation::new(spec(8, 8)).unwrap();
    let document = write_document(&animation).unwrap();
    assert_eq!(read_document(document.as_bytes()).unwrap(), animation);
}

#[test]
fn a_new_animation_out_of_its_limits_is_refused() {
    for (width, height) in [(0, 8), (8, 513)] {
        let refused = Animation::new(spec(width, height));
        assert_eq!(refused, Err(DocumentError::CanvasSize));
    }
    let too_short = NewAnimation {
        frame_duration_ms: Some(9),
        ..spec(8, 8)
    };
    assert_eq!(Animation::new(too_short), Err(DocumentError::FrameDuration));
}
