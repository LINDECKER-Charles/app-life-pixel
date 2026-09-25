//! The formats an animation exports to.

use serde::{Deserialize, Serialize};

/// Serialized `"wasm"`, `"gif"`, `"apng"`, `"sprite_sheet"`, `"png_frames"`: the names the API,
/// the MCP tools, the CLI and the interface all use.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ExportFormat {
    /// The self-contained WebAssembly bundle: the player and the animation's payload.
    Wasm,
    /// An animated GIF.
    Gif,
    /// An animated PNG.
    Apng,
    /// Every frame in one PNG, with a JSON describing where each one is.
    SpriteSheet,
    /// A zip of one PNG per frame.
    PngFrames,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn formats_serialize_as_their_shared_names() {
        let formats = [
            ExportFormat::Wasm,
            ExportFormat::Gif,
            ExportFormat::Apng,
            ExportFormat::SpriteSheet,
            ExportFormat::PngFrames,
        ];
        let names = serde_json::to_value(formats).unwrap();
        let expected = ["wasm", "gif", "apng", "sprite_sheet", "png_frames"];
        assert_eq!(names, serde_json::json!(expected));
        let parsed: Vec<ExportFormat> = serde_json::from_value(names).unwrap();
        assert_eq!(parsed, formats);
    }
}
