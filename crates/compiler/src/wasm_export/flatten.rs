//! An animation as the format encodes it: every frame composited into palette indices by
//! `life-pixel-core`, with the palette, the title and the tags.

use life_pixel_core::{Animation, Frame, LoopMode, Palette, Rgba, Tag, render};
use life_pixel_format::{AnimationData, FrameData, TagData};

/// `animation` flattened: a payload knows no layer, only each frame's composited indices.
pub(super) fn animation_data(animation: &Animation) -> AnimationData {
    AnimationData {
        width: animation.width(),
        height: animation.height(),
        palette: palette(animation.palette()),
        title: animation.title().as_str().to_owned(),
        tags: animation.tags().iter().map(tag).collect(),
        frames: animation
            .frames()
            .iter()
            .map(|frame| frame_data(animation, frame))
            .collect(),
    }
}

/// The palette's entries, entry 0 written `00 00 00 00` as payload v1 requires: `core` only
/// requires it fully transparent, and a fully transparent pixel shows nothing whatever its red,
/// green and blue.
fn palette(palette: &Palette) -> Vec<life_pixel_format::Rgba> {
    let mut entries: Vec<_> = palette.entries().iter().copied().map(colour).collect();
    if let Some(transparent) = entries.first_mut() {
        *transparent = life_pixel_format::Rgba::default();
    }
    entries
}

fn colour(Rgba { r, g, b, a }: Rgba) -> life_pixel_format::Rgba {
    life_pixel_format::Rgba { r, g, b, a }
}

fn tag(tag: &Tag) -> TagData {
    TagData {
        name: tag.name().as_str().to_owned(),
        first: tag.first(),
        last: tag.last(),
        loop_mode: match tag.loop_mode() {
            LoopMode::Loop => life_pixel_format::LoopMode::Loop,
            LoopMode::Once => life_pixel_format::LoopMode::Once,
        },
    }
}

fn frame_data(animation: &Animation, frame: &Frame) -> FrameData {
    FrameData {
        duration_ms: frame.duration_ms(),
        indices: render::composite(animation, frame.id()),
    }
}

#[cfg(test)]
mod tests {
    use life_pixel_core::serialize::read_document;

    use super::*;

    const DOCUMENT: &str = r##"{
      "format": "life-pixel/animation", "version": 1, "title": "Two layers", "width": 2,
      "height": 1, "palette": ["#ff000000", "#00ff00ff", "#0000ff80"],
      "layers": [{ "id": 1, "name": "Back", "visible": true },
                 { "id": 2, "name": "Front", "visible": true }],
      "frames": [{ "id": 3, "durationMs": 40 }, { "id": 4, "durationMs": 60 }],
      "cels": [{ "layer": 1, "frame": 3, "grid": ["11"] },
               { "layer": 2, "frame": 3, "grid": [".2"] }],
      "tags": [{ "name": "end", "first": 1, "last": 1, "loop": "once" }],
      "nextId": 5
    }"##;

    #[test]
    fn frames_are_flattened_with_their_durations_palette_title_and_tags() {
        let animation = read_document(DOCUMENT.as_bytes()).unwrap();

        let data = animation_data(&animation);

        assert_eq!((data.width, data.height), (2, 1));
        assert_eq!(data.title, "Two layers");
        let frames: Vec<_> = data
            .frames
            .iter()
            .map(|f| (f.duration_ms, &f.indices[..]))
            .collect();
        assert_eq!(frames, [(40, &[1, 2][..]), (60, &[0, 0][..])]);
        let tag = TagData {
            name: "end".to_owned(),
            first: 1,
            last: 1,
            loop_mode: life_pixel_format::LoopMode::Once,
        };
        assert_eq!(data.tags, [tag]);
    }

    #[test]
    fn palette_entry_0_is_written_all_zeros_and_the_others_as_they_are() {
        let animation = read_document(DOCUMENT.as_bytes()).unwrap();

        let palette = animation_data(&animation).palette;

        let bytes: Vec<_> = palette.iter().map(|c| [c.r, c.g, c.b, c.a]).collect();
        assert_eq!(bytes, [[0, 0, 0, 0], [0, 255, 0, 255], [0, 0, 255, 128]]);
    }
}
