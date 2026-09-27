//! The shape of document version 1, field for field and in the order it is written. Numbers are
//! read wider than the model holds them, so that an out-of-range value fails with its own code
//! rather than as malformed.

use serde::{Deserialize, Serialize};

use crate::model::LoopMode;

/// A whole document.
#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub(super) struct DocumentV1 {
    pub(super) format: String,
    pub(super) version: u64,
    pub(super) title: String,
    pub(super) width: u32,
    pub(super) height: u32,
    pub(super) palette: Vec<String>,
    pub(super) layers: Vec<LayerV1>,
    pub(super) frames: Vec<FrameV1>,
    pub(super) cels: Vec<CelV1>,
    pub(super) tags: Vec<TagV1>,
    pub(super) next_id: u32,
}

/// A layer.
#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub(super) struct LayerV1 {
    pub(super) id: u32,
    pub(super) name: String,
    pub(super) visible: bool,
}

/// A frame.
#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub(super) struct FrameV1 {
    pub(super) id: u32,
    pub(super) duration_ms: u32,
}

/// A cel: exactly one of `rle` and `grid`. The writer always writes `rle`.
#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub(super) struct CelV1 {
    pub(super) layer: u32,
    pub(super) frame: u32,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(super) rle: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(super) grid: Option<Vec<String>>,
}

/// A tag.
#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub(super) struct TagV1 {
    pub(super) name: String,
    pub(super) first: u32,
    pub(super) last: u32,
    #[serde(rename = "loop")]
    pub(super) loop_mode: LoopMode,
}
