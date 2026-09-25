use crate::model::{Animation, Cel, FrameId, LayerId};

/// A layer's cel standing in for the one the animation holds, for previews.
pub type Replacement<'cel> = (LayerId, &'cel Cel);

/// The palette indices of `frame`, row by row: index 0 everywhere, then, for each visible layer
/// from bottom to top, every non-zero index of its cel. A frame the animation lacks is blank.
#[must_use]
pub fn composite(animation: &Animation, frame: FrameId) -> Vec<u8> {
    compose(animation, frame, None)
}

/// [`composite`], with `replacement`'s cel in place of its layer's own on `frame`. A replacement
/// of the wrong size paints only the pixels both have.
#[must_use]
pub fn composite_with(animation: &Animation, frame: FrameId, replacement: Replacement) -> Vec<u8> {
    compose(animation, frame, Some(replacement))
}

fn compose(animation: &Animation, frame: FrameId, replacement: Option<Replacement>) -> Vec<u8> {
    let mut pixels = vec![0; animation.cel_shape().pixel_count()];
    let visible_layers = animation.layers().iter().filter(|layer| layer.is_visible());
    for layer in visible_layers {
        let cel = match replacement {
            Some((replaced, cel)) if replaced == layer.id() => Some(cel),
            _ => animation.cel(layer.id(), frame),
        };
        if let Some(cel) = cel {
            paint(&mut pixels, cel);
        }
    }
    pixels
}

/// Copies every non-zero index of `cel` over `pixels`.
fn paint(pixels: &mut [u8], cel: &Cel) {
    let covered = pixels.iter_mut().zip(cel.indices());
    for (pixel, &index) in covered.filter(|(_, index)| **index != 0) {
        *pixel = index;
    }
}
