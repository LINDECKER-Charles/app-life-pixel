//! Document version 1: an animation as JSON with 2-space indentation, fields in a fixed order,
//! cels sorted by layer id then frame id — the same animation always gives the same bytes.

use std::collections::BTreeSet;

use super::document_v1::{CelV1, DocumentV1, FrameV1, LayerV1, TagV1};
use super::{grid, rle};
use crate::error::DocumentError;
use crate::limits::MAX_DOCUMENT_BYTES;
use crate::model::{
    Animation, AnimationParts, Cel, CelShape, Frame, FrameId, Layer, LayerId, Name, Palette, Rgba,
    Tag, TagName,
};

/// The `format` of every document.
pub const DOCUMENT_FORMAT: &str = "life-pixel/animation";
/// The document `version` this build writes.
pub const DOCUMENT_VERSION: u64 = 1;

/// The document of `animation`, in version [`DOCUMENT_VERSION`], ending with a line break.
///
/// # Errors
///
/// [`DocumentError::TooLarge`] when the document would exceed
/// [`MAX_DOCUMENT_BYTES`](crate::limits::MAX_DOCUMENT_BYTES): it could not be read back.
pub fn write_document(animation: &Animation) -> Result<String, DocumentError> {
    let document = to_document(animation);
    let mut text = serde_json::to_string_pretty(&document).map_err(|_| DocumentError::Malformed)?;
    text.push('\n');
    (text.len() <= MAX_DOCUMENT_BYTES)
        .then_some(text)
        .ok_or(DocumentError::TooLarge)
}

/// The animation of a version 1 document whose `format` and `version` are already checked.
pub(super) fn read_v1(bytes: &[u8]) -> Result<Animation, DocumentError> {
    let document: DocumentV1 =
        serde_json::from_slice(bytes).map_err(|_| DocumentError::Malformed)?;
    let mut animation = Animation::from_parts(AnimationParts {
        title: Name::new(&document.title)?,
        width: read_side(document.width)?,
        height: read_side(document.height)?,
        palette: read_palette(&document.palette)?,
        layers: document
            .layers
            .iter()
            .map(read_layer)
            .collect::<Result<_, _>>()?,
        frames: document
            .frames
            .iter()
            .map(read_frame)
            .collect::<Result<_, _>>()?,
        tags: document
            .tags
            .iter()
            .map(read_tag)
            .collect::<Result<_, _>>()?,
        next_id: document.next_id,
    })?;
    read_cels(document.cels, &mut animation)?;
    Ok(animation)
}

fn read_side(side: u32) -> Result<u16, DocumentError> {
    u16::try_from(side).map_err(|_| DocumentError::CanvasSize)
}

fn read_palette(entries: &[String]) -> Result<Palette, DocumentError> {
    let colours = entries.iter().map(|entry| entry.parse::<Rgba>());
    Palette::new(colours.collect::<Result<_, _>>()?)
}

fn read_layer(layer: &LayerV1) -> Result<Layer, DocumentError> {
    let name = Name::new(&layer.name)?;
    Ok(Layer::new(LayerId::new(layer.id), name).with_visibility(layer.visible))
}

fn read_frame(frame: &FrameV1) -> Result<Frame, DocumentError> {
    let duration_ms = u16::try_from(frame.duration_ms).map_err(|_| DocumentError::FrameDuration)?;
    Frame::new(FrameId::new(frame.id), duration_ms)
}

fn read_tag(tag: &TagV1) -> Result<Tag, DocumentError> {
    let name = TagName::new(&tag.name)?;
    let out_of_range = || DocumentError::Tag {
        name: tag.name.clone(),
    };
    let first = u16::try_from(tag.first).map_err(|_| out_of_range())?;
    let last = u16::try_from(tag.last).map_err(|_| out_of_range())?;
    Ok(Tag::new(name, first..=last, tag.loop_mode))
}

/// Adds each cel in document order, so that the pixel budget bounds what is decoded.
fn read_cels(cels: Vec<CelV1>, animation: &mut Animation) -> Result<(), DocumentError> {
    let mut keys = BTreeSet::new();
    for cel in cels {
        let key = (LayerId::new(cel.layer), FrameId::new(cel.frame));
        if !keys.insert(key) {
            return Err(DocumentError::Reference);
        }
        let pixels = read_cel(cel, animation.cel_shape())?;
        animation.insert_cel(key, pixels)?;
    }
    Ok(())
}

fn read_cel(cel: CelV1, shape: CelShape) -> Result<Cel, DocumentError> {
    match (cel.rle, cel.grid) {
        (Some(text), None) => rle::decode(&text, shape.pixel_count()).map(Cel::new),
        (None, Some(rows)) => grid::parse(&rows, shape).map_err(|_| DocumentError::Cel),
        _ => Err(DocumentError::Malformed),
    }
}

fn to_document(animation: &Animation) -> DocumentV1 {
    DocumentV1 {
        format: DOCUMENT_FORMAT.to_owned(),
        version: DOCUMENT_VERSION,
        title: animation.title().to_string(),
        width: u32::from(animation.width()),
        height: u32::from(animation.height()),
        palette: animation
            .palette()
            .entries()
            .iter()
            .map(Rgba::to_string)
            .collect(),
        layers: animation.layers().iter().map(to_layer).collect(),
        frames: animation.frames().iter().map(to_frame).collect(),
        cels: animation.cels().iter().map(to_cel).collect(),
        tags: animation.tags().iter().map(to_tag).collect(),
        next_id: animation.next_id(),
    }
}

fn to_layer(layer: &Layer) -> LayerV1 {
    LayerV1 {
        id: layer.id().get(),
        name: layer.name().to_string(),
        visible: layer.is_visible(),
    }
}

fn to_frame(frame: &Frame) -> FrameV1 {
    FrameV1 {
        id: frame.id().get(),
        duration_ms: u32::from(frame.duration_ms()),
    }
}

fn to_cel(((layer, frame), cel): (&(LayerId, FrameId), &Cel)) -> CelV1 {
    CelV1 {
        layer: layer.get(),
        frame: frame.get(),
        rle: Some(rle::encode(cel.indices())),
        grid: None,
    }
}

fn to_tag(tag: &Tag) -> TagV1 {
    TagV1 {
        name: tag.name().to_string(),
        first: u32::from(tag.first()),
        last: u32::from(tag.last()),
        loop_mode: tag.loop_mode(),
    }
}
