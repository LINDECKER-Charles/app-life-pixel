//! An `export` tool call, as the transport's delivery receives it.

use life_pixel_compiler::ExportFormat;
use life_pixel_service::AnimationId;
use serde_json::{Map, Value};

/// What an `export` call asked for, once its common arguments are checked.
#[derive(Clone, Debug, PartialEq)]
pub struct ExportCall {
    /// The animation exported.
    pub id: AnimationId,
    /// The version of its document read just before the files were compiled: a newer version
    /// means the animation changed since.
    pub version: u64,
    /// The format of the files.
    pub format: ExportFormat,
    /// The tag whose frames were exported; the whole animation when `None`.
    pub tag: Option<String>,
    /// The scale of the classic formats; the export's own when `None`.
    pub scale: Option<u8>,
    /// Every other argument of the call, as the client sent it: the transport's own, described
    /// by [`ExportDelivery::export_schema`](crate::ExportDelivery::export_schema).
    pub options: Map<String, Value>,
}
