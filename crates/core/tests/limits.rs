//! `Limits::current()`, as the engine gives it to the interface.

use life_pixel_core::Limits;
use serde_json::{Value, json};

/// The number of constants in core.md's limits table.
const LIMIT_COUNT: usize = 37;

#[test]
fn limits_serialize_one_camel_case_field_per_constant() {
    let limits = serde_json::to_value(Limits::current()).unwrap();
    let Value::Object(fields) = &limits else {
        panic!("an object expected")
    };
    assert_eq!(fields.len(), LIMIT_COUNT);
    assert!(
        fields
            .keys()
            .all(|name| name.chars().all(|c| c.is_ascii_alphabetic()))
    );
    let expected = [
        ("canvasMinSide", json!(1)),
        ("canvasMaxSide", json!(512)),
        ("maxFrames", json!(1_024)),
        ("maxCelPixels", json!(16_777_216)),
        ("maxPaletteEntries", json!(256)),
        ("defaultFrameDurationMs", json!(100)),
        ("maxDocumentBytes", json!(33_554_432)),
        ("historyMaxBytes", json!(67_108_864)),
        ("exportMaxSide", json!(8_192)),
        ("tokenExpiryDays", json!([30, 90, 365])),
        ("mcpPageSizeMax", json!(50)),
    ];
    for (name, value) in expected {
        assert_eq!(limits[name], value, "{name}");
    }
}
