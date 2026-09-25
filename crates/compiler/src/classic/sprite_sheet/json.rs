//! The sheet's description, in Aseprite's "array" layout, which most game engines import.

use life_pixel_core::{LoopMode, Tag};
use serde::Serialize;

use super::grid::Grid;
use crate::ExportError;
use crate::classic::plan::Plan;

/// The application that wrote the sheet.
const APP: &str = "Life Pixel";
/// The version of this layout.
const LAYOUT_VERSION: &str = "1";
/// The pixel format Aseprite names; the PNG itself is indexed.
const PIXEL_FORMAT: &str = "RGBA8888";
/// Every tag plays forward.
const FORWARD: &str = "forward";
/// The `repeat` of a tag played once.
const REPEAT_ONCE: &str = "1";

#[derive(Serialize)]
struct Description<'plan> {
    frames: Vec<FrameEntry>,
    meta: Meta<'plan>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct FrameEntry {
    filename: String,
    frame: Rectangle,
    rotated: bool,
    trimmed: bool,
    sprite_source_size: Rectangle,
    source_size: Size,
    duration: u16,
}

#[derive(Serialize)]
struct Rectangle {
    x: u32,
    y: u32,
    w: u32,
    h: u32,
}

#[derive(Serialize)]
struct Size {
    w: u32,
    h: u32,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct Meta<'plan> {
    app: &'static str,
    version: &'static str,
    image: &'plan str,
    format: &'static str,
    size: Size,
    scale: String,
    frame_tags: Vec<FrameTag>,
}

#[derive(Serialize)]
struct FrameTag {
    name: String,
    from: usize,
    to: usize,
    direction: &'static str,
    #[serde(skip_serializing_if = "Option::is_none")]
    repeat: Option<&'static str>,
}

/// The JSON of the sheet `image`, laid out as `grid`: 2-space indentation, then a newline.
pub(super) fn describe(plan: &Plan, grid: &Grid, image: &str) -> Result<Vec<u8>, ExportError> {
    let description = Description {
        frames: frame_entries(plan, grid),
        meta: Meta {
            app: APP,
            version: LAYOUT_VERSION,
            image,
            format: PIXEL_FORMAT,
            size: Size {
                w: grid.width,
                h: grid.height,
            },
            scale: plan.scale.to_string(),
            frame_tags: frame_tags(plan),
        },
    };
    let mut bytes = serde_json::to_vec_pretty(&description).map_err(ExportError::encoding)?;
    bytes.push(b'\n');
    Ok(bytes)
}

/// Each frame of the range, named `<stem> <position>`, with its cell and its duration.
fn frame_entries(plan: &Plan, grid: &Grid) -> Vec<FrameEntry> {
    let stem = plan.stem();
    let (w, h) = (grid.cell_width, grid.cell_height);
    let frames = plan.range.frames.iter().enumerate();
    frames
        .map(|(position, frame)| {
            let (x, y) = grid.origin(position);
            FrameEntry {
                filename: format!("{stem} {position}"),
                frame: Rectangle { x, y, w, h },
                rotated: false,
                trimmed: false,
                sprite_source_size: Rectangle { x: 0, y: 0, w, h },
                source_size: Size { w, h },
                duration: frame.duration_ms(),
            }
        })
        .collect()
}

/// The tags whose frames all lie in the range, their positions counted from its first frame.
fn frame_tags(plan: &Plan) -> Vec<FrameTag> {
    let range = &plan.range;
    let tags = plan.animation().tags().iter();
    let held = tags.filter(|tag| range.holds(usize::from(tag.first()), usize::from(tag.last())));
    held.map(|tag| frame_tag(tag, range.first)).collect()
}

fn frame_tag(tag: &Tag, offset: usize) -> FrameTag {
    FrameTag {
        name: tag.name().as_str().to_owned(),
        from: usize::from(tag.first()) - offset,
        to: usize::from(tag.last()) - offset,
        direction: FORWARD,
        repeat: (tag.loop_mode() == LoopMode::Once).then_some(REPEAT_ONCE),
    }
}
