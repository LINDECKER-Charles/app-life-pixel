//! Every invariant of the validation table, each failing with its code and params.

mod common;

use common::{read, refusal, sample_document};
use life_pixel_core::DocumentError;
use life_pixel_core::limits::{
    MAX_DOCUMENT_BYTES, MAX_FRAMES, MAX_LAYERS, MAX_PALETTE_ENTRIES, MAX_TAGS, NAME_MAX_CHARS,
};
use life_pixel_core::serialize::read_document;
use serde_json::{Value, json};

/// A side of 512 pixels: one cel holds 262,144 of the 16,777,216 pixels of the budget.
const FULL_SIDE: u16 = 512;
/// The `rle` of a 512 × 512 cel of index 1: one run of 262,144.
const FULL_CEL_RLE: &str = "gIAQAQ==";

fn tag(name: &str, first: u32, last: u32) -> Value {
    json!({ "name": name, "first": first, "last": last, "loop": "loop" })
}

fn assert_code(error: &DocumentError, code: &str) {
    assert_eq!(error.code(), code, "{error:?}");
}

#[test]
fn the_sample_document_is_valid() {
    assert!(read(&sample_document()).is_ok());
}

#[test]
fn a_malformed_document_is_refused() {
    let changes: [fn(&mut Value); 7] = [
        |document| {
            document.as_object_mut().unwrap().remove("title");
        },
        |document| document["extra"] = json!(true),
        |document| document["layers"][0]["locked"] = json!(false),
        |document| document["width"] = json!("2"),
        |document| document["format"] = json!("life-pixel/project"),
        |document| document["cels"][0]["grid"] = json!(["11", "2."]),
        |document| {
            document["cels"][0].as_object_mut().unwrap().remove("rle");
        },
    ];
    for change in changes {
        assert_eq!(refusal(change), DocumentError::Malformed);
    }
    assert_eq!(
        read_document(b"{\"format\": "),
        Err(DocumentError::Malformed)
    );
}

#[test]
fn an_unknown_version_is_refused_with_its_number() {
    let error = refusal(|document| document["version"] = json!(2));
    assert_code(&error, "document.unsupported_version");
    assert_eq!(Value::Object(error.params()), json!({ "version": 2 }));
    let error = refusal(|document| document["version"] = json!(0));
    assert_eq!(error, DocumentError::UnsupportedVersion { version: 0 });
}

#[test]
fn a_document_above_32_mib_is_refused_before_parsing() {
    let error = read_document(&vec![b' '; MAX_DOCUMENT_BYTES + 1]).unwrap_err();
    assert_code(&error, "document.too_large");
    assert_eq!(
        Value::Object(error.params()),
        json!({ "maxBytes": 33_554_432 })
    );
}

#[test]
fn a_canvas_side_out_of_1_to_512_is_refused() {
    for (side, value) in [("width", 0), ("height", 513), ("width", 70_000)] {
        let error = refusal(|document| document[side] = json!(value));
        assert_code(&error, "document.canvas_size");
        assert_eq!(
            Value::Object(error.params()),
            json!({ "min": 1, "max": 512 })
        );
    }
}

#[test]
fn no_frame_or_too_many_frames_is_refused() {
    let many: Vec<Value> = (0..=MAX_FRAMES)
        .map(|position| json!({ "id": 10 + position, "durationMs": 100 }))
        .collect();
    for frames in [json!([]), Value::Array(many)] {
        let error = refusal(|document| {
            document["frames"] = frames;
            document["cels"] = json!([]);
            document["tags"] = json!([]);
            document["nextId"] = json!(10_000);
        });
        assert_code(&error, "document.frame_count");
        assert_eq!(Value::Object(error.params()), json!({ "max": 1_024 }));
    }
}

#[test]
fn no_layer_or_too_many_layers_is_refused() {
    let many: Vec<Value> = (0..=MAX_LAYERS)
        .map(|position| json!({ "id": 10 + position, "name": "Layer", "visible": true }))
        .collect();
    for layers in [json!([]), Value::Array(many)] {
        let error = refusal(|document| {
            document["layers"] = layers;
            document["cels"] = json!([]);
            document["nextId"] = json!(10_000);
        });
        assert_code(&error, "document.layer_count");
        assert_eq!(Value::Object(error.params()), json!({ "max": 64 }));
    }
}

#[test]
fn more_than_64_tags_are_refused() {
    let tags: Vec<Value> = (0..=MAX_TAGS)
        .map(|number| {
            let name = format!("tag{number}");
            json!({ "name": name, "first": 0, "last": 0, "loop": "once" })
        })
        .collect();
    let error = refusal(|document| document["tags"] = Value::Array(tags));
    assert_code(&error, "document.tag_count");
    assert_eq!(Value::Object(error.params()), json!({ "max": 64 }));
}

