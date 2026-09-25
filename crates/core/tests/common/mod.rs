//! Documents shared by the integration tests.

#![allow(dead_code)] // Each test file uses its own share of the helpers.

use life_pixel_core::serialize::read_document;
use life_pixel_core::{Animation, DocumentError};
use serde_json::{Value, json};

/// A 2 × 2 animation: two layers, two frames, one cel in `rle` and one in `grid`, one tag.
/// Layer 1 frame 3 is `[1, 1, 2, 0]`; layer 2 frame 3 is `[2, 0, 1, 0]`.
pub fn sample_document() -> Value {
    json!({
        "format": "life-pixel/animation",
        "version": 1,
        "title": "Mascot",
        "width": 2,
        "height": 2,
        "palette": ["#00000000", "#000000ff", "#ffffffff"],
        "layers": [
            { "id": 1, "name": "Body", "visible": true },
            { "id": 2, "name": "Face", "visible": true }
        ],
        "frames": [{ "id": 3, "durationMs": 100 }, { "id": 4, "durationMs": 150 }],
        "cels": [
            { "layer": 1, "frame": 3, "rle": "AgEBAgEA" },
            { "layer": 2, "frame": 3, "grid": ["2.", "1."] }
        ],
        "tags": [{ "name": "idle", "first": 0, "last": 1, "loop": "loop" }],
        "nextId": 5
    })
}

/// The animation of `document`.
pub fn read(document: &Value) -> Result<Animation, DocumentError> {
    read_document(document.to_string().as_bytes())
}

/// The error the sample document fails with once `change` is applied to it.
pub fn refusal(change: impl FnOnce(&mut Value)) -> DocumentError {
    let mut document = sample_document();
    change(&mut document);
    match read(&document) {
        Err(error) => error,
        Ok(_) => panic!("the changed document should be refused: {document}"),
    }
}
