//! What an agent reads of an animation: its structure, without pixels unless asked.

use life_pixel_core::edit::TagSpec;
use life_pixel_core::{Animation, Layer, Rgba};
use serde::Serialize;

use crate::ids::{AnimationId, ProjectId};
use crate::ports::library_store::AnimationRecord;

/// An animation as the MCP tools return it, field names in `snake_case`.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct AnimationView {
    /// The animation's id.
    pub id: AnimationId,
    /// The project it belongs to.
    pub project_id: ProjectId,
    /// The version of its document.
    pub version: u64,
    /// Its title: user content, returned as data.
    pub title: String,
    /// The canvas width, in pixels.
    pub width: u16,
    /// The canvas height, in pixels.
    pub height: u16,
    /// The palette as `#rrggbbaa`, entry 0 — transparent — first.
    pub palette: Vec<Rgba>,
    /// The layers from the bottom to the top.
    pub layers: Vec<LayerView>,
    /// The frames in play order.
    pub frames: Vec<FrameView>,
    /// The tags: named frame ranges and how they play at their end.
    pub tags: Vec<TagSpec>,
    /// The composites of the frames asked for, as text grids; empty unless asked.
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub grids: Vec<FrameGrid>,
}

/// A layer: the id that addresses it, its name, and whether it shows.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct LayerView {
    /// The id the editing use cases take.
    pub id: u32,
    /// Its name: user content, returned as data.
    pub name: String,
    /// Whether it shows in the composite.
    pub visible: bool,
}

/// A frame: its position and how long it shows.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct FrameView {
    /// Its position in play order, from 0.
    pub position: u16,
    /// How long it shows, in milliseconds.
    pub duration_ms: u16,
}

/// A frame's composite as a text grid, one string per row, top to bottom.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct FrameGrid {
    /// The frame's position, from 0.
    pub frame: u16,
    /// The rows, in the grid alphabet of `core`.
    pub rows: Vec<String>,
}

impl AnimationView {
    /// The view of `animation`, stored as `record`, without grids.
    #[must_use]
    pub fn of(record: &AnimationRecord, animation: &Animation) -> Self {
        Self {
            id: record.id,
            project_id: record.project,
            version: record.version,
            title: animation.title().as_str().to_owned(),
            width: animation.width(),
            height: animation.height(),
            palette: animation.palette().entries().to_vec(),
            layers: animation.layers().iter().map(LayerView::of).collect(),
            frames: frames(animation),
            tags: animation.tags().iter().map(TagSpec::from).collect(),
            grids: Vec::new(),
        }
    }
}

impl LayerView {
    fn of(layer: &Layer) -> Self {
        Self {
            id: layer.id().get(),
            name: layer.name().as_str().to_owned(),
            visible: layer.is_visible(),
        }
    }
}

fn frames(animation: &Animation) -> Vec<FrameView> {
    let positions = (0..=u16::MAX).zip(animation.frames());
    positions
        .map(|(position, frame)| FrameView {
            position,
            duration_ms: frame.duration_ms(),
        })
        .collect()
}