#[test]
fn cels_beyond_the_pixel_budget_are_refused() {
    let full_frames = |count: u32| {
        let mut document = sample_document();
        let frames = (0..count).map(|number| json!({ "id": 10 + number, "durationMs": 100 }));
        let cels = (0..count)
            .map(|number| json!({ "layer": 1, "frame": 10 + number, "rle": FULL_CEL_RLE }));
        document["width"] = json!(FULL_SIDE);
        document["height"] = json!(FULL_SIDE);
        document["frames"] = Value::Array(frames.collect());
        document["cels"] = Value::Array(cels.collect());
        document["nextId"] = json!(10_000);
        document
    };
    assert!(read(&full_frames(64)).is_ok());
    let error = read(&full_frames(65)).unwrap_err();
    assert_code(&error, "document.pixel_budget");
    assert_eq!(Value::Object(error.params()), json!({ "max": 16_777_216 }));
}

#[test]
fn an_invalid_palette_is_refused() {
    let too_many = vec![json!("#00000000"); MAX_PALETTE_ENTRIES + 1];
    let palettes = [
        json!([]),
        Value::Array(too_many),
        json!(["#000000ff", "#000000ff", "#ffffffff"]),
        json!(["#00000000", "#000000", "#ffffffff"]),
        json!(["#00000000", "black", "#ffffffff"]),
    ];
    for palette in palettes {
        let error = refusal(|document| document["palette"] = palette);
        assert_code(&error, "document.palette");
        assert_eq!(Value::Object(error.params()), json!({ "max": 256 }));
    }
}

#[test]
fn a_frame_duration_out_of_10_to_65535_ms_is_refused() {
    for duration in [0, 9, 65_536] {
        let error = refusal(|document| document["frames"][1]["durationMs"] = json!(duration));
        assert_code(&error, "document.frame_duration");
        assert_eq!(
            Value::Object(error.params()),
            json!({ "min": 10, "max": 65_535 })
        );
    }
}

#[test]
fn an_invalid_title_or_layer_name_is_refused() {
    let too_long = "a".repeat(NAME_MAX_CHARS + 1);
    let changes = [
        ("title", json!("   ")),
        ("title", json!("line\nbreak")),
        ("layer", json!(too_long)),
    ];
    for (field, name) in changes {
        let error = refusal(|document| match field {
            "title" => document["title"] = name,
            _ => document["layers"][1]["name"] = name,
        });
        assert_code(&error, "document.name");
        assert_eq!(Value::Object(error.params()), json!({ "max": 100 }));
    }
}

#[test]
fn an_invalid_duplicate_or_out_of_range_tag_is_refused_by_name() {
    let cases = [
        (json!([tag("Idle", 0, 0)]), "Idle"),
        (json!([tag("idle", 0, 0), tag("idle", 1, 1)]), "idle"),
        (json!([tag("walk", 0, 2)]), "walk"),
        (json!([tag("back", 1, 0)]), "back"),
        (json!([tag("far", 0, 70_000)]), "far"),
    ];
    for (tags, name) in cases {
        let error = refusal(|document| document["tags"] = tags);
        assert_code(&error, "document.tag");
        assert_eq!(Value::Object(error.params()), json!({ "name": name }));
    }
}

#[test]
fn a_broken_reference_is_refused() {
    let changes: [fn(&mut Value); 6] = [
        |document| document["frames"][0]["id"] = json!(1),
        |document| document["layers"][1]["id"] = json!(1),
        |document| document["nextId"] = json!(4),
        |document| document["cels"][0]["layer"] = json!(9),
        |document| document["cels"][0]["frame"] = json!(1),
        |document| document["cels"][1]["layer"] = json!(1),
    ];
    for change in changes {
        let error = refusal(change);
        assert_eq!(error, DocumentError::Reference);
        assert!(error.params().is_empty());
    }
}

#[test]
fn a_cel_of_the_wrong_size_undecodable_or_outside_the_palette_is_refused() {
    let changes: [fn(&mut Value); 5] = [
        |document| document["cels"][0]["rle"] = json!("AwE="),
        |document| document["cels"][0]["rle"] = json!("not base64"),
        |document| document["cels"][0]["rle"] = json!("BAM="),
        |document| document["cels"][1]["grid"] = json!(["2.", "x."]),
        |document| document["cels"][1]["grid"] = json!(["2."]),
    ];
    for change in changes {
        let error = refusal(change);
        assert_eq!(error, DocumentError::Cel);
        assert!(error.params().is_empty());
    }
}
