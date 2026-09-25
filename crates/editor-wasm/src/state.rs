//! What the engine publishes after every command: the interface's `EngineState` and the
//! `DocumentSummary` of the open animation.

use life_pixel_core::edit::TagSpec;
use life_pixel_core::{Animation, FrameId, LayerId, Limits, Rgba};
use serde::Serialize;

/// The interface's `EngineState`, less the `starting` and `failed` statuses, which only the
/// adapter in the page knows.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct EngineState {
    /// Whether an animation is open.
    pub status: Status,
    /// The open animation; `None` when [`Status::Empty`].
    pub document: Option<DocumentSummary>,
    /// Whether there is a step to undo.
    pub can_undo: bool,
    /// Whether there is a step to redo.
    pub can_redo: bool,
    /// Whether the animation changed since it was created, opened or last saved.
    pub has_unsaved_work: bool,
    /// The limits of this build.
    pub limits: Limits,
}

/// Serialized `"empty"` or `"ready"`.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Status {
    /// No animation is open yet.
    Empty,
    /// An animation is open.
    Ready,
}

/// The interface's `DocumentSummary`: everything but the pixels.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DocumentSummary {
    /// The title.
    pub title: String,
    /// The canvas width, in pixels.
    pub width: u16,
    /// The canvas height, in pixels.
    pub height: u16,
    /// The palette, written `#rrggbbaa`; entry 0 is transparent.
    pub palette: Vec<Rgba>,
    /// The layers, from bottom to top.
    pub layers: Vec<LayerSummary>,
    /// The frames, in play order.
    pub frames: Vec<FrameSummary>,
    /// The tags.
    pub tags: Vec<TagSpec>,
}

/// A layer of a [`DocumentSummary`].
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LayerSummary {
    /// The layer's id.
    pub id: LayerId,
    /// Its name.
    pub name: String,
    /// Whether it is shown.
    pub visible: bool,
}

/// A frame of a [`DocumentSummary`].
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FrameSummary {
    /// The frame's id.
    pub id: FrameId,
    /// How long it shows, in milliseconds.
    pub duration_ms: u16,
}

impl From<&Animation> for DocumentSummary {
    fn from(animation: &Animation) -> Self {
        let layers = animation.layers().iter().map(|layer| LayerSummary {
            id: layer.id(),
            name: layer.name().to_string(),
            visible: layer.is_visible(),
        });
        let frames = animation.frames().iter().map(|frame| FrameSummary {
            id: frame.id(),
            duration_ms: frame.duration_ms(),
        });
        Self {
            title: animation.title().to_string(),
            width: animation.width(),
            height: animation.height(),
            palette: animation.palette().entries().to_vec(),
            layers: layers.collect(),
            frames: frames.collect(),
            tags: animation.tags().iter().map(TagSpec::from).collect(),
        }
    }
}
