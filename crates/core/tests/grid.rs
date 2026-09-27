//! The text grid: single and pair modes, both ways, and each of its errors.

use life_pixel_core::serialize::grid;
use life_pixel_core::{CelShape, DocumentError};
use serde_json::{Value, json};

fn shape(width: u16, height: u16, palette_len: usize) -> CelShape {
    CelShape {
        width,
        height,
        palette_len,
    }
}

#[test]
fn single_mode_writes_one_character_per_pixel() {
    let shape = shape(4, 2, 62);
    let indices = [0, 1, 9, 10, 35, 36, 61, 0];
    let rows = grid::format(&indices, shape);
    assert_eq!(rows, [".19a", "zAZ."]);
    assert_eq!(grid::parse(&rows, shape).unwrap().indices(), indices);
}

#[test]
fn pair_mode_writes_two_hexadecimal_digits_above_62_entries() {
    let shape = shape(3, 1, 256);
    let indices = [0, 62, 255];
    let rows = grid::format(&indices, shape);
    assert_eq!(rows, ["003eff"]);
    assert_eq!(grid::parse(&rows, shape).unwrap().indices(), indices);
    assert_eq!(grid::format(&[0, 1], self::shape(2, 1, 63)), ["0001"]);
}

#[test]
fn parse_picks_the_mode_from_the_row_length() {
    let shape = shape(2, 1, 16);
    assert_eq!(grid::parse(&["1f"], shape).unwrap().indices(), [1, 15]);
    assert_eq!(grid::parse(&["010f"], shape).unwrap().indices(), [1, 15]);
}

#[test]
fn a_grid_of_the_wrong_size_is_refused() {
    let shape = shape(2, 2, 16);
    let grids: [&[&str]; 5] = [
        &["..", "..", ".."],
        &[".."],
        &["...", "..."],
        &["..", "0000"],
        &[],
    ];
    for rows in grids {
        let error = grid::parse(rows, shape).unwrap_err();
        assert_eq!(error.code(), "grid.size");
        assert_eq!(
            Value::Object(error.params()),
            json!({ "width": 2, "height": 2 })
        );
    }
}

#[test]
fn a_character_outside_the_alphabet_is_refused_where_it_is() {
    let shape = shape(3, 2, 256);
    let cases: [(&[&str], usize, usize); 4] = [
        (&["...", "..#"], 1, 2),
        (&["...", "é.."], 1, 0),
        (&["000000", "00FF00"], 1, 1),
        (&["00 000", "000000"], 0, 1),
    ];
    for (rows, row, column) in cases {
        let error = grid::parse(rows, shape).unwrap_err();
        assert_eq!(
            error,
            DocumentError::GridCharacter { row, column },
            "{rows:?}"
        );
        assert_eq!(
            Value::Object(error.params()),
            json!({ "row": row, "column": column })
        );
    }
}

#[test]
fn an_index_outside_the_palette_is_refused_where_it_is() {
    let error = grid::parse(&["...", ".2f"], shape(3, 2, 3)).unwrap_err();
    assert_eq!(error.code(), "grid.index");
    let params = json!({ "row": 1, "column": 2, "index": 15 });
    assert_eq!(Value::Object(error.params()), params);
    let error = grid::parse(&["0003"], shape(2, 1, 3)).unwrap_err();
    assert_eq!(
        error,
        DocumentError::GridIndex {
            row: 0,
            column: 1,
            index: 3
        }
    );
}
