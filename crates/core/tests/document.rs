//! The JSON document: written byte for byte the same, `grid` cels read like their `rle` twins.

mod common;

use common::{read, sample_document};
use life_pixel_core::serialize::{read_document, write_document};
use life_pixel_core::{FrameId, LayerId};
use serde_json::json;

/// The sample document as the writer writes it: fields in order, `rle` cels sorted.
const CANONICAL: &str = r##"{
  "format": "life-pixel/animation",
  "version": 1,
  "title": "Mascot",
  "width": 2,
  "height": 2,
  "palette": [
    "#00000000",
    "#000000ff",
    "#ffffffff"
  ],
  "layers": [
    {
      "id": 1,
      "name": "Body",
      "visible": true
    },
    {
      "id": 2,
      "name": "Face",
      "visible": true
    }
  ],
  "frames": [
    {
      "id": 3,
      "durationMs": 100
    },
    {
      "id": 4,
      "durationMs": 150
    }
  ],
  "cels": [
    {
      "layer": 1,
      "frame": 3,
      "rle": "AgEBAgEA"
    },
    {
      "layer": 2,
      "frame": 3,
      "rle": "AQIBAAEBAQA="
    }
  ],
  "tags": [
    {
      "name": "idle",
      "first": 0,
      "last": 1,
      "loop": "loop"
    }
  ],
  "nextId": 5
}
"##;

#[test]
fn a_document_round_trips_byte_for_byte() {
    let animation = read_document(CANONICAL.as_bytes()).unwrap();
    assert_eq!(write_document(&animation).unwrap(), CANONICAL);
}

#[test]
fn the_writer_sorts_cels_and_writes_rle_only() {
    let mut document = sample_document();
    let cels = document["cels"].as_array_mut().unwrap();
    cels.reverse();
    let animation = read(&document).unwrap();
    assert_eq!(write_document(&animation).unwrap(), CANONICAL);
}

#[test]
fn grid_cels_read_as_their_rle_twins() {
    let mut twin = sample_document();
    twin["cels"][1] = json!({ "layer": 2, "frame": 3, "rle": "AQIBAAEBAQA=" });
    let from_grid = read(&sample_document()).unwrap();
    assert_eq!(from_grid, read(&twin).unwrap());
    let face = from_grid.cel(LayerId::new(2), FrameId::new(3)).unwrap();
    assert_eq!(face.indices(), [2, 0, 1, 0]);
}

#[test]
fn a_blank_cel_is_read_and_left_out() {
    let mut document = sample_document();
    document["cels"][1]["grid"] = json!(["..", ".."]);
    let animation = read(&document).unwrap();
    assert_eq!(animation.cels().len(), 1);
    assert!(animation.cel(LayerId::new(2), FrameId::new(3)).is_none());
}

#[test]
fn names_are_trimmed_and_colours_lowercased_on_read() {
    let mut document = sample_document();
    document["title"] = json!("  Mascot  ");
    document["palette"][2] = json!("#FFFFFFFF");
    let animation = read(&document).unwrap();
    assert_eq!(write_document(&animation).unwrap(), CANONICAL);
}
